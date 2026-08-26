//! Production source-to-session preparation and atomic publication.
//!
//! This module owns only the composition of already accepted generic
//! contracts. Site knowledge stays behind `SiteAdapterRegistry`; display
//! registration/liveness stays behind `DisplaySessionService`; Playback
//! authority stays behind `ControlService`.

use crate::control::{
    ControlCommandError, ControlCommandRequest, ControlCommandResponse, ControlService,
    NavigationStart,
};
use crate::display_session::{DisplaySessionError, DisplaySessionService};
use crate::playback::{Command, CommandError, NavigationTicket};
use crate::{Binding, EgressScope, GatewayError, GatewayService};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use site_adapter_api::{
    AdapterError, MediaProtection, NavigationDirection, ResolvedMedia, SiteAdapterRegistry,
    StreamProtocol,
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

        let outcome = self.create_fresh(gateway, control, displays, &request);
        creations.insert(
            request.request_id,
            CreationRecord {
                fingerprint,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    fn create_fresh(
        &self,
        gateway: &GatewayService,
        control: &ControlService,
        displays: &DisplaySessionService,
        request: &CreateSessionRequest,
    ) -> CreationOutcome {
        if let Err(error) = displays.validate_live_selector(&request.display_id) {
            return failure_for_display(error);
        }

        let locator = match self.registry.recognize(&request.source) {
            Ok(locator) => locator,
            Err(error) => return failure_for_adapter(error),
        };
        let media = match self.registry.resolve(&locator) {
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
    use crate::GatewayService;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use generic_direct::GenericDirectAdapter;
    use site_adapter_api::{
        AdapterError, MediaProtection, RecognizeResult, ResolvedMedia, ResolvedStream, SiteAdapter,
        SiteAdapterRegistry, SourceLocator, StreamProtocol,
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
            let matched = input.starts_with("fixture://");
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
            let stream = |id: &str, headers: BTreeMap<String, String>| ResolvedStream {
                id: id.into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://example.test/fixture.mp4").unwrap(),
                public_headers: headers,
                upstream_access_ref: None,
            };
            let streams = match self.mode {
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
            Ok(ResolvedMedia {
                title: "fixture media".into(),
                source_site: self.site_id().into(),
                streams,
                subtitles: vec![],
                protection: MediaProtection::Clear,
            })
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
        let service = service();
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
