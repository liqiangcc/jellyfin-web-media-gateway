//! Production source-to-session preparation and atomic publication.
//!
//! This module owns only the composition of already accepted generic
//! contracts. Site knowledge stays behind `SiteAdapterRegistry`; display
//! registration/liveness stays behind `DisplaySessionService`; Playback
//! authority stays behind `ControlService`.

use crate::browser::{
    BrowserObservationHandoff, BrowserNavigationRequest, BrowserWorker, R008NavigationPolicy,
};
use crate::control::{
    ControlCommandError, ControlCommandRequest, ControlCommandResponse, ControlService,
    NavigationStart,
};
use crate::display_session::{DisplaySessionError, DisplaySessionService};
use crate::playback::{Command, CommandError, NavigationTicket};
use crate::{Binding, EgressPolicy, EgressScope, GatewayError, GatewayService};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use site_adapter_api::{
    AdapterError, MediaProtection, NavigationDirection, ResolveContext, ResolvedMedia,
    SiteAdapterRegistry, StreamProtocol,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use uuid::Uuid;

const MAX_REQUEST_ID_BYTES: usize = 128;
const MAX_SOURCE_BYTES: usize = 4096;
const MAX_STREAM_ID_BYTES: usize = 128;
const MAX_CREATION_RECORDS: usize = 1024;
const MEDIA_CAPABILITY_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSessionRequest {
    pub request_id: String,
    pub source: String,
    pub display_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SessionMediaStream {
    pub id: String,
    pub protocol: String,
    pub gateway_path: String,
    pub kind: String,
    pub group_id: Option<String>,
    pub codec: Option<String>,
    pub container: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SessionMediaView {
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub media_generation: u64,
    pub title: String,
    pub source_site: String,
    pub streams: Vec<SessionMediaStream>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CreateSessionResponse {
    pub request_id: String,
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub session_revision: u64,
    pub display_id: String,
    pub source_site: String,
    pub media: SessionMediaView,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CreateSessionErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CreationFingerprint {
    source: String,
    display_id: String,
}

#[derive(Clone, Debug)]
pub(crate) enum CreationOutcome {
    Success(Box<CreateSessionResponse>),
    Failure {
        status: axum::http::StatusCode,
        error: CreateSessionErrorResponse,
    },
}

impl CreationOutcome {
    pub(crate) fn into_response(self) -> Response {
        match self {
            Self::Success(response) => axum::Json(response).into_response(),
            Self::Failure { status, error } => (status, axum::Json(error)).into_response(),
        }
    }
}

#[derive(Clone, Debug)]
struct CreationRecord {
    fingerprint: CreationFingerprint,
    outcome: CreationOutcome,
}

#[derive(Clone)]
pub(crate) struct SourceSessionService {
    registry: Arc<SiteAdapterRegistry>,
    creations: Arc<Mutex<HashMap<String, CreationRecord>>>,
    media_views: Arc<RwLock<HashMap<String, SessionMediaView>>>,
}

impl SourceSessionService {
    pub(crate) fn new(registry: Arc<SiteAdapterRegistry>) -> Self {
        Self {
            registry,
            creations: Arc::new(Mutex::new(HashMap::new())),
            media_views: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) fn media_for_snapshot(
        &self,
        snapshot: &crate::ControlSnapshot,
    ) -> Option<SessionMediaView> {
        let media = self
            .media_views
            .read()
            .expect("source media views poisoned")
            .get(&snapshot.session_id)
            .cloned()?;
        (media.item_id == snapshot.current_item.item_id
            && media.item_revision == snapshot.current_item.item_revision
            && media.media_generation == snapshot.current_item.media_generation)
            .then_some(media)
    }

    pub(crate) fn recognize(
        &self,
        source: &str,
    ) -> Result<site_adapter_api::SourceLocator, AdapterError> {
        self.registry.recognize(source)
    }

    /// Publish a media projection without allowing a delayed preparation to
    /// roll the projection back after a newer Playback item has committed.
    ///
    /// Playback commits and this projection live behind separate authorities,
    /// so publication must be monotonic as well as protected by the map lock.
    /// `item_revision` is the per-session authority for item transitions;
    /// equal revisions are never replaced by a later navigation publication.
    fn publish_media_view(&self, view: SessionMediaView) {
        let mut media_views = self
            .media_views
            .write()
            .expect("source media views poisoned");
        if media_views
            .get(&view.session_id)
            .is_some_and(|current| current.item_revision >= view.item_revision)
        {
            return;
        }
        media_views.insert(view.session_id.clone(), view);
    }

    pub(crate) fn create(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: CreateSessionRequest,
    ) -> CreationOutcome {
        self.create_with_context(
            gateway,
            control,
            displays,
            request,
            ResolveContext::default(),
        )
    }

    /// Resolve a public source through the generic Browser Worker before
    /// entering the normal SourceSession publication path.  The worker owns
    /// the observation and server-side media ledger; HTTP callers provide only
    /// the opaque source selector and display identity.  Adapters without a
    /// browser navigation contract retain the ordinary direct-resolution path.
    pub(crate) async fn create_public_browser<W: BrowserWorker + 'static>(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: CreateSessionRequest,
        worker: &W,
        egress: EgressPolicy,
    ) -> CreationOutcome {
        if let Err(error) = validate_request(&request) {
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::BAD_REQUEST,
                error,
            };
        }

        let locator = match self.registry.recognize(&request.source) {
            Ok(locator) => locator,
            Err(error) => return failure_for_adapter(error),
        };
        // Keep direct adapters on their existing path.  A browser is needed
        // only when the owning adapter explicitly reports that observation is
        // required; Core does not interpret any site-specific URL semantics.
        match self.registry.resolve(&locator) {
            Ok(_) => return self.create(gateway, control, displays, request),
            Err(AdapterError::ObservationRequired) => {}
            Err(error) => return failure_for_adapter(error),
        }
        let acquisition_target = match self.registry.browser_acquisition_target(&locator) {
            Ok(Some(target)) => target,
            Ok(None) | Err(AdapterError::UnsupportedAcquisition) => {
                return public_browser_failure("SOURCE_BROWSER_ACQUISITION_UNAVAILABLE")
            }
            Err(_) => return public_browser_failure("SOURCE_BROWSER_ACQUISITION_INVALID"),
        };

        let session = match worker.open_session(crate::browser::BrowserAuthMode::Passive).await {
            Ok(session) => session,
            Err(_) => return public_browser_failure("SOURCE_BROWSER_UNAVAILABLE"),
        };
        // The owning plugin supplied this target. Core does not parse or
        // reproduce site URL/BVID/part semantics; R008 authorizes the target
        // before the generic worker consumes it.
        let operation = BrowserNavigationRequest::new(acquisition_target.url().clone());
        drop(acquisition_target);
        let operation_id = operation.operation_id();
        let result = worker
            .navigate(
                session.id(),
                operation,
                &R008NavigationPolicy::public_web(egress),
            )
            .await;
        if result.is_err() {
            let _ = worker.close(session.id());
            return public_browser_failure("SOURCE_BROWSER_NAVIGATION_FAILED");
        }

        let handoff = BrowserObservationHandoff::take_from_worker(
            worker,
            session.id(),
            operation_id,
            locator,
            crate::browser::BROWSER_HANDOFF_TTL,
        );
        let outcome = match handoff {
            Ok(Some(handoff)) => self.create_with_context(
                gateway,
                control,
                displays,
                request,
                ResolveContext {
                    browser_observation: Some(handoff.observation()),
                    server_observation: Some(handoff.server_observation()),
                    authenticated_session: None,
                },
            ),
            Ok(None) | Err(_) => public_browser_failure("SOURCE_BROWSER_OBSERVATION_UNAVAILABLE"),
        };
        let _ = worker.close(session.id());
        outcome
    }

    /// Server-side acquisition handoff used by the browser/plugin integration.
    /// The HTTP request surface does not accept this context; callers must
    /// obtain it from the bounded Browser Worker and server-owned capability
    /// path before entering SourceSession.
    pub(crate) fn create_with_context(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: CreateSessionRequest,
        context: ResolveContext<'_>,
    ) -> CreationOutcome {
        if let Err(error) = validate_request(&request) {
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::BAD_REQUEST,
                error,
            };
        }

        let fingerprint = CreationFingerprint {
            source: request.source.clone(),
            display_id: request.display_id.clone(),
        };
        let mut creations = self.creations.lock().expect("source creations poisoned");
        if let Some(record) = creations.get(&request.request_id) {
            if record.fingerprint == fingerprint {
                return record.outcome.clone();
            }
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::CONFLICT,
                error: CreateSessionErrorResponse {
                    code: "CREATE_REQUEST_ID_MISMATCH",
                    message: "request_id was reused with a different creation input",
                },
            };
        }
        if creations.len() >= MAX_CREATION_RECORDS {
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::SERVICE_UNAVAILABLE,
                error: CreateSessionErrorResponse {
                    code: "CREATE_REQUEST_STORE_FULL",
                    message: "session creation idempotency capacity is temporarily full",
                },
            };
        }

        let outcome = self.create_fresh(gateway, control, displays, &request, context);
        creations.insert(
            request.request_id,
            CreationRecord {
                fingerprint,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    /// Consume a server-owned authenticated browser handoff through the
    /// normal source/session publication path. The Browser Worker facts and
    /// opaque auth proof are passed to the owning SiteAdapterRegistry; this
    /// method never receives or projects session material.
    pub(crate) fn create_authenticated(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: CreateSessionRequest,
        browser_handoff: BrowserObservationHandoff,
        authenticated_session: site_adapter_api::AuthenticatedSessionHandoff,
    ) -> CreationOutcome {
        if browser_handoff.is_expired() {
            return authenticated_failure(
                axum::http::StatusCode::CONFLICT,
                "SOURCE_OBSERVATION_EXPIRED",
                "authenticated browser observation has expired",
            );
        }
        if browser_handoff.locator().site_id.as_str() != authenticated_session.site_id() {
            return authenticated_failure(
                axum::http::StatusCode::CONFLICT,
                "SOURCE_AUTH_STALE",
                "authenticated browser handoff is stale for this source",
            );
        }

        match self.recognize(&request.source) {
            Ok(locator) if locator == *browser_handoff.locator() => {}
            Ok(_) | Err(_) => {
                return authenticated_failure(
                    axum::http::StatusCode::CONFLICT,
                    "SOURCE_AUTH_STALE",
                    "authenticated browser handoff is stale for this source",
                );
            }
        }

        self.create_with_context(
            gateway,
            control,
            displays,
            request,
            ResolveContext {
                browser_observation: Some(browser_handoff.observation()),
                server_observation: Some(browser_handoff.server_observation()),
                authenticated_session: Some(&authenticated_session),
            },
        )
    }

    fn create_fresh(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: &CreateSessionRequest,
        context: ResolveContext<'_>,
    ) -> CreationOutcome {
        if let Err(error) = displays.validate_live_selector(&request.display_id) {
            return failure_for_display(error);
        }

        let locator = match self.registry.recognize(&request.source) {
            Ok(locator) => locator,
            Err(error) => return failure_for_adapter(error),
        };
        let media = match self.registry.resolve_with_context(&locator, context) {
            Ok(media) => media,
            Err(error) => return failure_for_adapter(error),
        };
        if let Err(error) = validate_media(&media) {
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                error,
            };
        }
        if media.streams.len() > gateway.max_capabilities() {
            return CreationOutcome::Failure {
                status: axum::http::StatusCode::SERVICE_UNAVAILABLE,
                error: CreateSessionErrorResponse {
                    code: "MEDIA_CAPABILITY_LIMIT",
                    message: "resolved media requires more Gateway capabilities than available",
                },
            };
        }

        let session_id = format!("s-{}", Uuid::new_v4().simple());
        let item_id = format!("i-{}", Uuid::new_v4().simple());
        let item_revision = 1;
        let mut issued_tokens = Vec::with_capacity(media.streams.len());
        let mut streams = Vec::with_capacity(media.streams.len());
        for stream in &media.streams {
            let resource =
                match GatewayService::resource_from_resolved(stream, EgressScope::PublicWeb) {
                    Ok(resource) => resource,
                    Err(error) => {
                        revoke_all(gateway, &issued_tokens);
                        return failure_for_gateway(error);
                    }
                };
            let binding = Binding::new(&session_id, &item_id, item_revision, &stream.id);
            let (gateway_path, token) =
                gateway.issue_path_with_token(binding, resource, MEDIA_CAPABILITY_TTL);
            issued_tokens.push(token);
            streams.push(SessionMediaStream {
                id: stream.id.clone(),
                protocol: protocol_name(stream.protocol).into(),
                gateway_path,
                kind: media
                    .shape
                    .track(&stream.id)
                    .map(|track| track.kind.as_str().into())
                    .unwrap_or_else(|| "muxed".into()),
                group_id: media
                    .shape
                    .track(&stream.id)
                    .and_then(|track| track.group_id.clone()),
                codec: media
                    .shape
                    .track(&stream.id)
                    .and_then(|track| track.codec.clone()),
                container: media
                    .shape
                    .track(&stream.id)
                    .and_then(|track| track.container.clone()),
            });
        }

        let media_view = SessionMediaView {
            session_id: session_id.clone(),
            item_id: item_id.clone(),
            item_revision,
            media_generation: 0,
            title: sanitize_metadata(&media.title, 256),
            source_site: sanitize_metadata(&media.source_site, 128),
            streams,
        };
        let descriptor = match serde_json::to_string(&media_view) {
            Ok(descriptor) => descriptor,
            Err(_) => {
                revoke_all(gateway, &issued_tokens);
                return internal_failure();
            }
        };

        let snapshot = match control.publish_prepared_session(
            session_id.clone(),
            item_id.clone(),
            descriptor,
            request.display_id.clone(),
            locator.clone(),
        ) {
            Ok(snapshot) => snapshot,
            Err(_) => {
                revoke_all(gateway, &issued_tokens);
                return internal_failure();
            }
        };
        self.publish_media_view(media_view.clone());
        if displays
            .set_current_rendering_session(&request.display_id, &session_id)
            .is_err()
        {
            // The display selector was validated before resolution and the
            // relationship is bounded server-owned integration state. A
            // validation failure here cannot undo the already accepted #44
            // publication, so leave the session authoritative and let the
            // normal reconnect lookup fail closed until the display is live.
        }
        CreationOutcome::Success(Box::new(CreateSessionResponse {
            request_id: request.request_id.clone(),
            session_id,
            item_id,
            item_revision,
            session_revision: snapshot.session_revision,
            display_id: request.display_id.clone(),
            source_site: media_view.source_site.clone(),
            media: media_view,
        }))
    }

    pub(crate) fn navigate(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        session_id: &str,
        request: ControlCommandRequest,
    ) -> Result<ControlCommandResponse, ControlCommandError> {
        let start = control.begin_navigation(session_id, request)?;
        let NavigationStart::Prepare { envelope, snapshot } = start else {
            let NavigationStart::Replay(outcome) = start else {
                unreachable!("navigation start must be prepare or replay")
            };
            return control.replay_navigation(session_id, outcome);
        };
        let direction = match &envelope.command {
            Command::NextItem => NavigationDirection::Next,
            Command::PreviousItem => NavigationDirection::Previous,
            _ => {
                return Err(ControlCommandError::Playback(
                    CommandError::NavigationUnsupported,
                ));
            }
        };
        let context = match self.registry.navigation(&snapshot.source_locator) {
            Ok(context) => context,
            Err(AdapterError::UnsupportedNavigation) => {
                return control.remember_navigation_failure(
                    session_id,
                    &envelope,
                    CommandError::NavigationUnsupported,
                );
            }
            Err(_) => {
                return control.remember_navigation_failure(
                    session_id,
                    &envelope,
                    CommandError::NavigationPreparationFailed,
                );
            }
        };
        let Some(target_locator) = direction.select(&context).cloned() else {
            return control.remember_navigation_failure(
                session_id,
                &envelope,
                CommandError::NavigationNoTarget,
            );
        };
        let media = match self.registry.resolve(&target_locator) {
            Ok(media) => media,
            Err(_) => {
                return control.remember_navigation_failure(
                    session_id,
                    &envelope,
                    CommandError::NavigationPreparationFailed,
                );
            }
        };
        if validate_media(&media).is_err() || media.streams.len() > gateway.max_capabilities() {
            return control.remember_navigation_failure(
                session_id,
                &envelope,
                CommandError::NavigationPreparationFailed,
            );
        }

        let item_id = format!("i-{}", Uuid::new_v4().simple());
        let prepared = match prepare_media(
            gateway,
            &media,
            session_id,
            &item_id,
            snapshot.item_revision.saturating_add(1),
        ) {
            Ok(prepared) => prepared,
            Err((error, tokens)) => {
                revoke_all(gateway, &tokens);
                return control.remember_navigation_failure(session_id, &envelope, error);
            }
        };
        let ticket = NavigationTicket {
            session_id: snapshot.session_id.clone(),
            expected_session_revision: snapshot.session_revision,
            expected_item_id: snapshot.item_id,
            expected_item_revision: snapshot.item_revision,
            direction,
            target_locator,
        };
        let result = control.commit_prepared_navigation(
            session_id,
            envelope,
            &ticket,
            item_id,
            prepared.descriptor.clone(),
        );
        match result {
            Ok(result) => {
                self.publish_media_view(prepared.view);
                Ok(result)
            }
            Err(error) => {
                revoke_all(gateway, &prepared.tokens);
                Err(error)
            }
        }
    }
}

struct PreparedMedia {
    view: SessionMediaView,
    descriptor: String,
    tokens: Vec<String>,
}

fn prepare_media(
    gateway: &GatewayService,
    media: &ResolvedMedia,
    session_id: &str,
    item_id: &str,
    item_revision: u64,
) -> Result<PreparedMedia, (CommandError, Vec<String>)> {
    let mut issued_tokens = Vec::with_capacity(media.streams.len());
    let mut streams = Vec::with_capacity(media.streams.len());
    for stream in &media.streams {
        let resource = match GatewayService::resource_from_resolved(stream, EgressScope::PublicWeb)
        {
            Ok(resource) => resource,
            Err(_) => return Err((CommandError::NavigationPreparationFailed, issued_tokens)),
        };
        let binding = Binding::new(session_id, item_id, item_revision, &stream.id);
        let (gateway_path, token) =
            gateway.issue_path_with_token(binding, resource, MEDIA_CAPABILITY_TTL);
        issued_tokens.push(token);
        streams.push(SessionMediaStream {
            id: stream.id.clone(),
            protocol: protocol_name(stream.protocol).into(),
            gateway_path,
            kind: media
                .shape
                .track(&stream.id)
                .map(|track| track.kind.as_str().into())
                .unwrap_or_else(|| "muxed".into()),
            group_id: media
                .shape
                .track(&stream.id)
                .and_then(|track| track.group_id.clone()),
            codec: media
                .shape
                .track(&stream.id)
                .and_then(|track| track.codec.clone()),
            container: media
                .shape
                .track(&stream.id)
                .and_then(|track| track.container.clone()),
        });
    }
    let view = SessionMediaView {
        session_id: session_id.into(),
        item_id: item_id.into(),
        item_revision,
        media_generation: 0,
        title: sanitize_metadata(&media.title, 256),
        source_site: sanitize_metadata(&media.source_site, 128),
        streams,
    };
    let descriptor = match serde_json::to_string(&view) {
        Ok(descriptor) => descriptor,
        Err(_) => return Err((CommandError::NavigationPreparationFailed, issued_tokens)),
    };
    Ok(PreparedMedia {
        view,
        descriptor,
        tokens: issued_tokens,
    })
}

fn validate_request(request: &CreateSessionRequest) -> Result<(), CreateSessionErrorResponse> {
    validate_identifier(&request.request_id, MAX_REQUEST_ID_BYTES, "REQUEST_ID")?;
    if request.source.is_empty() || request.source.len() > MAX_SOURCE_BYTES {
        return Err(CreateSessionErrorResponse {
            code: "SOURCE_INVALID",
            message: "source is empty or exceeds the maximum length",
        });
    }
    if request.source.chars().any(char::is_control) {
        return Err(CreateSessionErrorResponse {
            code: "SOURCE_INVALID",
            message: "source contains unsupported control characters",
        });
    }
    validate_identifier(&request.display_id, 128, "DISPLAY_ID")
}

fn validate_identifier(
    value: &str,
    max_bytes: usize,
    field: &'static str,
) -> Result<(), CreateSessionErrorResponse> {
    if value.is_empty() {
        return Err(CreateSessionErrorResponse {
            code: match field {
                "REQUEST_ID" => "REQUEST_ID_INVALID",
                "DISPLAY_ID" => "DISPLAY_ID_INVALID",
                _ => "IDENTIFIER_INVALID",
            },
            message: "identifier is empty or contains unsupported characters",
        });
    }
    if value.len() > max_bytes {
        return Err(CreateSessionErrorResponse {
            code: match field {
                "REQUEST_ID" => "REQUEST_ID_TOO_LONG",
                "DISPLAY_ID" => "DISPLAY_ID_TOO_LONG",
                _ => "IDENTIFIER_TOO_LONG",
            },
            message: "identifier exceeds the maximum length",
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
    {
        return Err(CreateSessionErrorResponse {
            code: match field {
                "REQUEST_ID" => "REQUEST_ID_INVALID",
                "DISPLAY_ID" => "DISPLAY_ID_INVALID",
                _ => "IDENTIFIER_INVALID",
            },
            message: "identifier contains unsupported characters",
        });
    }
    Ok(())
}

fn validate_media(media: &ResolvedMedia) -> Result<(), CreateSessionErrorResponse> {
    if media.protection != MediaProtection::Clear || media.streams.is_empty() {
        return Err(CreateSessionErrorResponse {
            code: "MEDIA_UNSUPPORTED",
            message: "resolved media is not a supported clear stream",
        });
    }
    if site_adapter_api::conformance::validate_media_shape(&media.shape, &media.streams).is_err() {
        return Err(CreateSessionErrorResponse {
            code: "MEDIA_INVALID",
            message: "resolved media shape is invalid or mismatched",
        });
    }
    for stream in &media.streams {
        if stream.upstream_access_ref.is_some()
            || stream.id.is_empty()
            || stream.id.len() > MAX_STREAM_ID_BYTES
            || !stream
                .id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
        {
            return Err(CreateSessionErrorResponse {
                code: "MEDIA_INVALID",
                message: "resolved media contains an unsafe stream descriptor",
            });
        }
    }
    Ok(())
}

fn protocol_name(protocol: StreamProtocol) -> &'static str {
    match protocol {
        StreamProtocol::HttpFile => "http_file",
        StreamProtocol::Hls => "hls",
        StreamProtocol::Dash => "dash",
    }
}

fn sanitize_metadata(value: &str, max_bytes: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(max_bytes)
        .collect()
}

fn revoke_all(gateway: &GatewayService, tokens: &[String]) {
    for token in tokens {
        gateway.revoke_capability(token);
    }
}

fn failure_for_display(error: DisplaySessionError) -> CreationOutcome {
    let (status, code, message) = match error {
        DisplaySessionError::InvalidIdentifier(_) => (
            axum::http::StatusCode::BAD_REQUEST,
            "DISPLAY_ID_INVALID",
            "display_id is invalid",
        ),
        DisplaySessionError::LeaseExpired => (
            axum::http::StatusCode::CONFLICT,
            "DISPLAY_OFFLINE",
            "display registration is offline",
        ),
        DisplaySessionError::RegistrationNotFound => (
            axum::http::StatusCode::NOT_FOUND,
            "DISPLAY_NOT_FOUND",
            "display registration was not found",
        ),
        _ => (
            axum::http::StatusCode::CONFLICT,
            "DISPLAY_UNAVAILABLE",
            "display registration is unavailable",
        ),
    };
    CreationOutcome::Failure {
        status,
        error: CreateSessionErrorResponse { code, message },
    }
}

fn failure_for_adapter(error: AdapterError) -> CreationOutcome {
    let (status, code, message) = match error {
        AdapterError::NoMatch => (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "SOURCE_NOT_RECOGNIZED",
            "source was not recognized by a registered adapter",
        ),
        AdapterError::AmbiguousMatch => (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "SOURCE_AMBIGUOUS",
            "source matched multiple registered adapters",
        ),
        AdapterError::ObservationExpired => (
            axum::http::StatusCode::CONFLICT,
            "SOURCE_OBSERVATION_EXPIRED",
            "authenticated browser observation has expired",
        ),
        AdapterError::ContentNotFound => (
            axum::http::StatusCode::CONFLICT,
            "SOURCE_AUTH_STALE",
            "authenticated browser observation is stale for this source",
        ),
        _ => (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "SOURCE_UNSUPPORTED",
            "registered adapter could not prepare the source",
        ),
    };
    CreationOutcome::Failure {
        status,
        error: CreateSessionErrorResponse { code, message },
    }
}

fn public_browser_failure(code: &'static str) -> CreationOutcome {
    CreationOutcome::Failure {
        status: axum::http::StatusCode::BAD_GATEWAY,
        error: CreateSessionErrorResponse {
            code,
            message: "public source browser acquisition did not produce a usable observation",
        },
    }
}

fn authenticated_failure(
    status: axum::http::StatusCode,
    code: &'static str,
    message: &'static str,
) -> CreationOutcome {
    CreationOutcome::Failure {
        status,
        error: CreateSessionErrorResponse { code, message },
    }
}

fn failure_for_gateway(error: GatewayError) -> CreationOutcome {
    let code = match error {
        GatewayError::SecretHeader => "MEDIA_SECRET_REJECTED",
        GatewayError::InvalidHeader
        | GatewayError::InvalidSubtitle
        | GatewayError::UnsupportedSubtitleContentType => "MEDIA_INVALID",
    };
    CreationOutcome::Failure {
        status: axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        error: CreateSessionErrorResponse {
            code,
            message: "resolved media cannot be exposed through a Gateway capability",
        },
    }
}

fn internal_failure() -> CreationOutcome {
    CreationOutcome::Failure {
        status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error: CreateSessionErrorResponse {
            code: "SESSION_CREATE_FAILED",
            message: "session creation failed before publication",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionMediaView, SourceSessionService};
    use crate::GatewayService;
    use crate::browser::{
        BrowserAuthMode, BrowserObservationPayload, BrowserWorker, FakeBrowserWorker,
    };
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use generic_direct::GenericDirectAdapter;
    use site_adapter_api::{
        AdapterError, BROWSER_OBSERVATION_VERSION, BrowserExpiryHint, BrowserMediaCandidate,
        BrowserMediaKind, BrowserObservation, BrowserRangeSupport, BrowserStatusClass,
        MediaProtection, RecognizeResult, ResolveContext, ResolvedMedia, ResolvedStream,
        ServerOwnedMedia, ServerOwnedObservation, SiteAdapter, SiteAdapterRegistry, SourceLocator,
        StreamProtocol,
    };
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use tower::ServiceExt;
    use url::Url;

    const HOST: &str = "127.0.0.1:8787";
    const ORIGIN: &str = "http://127.0.0.1:8787";
    const SOURCE: &str = "https://example.test/video.mp4";

    #[derive(Clone, Copy)]
    enum FixtureMode {
        InvalidRecognition,
        Ambiguous,
        Rollback,
        SecretReference,
        Navigation,
        Observation,
        AuthenticatedObservation,
    }

    struct FixtureAdapter {
        plugin: &'static str,
        priority: u16,
        mode: FixtureMode,
    }

    impl SiteAdapter for FixtureAdapter {
        fn site_id(&self) -> &'static str {
            "fixture"
        }

        fn plugin_id(&self) -> &'static str {
            self.plugin
        }

        fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
            let matched = input.starts_with("fixture://")
                || (matches!(self.mode, FixtureMode::Observation)
                    && input.starts_with("https://www.example.com/fixture"));
            if !matched {
                return Ok(RecognizeResult {
                    matched: false,
                    site_id: self.site_id().into(),
                    plugin_id: self.plugin_id().into(),
                    priority: self.priority,
                    locator: None,
                });
            }
            let locator = match self.mode {
                FixtureMode::InvalidRecognition => SourceLocator {
                    site_id: self.site_id().into(),
                    plugin_id: "foreign-plugin".into(),
                    locator_version: 1,
                    opaque_payload: input.into(),
                },
                _ => SourceLocator {
                    site_id: self.site_id().into(),
                    plugin_id: self.plugin_id().into(),
                    locator_version: 1,
                    opaque_payload: input.into(),
                },
            };
            Ok(RecognizeResult {
                matched: true,
                site_id: self.site_id().into(),
                plugin_id: self.plugin_id().into(),
                priority: self.priority,
                locator: Some(locator),
            })
        }

        fn resolve(&self, _locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
            if matches!(
                self.mode,
                FixtureMode::Observation | FixtureMode::AuthenticatedObservation
            ) {
                return Err(AdapterError::ObservationRequired);
            }
            fixture_resolved_media(self)
        }

        fn resolve_with_context(
            &self,
            _locator: &SourceLocator,
            context: ResolveContext<'_>,
        ) -> Result<ResolvedMedia, AdapterError> {
            if matches!(
                self.mode,
                FixtureMode::Observation | FixtureMode::AuthenticatedObservation
            ) && (context.browser_observation.is_none() || context.server_observation.is_none())
            {
                return Err(AdapterError::ObservationRequired);
            }
            if matches!(self.mode, FixtureMode::AuthenticatedObservation)
                && context.authenticated_session.is_none()
            {
                return Err(AdapterError::AccessRequired);
            }
            fixture_resolved_media(self)
        }

        fn browser_acquisition_target(
            &self,
            _locator: &SourceLocator,
        ) -> Result<Option<site_adapter_api::BrowserAcquisitionTarget>, AdapterError> {
            if matches!(self.mode, FixtureMode::Observation) {
                return Ok(Some(
                    site_adapter_api::BrowserAcquisitionTarget::new(
                        Url::parse("https://www.example.com/fixture").unwrap(),
                    )
                    .unwrap(),
                ));
            }
            Ok(None)
        }

        fn navigation(
            &self,
            locator: &SourceLocator,
        ) -> Result<site_adapter_api::NavigationContext, AdapterError> {
            if !matches!(self.mode, FixtureMode::Navigation) {
                return Err(AdapterError::UnsupportedNavigation);
            }
            let make_locator = |payload: &str| SourceLocator {
                site_id: self.site_id().into(),
                plugin_id: self.plugin_id().into(),
                locator_version: 1,
                opaque_payload: payload.into(),
            };
            Ok(site_adapter_api::NavigationContext {
                previous: (locator.opaque_payload != "fixture://start")
                    .then(|| make_locator("fixture://previous")),
                next: (locator.opaque_payload == "fixture://start"
                    || locator.opaque_payload == "fixture://middle")
                    .then(|| make_locator("fixture://end")),
                collection_id: Some("fixture-collection".into()),
                current_index: Some(1),
            })
        }
    }

    fn fixture_resolved_media(adapter: &FixtureAdapter) -> Result<ResolvedMedia, AdapterError> {
        let stream = |id: &str, headers: BTreeMap<String, String>| ResolvedStream {
            id: id.into(),
            protocol: StreamProtocol::HttpFile,
            url: Url::parse("https://example.test/fixture.mp4").unwrap(),
            public_headers: headers,
            upstream_access_ref: None,
        };
        let streams = match adapter.mode {
            FixtureMode::Rollback => vec![
                stream("primary", BTreeMap::new()),
                stream(
                    "broken",
                    BTreeMap::from([("invalid header".into(), "value".into())]),
                ),
            ],
            FixtureMode::SecretReference => vec![ResolvedStream {
                upstream_access_ref: Some("fixture-secret-ref".into()),
                ..stream("primary", BTreeMap::new())
            }],
            _ => vec![stream("primary", BTreeMap::new())],
        };
        Ok(ResolvedMedia::legacy(
            "fixture media",
            adapter.site_id(),
            streams,
            vec![],
            MediaProtection::Clear,
        ))
    }

    struct TraceAdapter {
        inner: FixtureAdapter,
        seen: Arc<Mutex<Vec<SourceLocator>>>,
    }

    impl SiteAdapter for TraceAdapter {
        fn site_id(&self) -> &'static str {
            self.inner.site_id()
        }

        fn plugin_id(&self) -> &'static str {
            self.inner.plugin_id()
        }

        fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
            self.inner.recognize(input)
        }

        fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
            self.inner.resolve(locator)
        }

        fn resolve_with_context(
            &self,
            locator: &SourceLocator,
            context: ResolveContext<'_>,
        ) -> Result<ResolvedMedia, AdapterError> {
            self.seen.lock().unwrap().push(locator.clone());
            self.inner.resolve_with_context(locator, context)
        }

        fn browser_acquisition_target(
            &self,
            locator: &SourceLocator,
        ) -> Result<Option<site_adapter_api::BrowserAcquisitionTarget>, AdapterError> {
            self.seen.lock().unwrap().push(locator.clone());
            self.inner.browser_acquisition_target(locator)
        }
    }

    fn service() -> GatewayService {
        let mut registry = SiteAdapterRegistry::default();
        registry.register(Arc::new(GenericDirectAdapter)).unwrap();
        service_with_registry(registry)
    }

    fn service_with_registry(registry: SiteAdapterRegistry) -> GatewayService {
        let service = GatewayService::with_registry(8, Arc::new(registry));
        service
            .configure_http_authority(Url::parse(ORIGIN).unwrap())
            .unwrap();
        service
    }

    fn observation_context() -> (BrowserObservation, ServerOwnedObservation) {
        let observation = BrowserObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "source-session-observation".into(),
            page_url: "https://example.test/fixture".into(),
            page_title: "Source Session Observation Fixture".into(),
            event_count: 2,
            resource_count: 1,
            candidates: vec![BrowserMediaCandidate {
                id: "primary".into(),
                kind: BrowserMediaKind::Muxed,
                group_id: None,
                codec: None,
                container: Some("mp4".into()),
                mime_type: Some("video/mp4".into()),
                width: None,
                height: None,
                bitrate: None,
                language: None,
                protocol: StreamProtocol::HttpFile,
                status: BrowserStatusClass::Success,
                range: BrowserRangeSupport::Supported,
                egress_allowed: true,
                access_ref: "source-session-ref".into(),
                expiry: BrowserExpiryHint::NoneObserved,
                expires_at: None,
            }],
        };
        let server = ServerOwnedObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "source-session-observation".into(),
            media: vec![ServerOwnedMedia {
                observation_id: "source-session-observation".into(),
                candidate_id: "primary".into(),
                access_ref: "source-session-ref".into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://media.example.invalid/fixture.mp4").unwrap(),
                public_headers: BTreeMap::new(),
            }],
        };
        (observation, server)
    }

    fn authenticated_browser_handoff(
        locator_payload: &str,
        ttl: std::time::Duration,
    ) -> crate::browser::BrowserObservationHandoff {
        let (observation, server_observation) = observation_context();
        crate::browser::BrowserObservationHandoff::bind(
            crate::browser::BrowserSessionId::new(),
            crate::browser::BrowserOperationId::from_value(248),
            SourceLocator {
                site_id: "fixture".into(),
                plugin_id: "fixture-authenticated".into(),
                locator_version: 1,
                opaque_payload: locator_payload.into(),
            },
            crate::browser::BrowserObservationPayload {
                operation_id: crate::browser::BrowserOperationId::from_value(248),
                observation,
                server_observation,
            },
            ttl,
        )
        .unwrap()
    }

    fn authenticated_session() -> site_adapter_api::AuthenticatedSessionHandoff {
        site_adapter_api::AuthenticatedSessionHandoff::new_server_owned(
            "fixture",
            "account-a",
            "candidate-ref",
            site_adapter_api::BrowserAuthObservation {
                schema_version: site_adapter_api::BROWSER_AUTH_OBSERVATION_VERSION,
                sequence: 1,
                state: site_adapter_api::BrowserAuthState::CandidateReady,
                diagnostic: site_adapter_api::BrowserAuthDiagnostic::CandidateAccepted,
            },
        )
        .unwrap()
    }

    async fn json(response: axum::response::Response) -> serde_json::Value {
        serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap()).unwrap()
    }

    fn post(path: &str, value: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(path)
            .header(header::HOST, HOST)
            .header(header::ORIGIN, ORIGIN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string()))
            .unwrap()
    }

    async fn register_display(service: &GatewayService) {
        let response = service
            .router()
            .oneshot(post(
                "/api/v1/displays/register",
                serde_json::json!({
                    "display_id": "display-a",
                    "label": "test display",
                    "capabilities": ["video"]
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn generic_source_creates_real_session_and_reuses_existing_control_surface() {
        let service = service();
        register_display(&service).await;
        let created = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "create-1",
                    "source": SOURCE,
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::OK);
        let body = json(created).await;
        let session_id = body["session_id"].as_str().unwrap();
        assert_eq!(body["item_revision"], 1);
        assert_eq!(body["session_revision"], 0);
        let stream_path = body["media"]["streams"][0]["gateway_path"]
            .as_str()
            .unwrap();
        assert!(stream_path.starts_with("/stream/"));
        let text = body.to_string();
        assert!(!text.contains(SOURCE));
        assert!(!text.contains("opaque_payload"));
        assert!(!text.contains("upstream_access_ref"));

        let snapshot = service
            .router()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/sessions/{session_id}"))
                    .header(header::HOST, HOST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(snapshot.status(), StatusCode::OK);
        let command = service
            .router()
            .oneshot(post(
                &format!("/api/v1/sessions/{session_id}/commands"),
                serde_json::json!({
                    "request_id": "play-1",
                    "expected_session_revision": 0,
                    "command": {"type": "play"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(command.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn public_browser_seam_consumes_server_owned_observation_before_publication() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-public-observation",
                priority: 100,
                mode: FixtureMode::Observation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;

        let worker = FakeBrowserWorker::new();
        let (observation, server_observation) = observation_context();
        let session = worker
            .open_session(BrowserAuthMode::Passive)
            .await
            .unwrap();
        worker
            .set_next_observation(
                session.id(),
                BrowserObservationPayload {
                    operation_id: crate::browser::BrowserOperationId::from_value(1),
                    observation,
                    server_observation,
                },
            )
            .unwrap();

        let outcome = service
            .create_public_session_with_worker(
                super::CreateSessionRequest {
                    request_id: "public-observation-1".into(),
                    source: "https://www.example.com/fixture".into(),
                    display_id: "display-a".into(),
                },
                &worker,
            )
            .await;
        let super::CreationOutcome::Success(response) = outcome else {
            panic!("public browser observation did not publish a session");
        };
        assert_eq!(response.source_site, "fixture");
        assert_eq!(response.media.streams.len(), 1);
    }

    #[tokio::test]
    async fn public_browser_reuses_same_locator_for_target_and_observation_resolution() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(TraceAdapter {
                inner: FixtureAdapter {
                    plugin: "fixture-trace",
                    priority: 100,
                    mode: FixtureMode::Observation,
                },
                seen: seen.clone(),
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let worker = FakeBrowserWorker::new();
        let (observation, server_observation) = observation_context();
        let session = worker
            .open_session(BrowserAuthMode::Passive)
            .await
            .unwrap();
        worker
            .set_next_observation(
                session.id(),
                BrowserObservationPayload {
                    operation_id: crate::browser::BrowserOperationId::from_value(1),
                    observation,
                    server_observation,
                },
            )
            .unwrap();

        let outcome = service
            .create_public_session_with_worker(
                super::CreateSessionRequest {
                    request_id: "public-same-locator".into(),
                    source: "https://www.example.com/fixture".into(),
                    display_id: "display-a".into(),
                },
                &worker,
            )
            .await;
        assert!(matches!(outcome, super::CreationOutcome::Success(_)));
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0], seen[1]);
        assert_eq!(seen[0].plugin_id, "fixture-trace");
    }

    #[tokio::test]
    async fn observation_required_without_plugin_target_fails_closed() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-auth-only",
                priority: 10,
                mode: FixtureMode::AuthenticatedObservation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let outcome = service
            .create_public_session_with_worker(
                super::CreateSessionRequest {
                    request_id: "public-no-target".into(),
                    source: "fixture://auth-only".into(),
                    display_id: "display-a".into(),
                },
                &FakeBrowserWorker::new(),
            )
            .await;
        let super::CreationOutcome::Failure { error, .. } = outcome else {
            panic!("observation without an acquisition target must fail closed");
        };
        assert_eq!(error.code, "SOURCE_BROWSER_ACQUISITION_UNAVAILABLE");
    }

    #[tokio::test]
    async fn browser_observation_handoff_enters_source_session_without_secret_projection() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-observation",
                priority: 10,
                mode: FixtureMode::Observation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let (observation, server_observation) = observation_context();
        let response = service
            .create_session_with_context(
                super::CreateSessionRequest {
                    request_id: "observation-fixture-create".into(),
                    source: "fixture://observation".into(),
                    display_id: "display-a".into(),
                },
                ResolveContext {
                    browser_observation: Some(&observation),
                    server_observation: Some(&server_observation),
                    authenticated_session: None,
                },
            )
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json(response).await;
        assert_eq!(body["source_site"], "fixture");
        assert!(body.to_string().contains("/stream/"));
        assert!(!body.to_string().contains("source-session-ref"));
        assert!(!body.to_string().contains("media.example.invalid"));
    }

    #[tokio::test]
    async fn authenticated_playback_seam_uses_context_and_is_exactly_once() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-authenticated",
                priority: 10,
                mode: FixtureMode::AuthenticatedObservation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let request = super::CreateSessionRequest {
            request_id: "authenticated-create".into(),
            source: "fixture://observation".into(),
            display_id: "display-a".into(),
        };
        let first = json(service.create_authenticated_playback_session(
            request.clone(),
            authenticated_browser_handoff(
                "fixture://observation",
                std::time::Duration::from_secs(30),
            ),
            authenticated_session(),
        ))
        .await;
        let capability_count = service.capability_count();
        let replay = json(service.create_authenticated_playback_session(
            request,
            authenticated_browser_handoff(
                "fixture://observation",
                std::time::Duration::from_secs(30),
            ),
            authenticated_session(),
        ))
        .await;

        assert_eq!(first["session_id"], replay["session_id"]);
        assert_eq!(first["media"]["streams"], replay["media"]["streams"]);
        assert_eq!(service.capability_count(), capability_count);
        assert_eq!(service.control().session_count(), 1);
        assert!(
            first["media"]["streams"][0]["gateway_path"]
                .as_str()
                .is_some_and(|path| path.starts_with("/stream/"))
        );
    }

    #[tokio::test]
    async fn authenticated_playback_seam_rejects_stale_handoff_without_publication() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-authenticated",
                priority: 10,
                mode: FixtureMode::AuthenticatedObservation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let response = service.create_authenticated_playback_session(
            super::CreateSessionRequest {
                request_id: "authenticated-stale".into(),
                source: "fixture://observation".into(),
                display_id: "display-a".into(),
            },
            authenticated_browser_handoff("fixture://other", std::time::Duration::from_secs(30)),
            authenticated_session(),
        );
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json(response).await;
        assert_eq!(body["code"], "SOURCE_AUTH_STALE");
        assert_eq!(service.control().session_count(), 0);
        assert_eq!(service.capability_count(), 0);
        assert!(!body.to_string().contains("candidate-ref"));
    }

    #[tokio::test]
    async fn authenticated_playback_seam_rejects_expired_observation() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-authenticated",
                priority: 10,
                mode: FixtureMode::AuthenticatedObservation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let response = service.create_authenticated_playback_session(
            super::CreateSessionRequest {
                request_id: "authenticated-expired".into(),
                source: "fixture://observation".into(),
                display_id: "display-a".into(),
            },
            authenticated_browser_handoff("fixture://observation", std::time::Duration::ZERO),
            authenticated_session(),
        );
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json(response).await;
        assert_eq!(body["code"], "SOURCE_OBSERVATION_EXPIRED");
        assert_eq!(service.control().session_count(), 0);
        assert_eq!(service.capability_count(), 0);
    }

    #[tokio::test]
    async fn authenticated_playback_errors_are_secret_safe() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-authenticated",
                priority: 10,
                mode: FixtureMode::AuthenticatedObservation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let response = service.create_authenticated_playback_session(
            super::CreateSessionRequest {
                request_id: "authenticated-secret-safe".into(),
                source: "fixture://observation".into(),
                display_id: "display-a".into(),
            },
            authenticated_browser_handoff("fixture://other", std::time::Duration::from_secs(30)),
            authenticated_session(),
        );
        let body = json(response).await;
        let serialized = body.to_string();
        assert!(!serialized.contains("candidate-ref"));
        assert!(!serialized.contains("source-session-ref"));
        assert!(!serialized.contains("media.example.invalid"));
        assert!(!serialized.contains("Bearer"));
        assert!(!serialized.contains("Cookie"));
    }

    #[tokio::test]
    async fn creation_idempotency_and_injection_boundaries_are_deterministic() {
        let service = service();
        register_display(&service).await;
        let request = serde_json::json!({
            "request_id": "create-1",
            "source": SOURCE,
            "display_id": "display-a"
        });
        let first = json(
            service
                .router()
                .oneshot(post("/api/v1/sessions", request.clone()))
                .await
                .unwrap(),
        )
        .await;
        let first_count = service.capability_count();
        let replay = json(
            service
                .router()
                .oneshot(post("/api/v1/sessions", request))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(first, replay);
        assert_eq!(service.capability_count(), first_count);

        let mismatch = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "create-1",
                    "source": "https://example.test/other.mp4",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(mismatch.status(), StatusCode::CONFLICT);
        assert_eq!(json(mismatch).await["code"], "CREATE_REQUEST_ID_MISMATCH");

        let injected = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "create-2",
                    "source": SOURCE,
                    "display_id": "display-a",
                    "resolved_media": {"url": "https://secret.invalid/raw"},
                    "lease_token": "lease-secret",
                    "display_generation": 99
                }),
            ))
            .await
            .unwrap();
        assert_eq!(injected.status(), StatusCode::BAD_REQUEST);
        assert_eq!(service.capability_count(), first_count);
    }

    #[tokio::test]
    async fn publication_failure_revokes_prepared_capabilities_and_publishes_nothing() {
        let service = service();
        register_display(&service).await;
        service.control().fail_next_publication();
        let failed = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "rollback-1",
                    "source": SOURCE,
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json(failed).await["code"], "SESSION_CREATE_FAILED");
        assert_eq!(service.capability_count(), 0);
        assert_eq!(service.control().session_count(), 0);
    }

    #[tokio::test]
    async fn invalid_display_and_source_are_rejected_without_publication() {
        let service = service();
        let missing_display = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "missing-display",
                    "source": SOURCE,
                    "display_id": "not-registered"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(missing_display.status(), StatusCode::NOT_FOUND);
        assert_eq!(service.control().session_count(), 0);
        assert_eq!(service.capability_count(), 0);

        register_display(&service).await;
        let no_match = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "no-match",
                    "source": "https://example.test/page",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(no_match.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json(no_match).await["code"], "SOURCE_NOT_RECOGNIZED");
        assert_eq!(service.control().session_count(), 0);
        assert_eq!(service.capability_count(), 0);
    }

    #[tokio::test]
    async fn registry_ambiguity_and_invalid_adapter_output_are_not_bypassed() {
        let mut ambiguous_registry = SiteAdapterRegistry::default();
        ambiguous_registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-a",
                priority: 10,
                mode: FixtureMode::Ambiguous,
            }))
            .unwrap();
        ambiguous_registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-b",
                priority: 10,
                mode: FixtureMode::Ambiguous,
            }))
            .unwrap();
        let service = service_with_registry(ambiguous_registry);
        register_display(&service).await;
        let ambiguous = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "ambiguous",
                    "source": "fixture://ambiguous",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(ambiguous.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json(ambiguous).await["code"], "SOURCE_AMBIGUOUS");

        let mut invalid_registry = SiteAdapterRegistry::default();
        invalid_registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-invalid",
                priority: 10,
                mode: FixtureMode::InvalidRecognition,
            }))
            .unwrap();
        let service = service_with_registry(invalid_registry);
        register_display(&service).await;
        let invalid = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "invalid-output",
                    "source": "fixture://invalid",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json(invalid).await["code"], "SOURCE_UNSUPPORTED");
        assert_eq!(service.control().session_count(), 0);
        assert_eq!(service.capability_count(), 0);
    }

    #[tokio::test]
    async fn preparation_failure_revokes_already_issued_capabilities() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-rollback",
                priority: 10,
                mode: FixtureMode::Rollback,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let failed = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "prepare-rollback",
                    "source": "fixture://rollback",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json(failed).await["code"], "MEDIA_INVALID");
        assert_eq!(service.capability_count(), 0);
        assert_eq!(service.control().session_count(), 0);
    }

    #[tokio::test]
    async fn resolved_secret_reference_is_rejected_before_capability_issue() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-secret",
                priority: 10,
                mode: FixtureMode::SecretReference,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let failed = service
            .router()
            .oneshot(post(
                "/api/v1/sessions",
                serde_json::json!({
                    "request_id": "secret-reference",
                    "source": "fixture://secret",
                    "display_id": "display-a"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = json(failed).await;
        assert_eq!(body["code"], "MEDIA_INVALID");
        assert!(!body.to_string().contains("fixture-secret-ref"));
        assert_eq!(service.capability_count(), 0);
        assert_eq!(service.control().session_count(), 0);
    }

    #[tokio::test]
    async fn navigation_prepares_before_commit_and_reuses_control_authority() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(FixtureAdapter {
                plugin: "fixture-navigation",
                priority: 10,
                mode: FixtureMode::Navigation,
            }))
            .unwrap();
        let service = service_with_registry(registry);
        register_display(&service).await;
        let created = json(
            service
                .router()
                .oneshot(post(
                    "/api/v1/sessions",
                    serde_json::json!({
                        "request_id": "navigation-create",
                        "source": "fixture://middle",
                        "display_id": "display-a"
                    }),
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(created["item_revision"], 1);
        let session_id = created["session_id"].as_str().unwrap().to_owned();
        let path = format!("/api/v1/sessions/{session_id}/commands");
        let next = service
            .router()
            .oneshot(post(
                &path,
                serde_json::json!({
                    "request_id": "navigation-next",
                    "expected_session_revision": 0,
                    "command": {"type": "next_item"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(next.status(), StatusCode::OK);
        let next_body = json(next).await;
        assert_eq!(next_body["session_revision"], 1);
        assert_eq!(next_body["snapshot"]["current_item"]["item_revision"], 2);
        assert_eq!(next_body["snapshot"]["position_ms"], 0);

        let duplicate = service
            .router()
            .oneshot(post(
                &path,
                serde_json::json!({
                    "request_id": "navigation-next",
                    "expected_session_revision": 0,
                    "command": {"type": "next_item"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(duplicate.status(), StatusCode::OK);
        let duplicate_body = json(duplicate).await;
        assert_eq!(duplicate_body["session_revision"], 1);
        assert_eq!(
            duplicate_body["snapshot"]["current_item"]["item_revision"],
            2
        );

        let mismatch = service
            .router()
            .oneshot(post(
                &path,
                serde_json::json!({
                    "request_id": "navigation-next",
                    "expected_session_revision": 0,
                    "command": {"type": "previous_item"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(mismatch.status(), StatusCode::CONFLICT);
        assert_eq!(json(mismatch).await["code"], "REQUEST_ID_MISMATCH");
        assert_eq!(
            service
                .control()
                .snapshot(&session_id)
                .unwrap()
                .session_revision,
            1
        );

        let edge = service
            .router()
            .oneshot(post(
                &path,
                serde_json::json!({
                    "request_id": "navigation-edge",
                    "expected_session_revision": 1,
                    "command": {"type": "next_item"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(edge.status(), StatusCode::CONFLICT);
        assert_eq!(json(edge).await["code"], "NAVIGATION_NO_TARGET");
        assert_eq!(
            service
                .control()
                .snapshot(&session_id)
                .unwrap()
                .current_item
                .item_revision,
            2
        );

        let previous = service
            .router()
            .oneshot(post(
                &path,
                serde_json::json!({
                    "request_id": "navigation-previous",
                    "expected_session_revision": 1,
                    "command": {"type": "previous_item"}
                }),
            ))
            .await
            .unwrap();
        assert_eq!(previous.status(), StatusCode::OK);
        let previous_body = json(previous).await;
        assert_eq!(previous_body["session_revision"], 2);
        assert_eq!(
            previous_body["snapshot"]["current_item"]["item_revision"],
            3
        );
    }

    #[test]
    fn reversed_post_commit_publication_keeps_latest_media_projection() {
        // This models two successful Playback commits whose projections are
        // published in reverse order. It is intentionally deterministic: the
        // publication boundary itself must reject the older item revision.
        let service = SourceSessionService::new(Arc::new(SiteAdapterRegistry::default()));
        let view = |item_id: &str, item_revision: u64| SessionMediaView {
            session_id: "session-navigation".into(),
            item_id: item_id.into(),
            item_revision,
            media_generation: 0,
            title: format!("title-{item_revision}"),
            source_site: "fixture".into(),
            streams: vec![],
        };

        let newer = view("item-newer", 3);
        let older = view("item-older", 2);
        service.publish_media_view(newer);
        service.publish_media_view(older);

        let snapshot = crate::ControlSnapshot {
            session_id: "session-navigation".into(),
            session_revision: 2,
            state: "playing",
            current_item: crate::ControlItemSnapshot {
                item_id: "item-newer".into(),
                item_revision: 3,
                media_generation: 0,
            },
            position_ms: 0,
            telemetry_sequence: 0,
            active_display: crate::ControlDisplaySnapshot {
                display_id: "display-a".into(),
                generation: 0,
            },
            handoff: None,
        };
        let published = service
            .media_for_snapshot(&snapshot)
            .expect("latest projection remains available");
        assert_eq!(published.item_id, "item-newer");
        assert_eq!(published.item_revision, 3);
        assert_eq!(published.title, "title-3");
    }
}
