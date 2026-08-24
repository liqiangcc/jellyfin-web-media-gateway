use display_adapter_api::{
    DisplayAdapter, DisplayAdapterError, DisplayCommand, DisplayContext, DisplayInstance,
    DisplayStatus, GatewayMediaCapability, PlaybackObservation, PositionSample, PrepareRequest,
    PrepareResult, ProbeRequest, StartRequest, StartResult,
};
use futures_util::future::BoxFuture;
use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

const TICKS_PER_MS: u64 = 10_000;

/// A local Jellyfin endpoint explicitly supplied by deployment configuration.
/// There is deliberately no constructor that accepts a browser/plugin request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfiguredJellyfinService {
    pub service_id: String,
    pub endpoint: Url,
}

impl ConfiguredJellyfinService {
    pub fn new(service_id: impl Into<String>, endpoint: Url) -> Result<Self, DisplayAdapterError> {
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || endpoint.username() != ""
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(DisplayAdapterError::InvalidConfiguration);
        }
        Ok(Self {
            service_id: service_id.into(),
            endpoint,
        })
    }

    fn url(&self, path: &str) -> Result<Url, DisplayAdapterError> {
        self.endpoint
            .join(path)
            .map_err(|_| DisplayAdapterError::InvalidConfiguration)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterTimeouts {
    pub request: Duration,
    pub playback_confirmation: Duration,
    pub poll_interval: Duration,
}

/// A Jellyfin library item backed by a server-side `.strm` file. The file's
/// contents are the Gateway capability URL; the adapter never sends that URL
/// as an invented field to the Session Play endpoint. It first asks Jellyfin
/// for PlaybackInfo and uses the returned real media-source identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JellyfinStrmEntry {
    pub item_id: String,
}

impl JellyfinStrmEntry {
    pub fn new(item_id: impl Into<String>) -> Result<Self, DisplayAdapterError> {
        let item_id = item_id.into();
        Uuid::parse_str(&item_id).map_err(|_| DisplayAdapterError::InvalidConfiguration)?;
        Ok(Self { item_id })
    }
}

impl Default for AdapterTimeouts {
    fn default() -> Self {
        Self {
            request: Duration::from_secs(3),
            playback_confirmation: Duration::from_secs(5),
            poll_interval: Duration::from_millis(100),
        }
    }
}

/// Credentials are held only by this adapter and never serialized or included
/// in Debug output. The real deployment should load this value server-side.
#[derive(Clone, Eq, PartialEq)]
pub struct JellyfinCredential(String);

impl JellyfinCredential {
    pub fn server_side(value: impl Into<String>) -> Result<Self, DisplayAdapterError> {
        let value = value.into();
        if value.is_empty() || value.contains('\n') || value.contains('\r') {
            return Err(DisplayAdapterError::InvalidConfiguration);
        }
        Ok(Self(value))
    }
}

impl fmt::Debug for JellyfinCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JellyfinCredential(REDACTED)")
    }
}

#[derive(Clone)]
pub struct JellyfinDisplayAdapter {
    service: ConfiguredJellyfinService,
    credential: JellyfinCredential,
    media_entry: JellyfinStrmEntry,
    client: Client,
    timeouts: AdapterTimeouts,
}

impl fmt::Debug for JellyfinDisplayAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JellyfinDisplayAdapter")
            .field("service", &self.service)
            .field("credential", &self.credential)
            .field("media_entry", &self.media_entry)
            .field("timeouts", &self.timeouts)
            .finish()
    }
}

impl JellyfinDisplayAdapter {
    pub fn new(
        service: ConfiguredJellyfinService,
        credential: JellyfinCredential,
        media_entry: JellyfinStrmEntry,
        timeouts: AdapterTimeouts,
    ) -> Result<Self, DisplayAdapterError> {
        if timeouts.request.is_zero()
            || timeouts.playback_confirmation.is_zero()
            || timeouts.poll_interval.is_zero()
        {
            return Err(DisplayAdapterError::InvalidConfiguration);
        }
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeouts.request)
            .build()
            .map_err(|_| DisplayAdapterError::InvalidConfiguration)?;
        Ok(Self {
            service,
            credential,
            media_entry,
            client,
            timeouts,
        })
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T, DisplayAdapterError> {
        let url = self.service.url(path)?;
        let mut request = self
            .client
            .request(method, url)
            .header("X-Emby-Token", &self.credential.0);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(map_request_error)?;
        let status = response.status();
        map_status(status)?;
        response
            .json::<T>()
            .await
            .map_err(|_| DisplayAdapterError::Protocol("invalid Jellyfin JSON".into()))
    }

    async fn request_empty(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<(), DisplayAdapterError> {
        let url = self.service.url(path)?;
        let mut request = self
            .client
            .request(method, url)
            .header("X-Emby-Token", &self.credential.0);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(map_request_error)?;
        map_status(response.status())
    }

    async fn sessions(&self) -> Result<Vec<JellyfinSession>, DisplayAdapterError> {
        self.request_json(Method::GET, "Sessions", None).await
    }

    async fn playback_info(
        &self,
        media: &GatewayMediaCapability,
    ) -> Result<JellyfinMediaSource, DisplayAdapterError> {
        let info: JellyfinPlaybackInfo = self
            .request_json(
                Method::GET,
                &format!("Items/{}/PlaybackInfo", self.media_entry.item_id),
                None,
            )
            .await?;
        let expected = media.url().as_str();
        info.media_sources
            .into_iter()
            .find(|source| source.path == expected)
            .filter(|source| !source.id.is_empty())
            .ok_or(DisplayAdapterError::MediaIncompatible)
    }

    fn validate_context(
        request_context: &DisplayContext,
        media_context: &DisplayContext,
    ) -> Result<(), DisplayAdapterError> {
        if request_context != media_context {
            return Err(DisplayAdapterError::StaleContext);
        }
        Ok(())
    }

    async fn target(
        &self,
        display_id: &str,
    ) -> Result<(DisplayInstance, JellyfinSession), DisplayAdapterError> {
        let sessions = self.sessions().await?;
        let matches: Vec<_> = sessions
            .into_iter()
            .filter(|session| session.id == display_id)
            .collect();
        let [session] = matches.as_slice() else {
            return if matches.is_empty() {
                Err(DisplayAdapterError::TargetMissing)
            } else {
                Err(DisplayAdapterError::TargetAmbiguous)
            };
        };
        let display = display_from_session(session)?;
        if !display.online {
            return Err(DisplayAdapterError::TargetOffline);
        }
        Ok((display, session.clone()))
    }

    fn validate_media(media: &GatewayMediaCapability) -> Result<(), DisplayAdapterError> {
        let url = media.url();
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.username() != ""
            || url.password().is_some()
            || url.fragment().is_some()
            || !url.path().contains("/stream/")
        {
            return Err(DisplayAdapterError::MediaIncompatible);
        }
        Ok(())
    }

    async fn status_inner(
        &self,
        context: DisplayContext,
        requested_ms: Option<u64>,
    ) -> Result<DisplayStatus, DisplayAdapterError> {
        let (_, session) = self.target(&context.display_id).await?;
        let reported_ms = ticks_to_ms(session.play_state.position_ticks)?;
        Ok(DisplayStatus {
            context,
            observation: observation(&session.play_state),
            position: PositionSample {
                requested_ms,
                reported_ms,
                error_ms: requested_ms.map(|requested| reported_ms as i64 - requested as i64),
            },
        })
    }

    async fn confirm_playing(
        &self,
        context: DisplayContext,
        requested_ms: u64,
    ) -> Result<DisplayStatus, DisplayAdapterError> {
        let wait = async {
            loop {
                let status = self
                    .status_inner(context.clone(), Some(requested_ms))
                    .await?;
                if status.observation == PlaybackObservation::Playing {
                    return Ok(status);
                }
                tokio::time::sleep(self.timeouts.poll_interval).await;
            }
        };
        tokio::time::timeout(self.timeouts.playback_confirmation, wait)
            .await
            .map_err(|_| DisplayAdapterError::PlaybackNotConfirmed {
                timeout_ms: self.timeouts.playback_confirmation.as_millis() as u64,
            })?
    }

    async fn command_inner(
        &self,
        context: DisplayContext,
        command: DisplayCommand,
    ) -> Result<DisplayStatus, DisplayAdapterError> {
        let (_, _) = self.target(&context.display_id).await?;
        let (path, body) = match command {
            DisplayCommand::Pause => (
                format!("Sessions/{}/Playing/Pause", context.display_id),
                None,
            ),
            DisplayCommand::Resume => (
                format!("Sessions/{}/Playing/Unpause", context.display_id),
                None,
            ),
            DisplayCommand::Seek { position_ms } => (
                format!("Sessions/{}/Playing/Seek", context.display_id),
                Some(serde_json::json!({
                    "SeekPositionTicks": ms_to_ticks(position_ms)?
                })),
            ),
            DisplayCommand::Stop => (
                format!("Sessions/{}/Playing/Stop", context.display_id),
                None,
            ),
        };
        self.request_empty(Method::POST, &path, body).await?;
        self.status_inner(context, None).await
    }
}

impl DisplayAdapter for JellyfinDisplayAdapter {
    fn adapter_type(&self) -> &'static str {
        "jellyfin"
    }

    fn list_or_register_displays<'a>(
        &'a self,
    ) -> BoxFuture<'a, Result<Vec<DisplayInstance>, DisplayAdapterError>> {
        Box::pin(async move {
            let mut displays = self
                .sessions()
                .await?
                .iter()
                .map(display_from_session)
                .collect::<Result<Vec<_>, _>>()?;
            displays.sort_by(|left, right| left.id.cmp(&right.id));
            Ok(displays
                .into_iter()
                .filter(|display| display.online)
                .collect())
        })
    }

    fn probe<'a>(
        &'a self,
        request: ProbeRequest,
    ) -> BoxFuture<'a, Result<DisplayInstance, DisplayAdapterError>> {
        Box::pin(async move {
            Self::validate_media(&request.media)?;
            Self::validate_context(&request.context, &request.media.context)?;
            let (display, _) = self.target(&request.context.display_id).await?;
            self.playback_info(&request.media).await?;
            Ok(display)
        })
    }

    fn prepare<'a>(
        &'a self,
        request: PrepareRequest,
    ) -> BoxFuture<'a, Result<PrepareResult, DisplayAdapterError>> {
        Box::pin(async move {
            Self::validate_media(&request.media)?;
            Self::validate_context(&request.context, &request.media.context)?;
            let (target, _) = self.target(&request.context.display_id).await?;
            self.playback_info(&request.media).await?;
            Ok(PrepareResult {
                context: request.context,
                target,
            })
        })
    }

    fn start<'a>(
        &'a self,
        request: StartRequest,
    ) -> BoxFuture<'a, Result<StartResult, DisplayAdapterError>> {
        Box::pin(async move {
            Self::validate_media(&request.media)?;
            Self::validate_context(&request.context, &request.media.context)?;
            let (_, _) = self.target(&request.context.display_id).await?;
            let media_source = self.playback_info(&request.media).await?;
            let ticks = ms_to_ticks(request.position_ms)?;
            let body = serde_json::json!({
                "PlayCommand": "PlayNow",
                "ItemIds": [self.media_entry.item_id.clone()],
                "MediaSourceId": media_source.id,
                "StartPositionTicks": ticks,
            });
            self.request_empty(
                Method::POST,
                &format!("Sessions/{}/Playing", request.context.display_id),
                Some(body),
            )
            .await?;
            let status = self
                .confirm_playing(request.context, request.position_ms)
                .await?;
            Ok(StartResult {
                status,
                command_accepted: true,
                playback_confirmed: true,
            })
        })
    }

    fn command<'a>(
        &'a self,
        context: DisplayContext,
        command: DisplayCommand,
    ) -> BoxFuture<'a, Result<DisplayStatus, DisplayAdapterError>> {
        Box::pin(self.command_inner(context, command))
    }

    fn status<'a>(
        &'a self,
        context: DisplayContext,
    ) -> BoxFuture<'a, Result<DisplayStatus, DisplayAdapterError>> {
        Box::pin(self.status_inner(context, None))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinSession {
    id: String,
    #[serde(default)]
    device_name: String,
    #[serde(default = "default_online")]
    is_online: bool,
    #[serde(default)]
    play_state: JellyfinPlayState,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinPlaybackInfo {
    #[serde(default)]
    media_sources: Vec<JellyfinMediaSource>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinMediaSource {
    id: String,
    path: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinPlayState {
    #[serde(default)]
    is_paused: bool,
    #[serde(default)]
    is_playing: bool,
    #[serde(default)]
    position_ticks: u64,
}

fn default_online() -> bool {
    true
}

fn display_from_session(session: &JellyfinSession) -> Result<DisplayInstance, DisplayAdapterError> {
    if session.id.is_empty() {
        return Err(DisplayAdapterError::Protocol("session has no id".into()));
    }
    Ok(DisplayInstance {
        id: session.id.clone(),
        adapter_type: "jellyfin".into(),
        label: if session.device_name.is_empty() {
            session.id.clone()
        } else {
            session.device_name.clone()
        },
        online: session.is_online,
        capabilities: vec![
            "probe".into(),
            "prepare".into(),
            "start".into(),
            "pause".into(),
            "resume".into(),
            "seek".into(),
            "stop".into(),
            "status".into(),
        ],
    })
}

fn observation(state: &JellyfinPlayState) -> PlaybackObservation {
    if state.is_playing && !state.is_paused {
        PlaybackObservation::Playing
    } else if state.is_paused {
        PlaybackObservation::Paused
    } else {
        PlaybackObservation::Stopped
    }
}

pub fn ms_to_ticks(position_ms: u64) -> Result<u64, DisplayAdapterError> {
    position_ms
        .checked_mul(TICKS_PER_MS)
        .ok_or_else(|| DisplayAdapterError::Protocol("position overflow".into()))
}

pub fn ticks_to_ms(position_ticks: u64) -> Result<u64, DisplayAdapterError> {
    position_ticks
        .checked_add(TICKS_PER_MS / 2)
        .map(|ticks| ticks / TICKS_PER_MS)
        .ok_or_else(|| DisplayAdapterError::Protocol("position overflow".into()))
}

fn map_request_error(error: reqwest::Error) -> DisplayAdapterError {
    if error.is_timeout() {
        DisplayAdapterError::Timeout
    } else if error.is_connect() {
        DisplayAdapterError::ServerUnavailable
    } else {
        DisplayAdapterError::Protocol("Jellyfin request failed".into())
    }
}

fn map_status(status: StatusCode) -> Result<(), DisplayAdapterError> {
    match status {
        StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            Err(DisplayAdapterError::AuthenticationFailed)
        }
        StatusCode::NOT_FOUND => Err(DisplayAdapterError::TargetMissing),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::UNSUPPORTED_MEDIA_TYPE => {
            Err(DisplayAdapterError::MediaIncompatible)
        }
        status if status.is_server_error() => Err(DisplayAdapterError::ServerUnavailable),
        _ => Err(DisplayAdapterError::CommandRejected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{Path, State};
    use axum::http::{HeaderMap, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::{get, post};
    use axum::{Json, Router};
    use display_adapter_api::{DisplayCommand, DisplayContext};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tokio::time::sleep;

    const SENTINEL: &str = "fake-jellyfin-secret-sentinel";
    const ITEM_ID: &str = "11111111-1111-4111-8111-111111111111";
    const MEDIA_SOURCE_ID: &str = "22222222-2222-4222-8222-222222222222";
    const MEDIA_URL: &str = "http://gateway.test/stream/capability-1";

    #[derive(Clone, Default)]
    struct Fixture {
        auth: String,
        online: bool,
        confirm_playback: bool,
        media_incompatible: bool,
        position_ticks: Arc<Mutex<u64>>,
        playing: Arc<Mutex<bool>>,
        paused: Arc<Mutex<bool>>,
        seen_auth: Arc<Mutex<Vec<String>>>,
        seen_start: Arc<Mutex<Vec<Value>>>,
    }

    async fn sessions(State(fixture): State<Fixture>, headers: HeaderMap) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return (StatusCode::UNAUTHORIZED, Json(json!({}))).into_response();
        }
        Json(json!([{
            "Id": "tv-1",
            "DeviceName": "Fixture TV",
            "IsOnline": fixture.online,
            "PlayState": {
                "IsPaused": *fixture.paused.lock().unwrap(),
                "IsPlaying": *fixture.playing.lock().unwrap(),
                "PositionTicks": *fixture.position_ticks.lock().unwrap()
            }
        }]))
        .into_response()
    }

    async fn playback_info(
        Path(item_id): Path<String>,
        State(fixture): State<Fixture>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return (StatusCode::UNAUTHORIZED, Json(json!({}))).into_response();
        }
        if item_id != ITEM_ID {
            return (StatusCode::NOT_FOUND, Json(json!({}))).into_response();
        }
        Json(json!({
            "MediaSources": [{
                "Id": MEDIA_SOURCE_ID,
                "Path": MEDIA_URL,
                "Protocol": "Http",
                "SupportsDirectPlay": true
            }]
        }))
        .into_response()
    }

    async fn start(
        State(fixture): State<Fixture>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return (StatusCode::UNAUTHORIZED, Json(json!({}))).into_response();
        }
        fixture.seen_start.lock().unwrap().push(body);
        if fixture.media_incompatible {
            return (StatusCode::UNSUPPORTED_MEDIA_TYPE, Json(json!({}))).into_response();
        }
        let body = fixture.seen_start.lock().unwrap().last().unwrap().clone();
        if body.get("MediaUrl").is_some()
            || body["ItemIds"] != json!([ITEM_ID])
            || body["MediaSourceId"] != MEDIA_SOURCE_ID
        {
            return (StatusCode::BAD_REQUEST, Json(json!({}))).into_response();
        }
        *fixture.position_ticks.lock().unwrap() = body["StartPositionTicks"].as_u64().unwrap();
        if fixture.confirm_playback {
            *fixture.playing.lock().unwrap() = true;
        }
        StatusCode::NO_CONTENT.into_response()
    }

    async fn pause(State(fixture): State<Fixture>, headers: HeaderMap) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        *fixture.playing.lock().unwrap() = false;
        *fixture.paused.lock().unwrap() = true;
        StatusCode::NO_CONTENT.into_response()
    }

    async fn unpause(State(fixture): State<Fixture>, headers: HeaderMap) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        *fixture.playing.lock().unwrap() = true;
        *fixture.paused.lock().unwrap() = false;
        StatusCode::NO_CONTENT.into_response()
    }

    async fn seek(
        State(fixture): State<Fixture>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        *fixture.position_ticks.lock().unwrap() = body["SeekPositionTicks"].as_u64().unwrap();
        StatusCode::NO_CONTENT.into_response()
    }

    async fn stop(State(fixture): State<Fixture>, headers: HeaderMap) -> impl IntoResponse {
        if !authorized(&fixture, &headers) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        *fixture.playing.lock().unwrap() = false;
        *fixture.paused.lock().unwrap() = false;
        StatusCode::NO_CONTENT.into_response()
    }

    fn authorized(fixture: &Fixture, headers: &HeaderMap) -> bool {
        if let Some(value) = headers.get("X-Emby-Token") {
            let value = value.to_str().unwrap().to_owned();
            fixture.seen_auth.lock().unwrap().push(value.clone());
            value == fixture.auth
        } else {
            false
        }
    }

    async fn spawn_fixture(fixture: Fixture) -> String {
        let app = Router::new()
            .route("/Sessions", get(sessions))
            .route("/Items/{id}/PlaybackInfo", get(playback_info))
            .route("/Sessions/{id}/Playing", post(start))
            .route("/Sessions/{id}/Playing/Pause", post(pause))
            .route("/Sessions/{id}/Playing/Unpause", post(unpause))
            .route("/Sessions/{id}/Playing/Seek", post(seek))
            .route("/Sessions/{id}/Playing/Stop", post(stop))
            .with_state(fixture);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        endpoint
    }

    fn context() -> DisplayContext {
        DisplayContext::new("session-1", "item-1", 4, "tv-1", 9)
    }

    fn media() -> GatewayMediaCapability {
        GatewayMediaCapability::from_gateway(Url::parse(MEDIA_URL).unwrap(), context()).unwrap()
    }

    async fn adapter(fixture: Fixture) -> JellyfinDisplayAdapter {
        let endpoint = spawn_fixture(fixture).await;
        JellyfinDisplayAdapter::new(
            ConfiguredJellyfinService::new("fixture", Url::parse(&endpoint).unwrap()).unwrap(),
            JellyfinCredential::server_side(SENTINEL).unwrap(),
            JellyfinStrmEntry::new(ITEM_ID).unwrap(),
            AdapterTimeouts {
                request: Duration::from_millis(300),
                playback_confirmation: Duration::from_millis(80),
                poll_interval: Duration::from_millis(10),
            },
        )
        .unwrap()
    }

    fn fixture() -> Fixture {
        Fixture {
            auth: SENTINEL.into(),
            online: true,
            confirm_playback: true,
            ..Fixture::default()
        }
    }

    #[test]
    fn position_conversion_is_bounded_and_measurable() {
        assert_eq!(
            ms_to_ticks(18 * 60 * 1000 + 24 * 1000).unwrap(),
            11_040_000_000
        );
        assert_eq!(ticks_to_ms(1_104_000_005_000).unwrap(), 110_400_001);
        assert_eq!(
            ms_to_ticks(u64::MAX),
            Err(DisplayAdapterError::Protocol("position overflow".into()))
        );
    }

    #[test]
    fn gateway_capability_rejects_upstream_secret_material() {
        let context = context();
        assert_eq!(
            GatewayMediaCapability::from_gateway(
                Url::parse("http://gateway.test/stream/capability?authorization=secret").unwrap(),
                context.clone(),
            )
            .unwrap_err(),
            display_adapter_api::CapabilityError::InvalidUrl
        );
        assert!(
            GatewayMediaCapability::from_gateway(
                Url::parse("http://gateway.test/stream/capability?token=opaque").unwrap(),
                context,
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn discovery_filters_offline_sessions_and_probe_is_stable() {
        let mut offline_fixture = fixture();
        offline_fixture.online = false;
        let offline_adapter = adapter(offline_fixture).await;
        assert!(
            offline_adapter
                .list_or_register_displays()
                .await
                .unwrap()
                .is_empty()
        );
        let error = offline_adapter
            .probe(ProbeRequest {
                context: context(),
                media: media(),
            })
            .await
            .unwrap_err();
        assert_eq!(error, DisplayAdapterError::TargetOffline);

        let available_adapter = adapter(fixture()).await;
        let mut missing = context();
        missing.display_id = "missing-tv".into();
        let error = available_adapter.status(missing).await.unwrap_err();
        assert_eq!(error, DisplayAdapterError::TargetMissing);
    }

    #[tokio::test]
    async fn start_requires_playback_confirmation_and_records_position() {
        let mut fixture = fixture();
        fixture.confirm_playback = false;
        let adapter = adapter(fixture.clone()).await;
        let error = adapter
            .start(StartRequest {
                context: context(),
                media: media(),
                position_ms: 1_104_000,
            })
            .await
            .unwrap_err();
        assert_eq!(
            error,
            DisplayAdapterError::PlaybackNotConfirmed { timeout_ms: 80 }
        );
        let body = fixture.seen_start.lock().unwrap()[0].clone();
        assert_eq!(body["StartPositionTicks"], 11_040_000_000u64);
        assert!(body.get("MediaUrl").is_none());
        assert_eq!(body["ItemIds"], json!([ITEM_ID]));
        assert_eq!(body["MediaSourceId"], MEDIA_SOURCE_ID);
        assert!(format!("{error:?}").contains("PlaybackNotConfirmed"));
        assert!(!format!("{error:?}").contains(SENTINEL));
    }

    #[tokio::test]
    async fn commands_map_to_fixture_and_status_is_context_bound() {
        let adapter = adapter(fixture()).await;
        let started = adapter
            .start(StartRequest {
                context: context(),
                media: media(),
                position_ms: 1234,
            })
            .await
            .unwrap();
        assert!(started.command_accepted && started.playback_confirmed);
        assert_eq!(started.status.position.reported_ms, 1234);
        let paused = adapter
            .command(context(), DisplayCommand::Pause)
            .await
            .unwrap();
        assert_eq!(paused.observation, PlaybackObservation::Paused);
        let resumed = adapter
            .command(context(), DisplayCommand::Resume)
            .await
            .unwrap();
        assert_eq!(resumed.observation, PlaybackObservation::Playing);
        let sought = adapter
            .command(context(), DisplayCommand::Seek { position_ms: 2500 })
            .await
            .unwrap();
        assert_eq!(sought.position.reported_ms, 2500);
        let stopped = adapter
            .command(context(), DisplayCommand::Stop)
            .await
            .unwrap();
        assert_eq!(stopped.observation, PlaybackObservation::Stopped);
        assert_eq!(stopped.context.display_generation, 9);
    }

    #[tokio::test]
    async fn auth_failure_and_media_failure_are_stable_without_secret_leakage() {
        let mut auth_fixture = fixture();
        auth_fixture.auth = "different-server-key".into();
        let auth_adapter = adapter(auth_fixture).await;
        let error = auth_adapter.status(context()).await.unwrap_err();
        assert_eq!(error, DisplayAdapterError::AuthenticationFailed);
        assert!(!format!("{auth_adapter:?}").contains(SENTINEL));

        let incompatible_fixture = Fixture {
            auth: SENTINEL.into(),
            online: true,
            media_incompatible: true,
            ..fixture()
        };
        let incompatible_adapter = adapter(incompatible_fixture).await;
        let error = incompatible_adapter
            .start(StartRequest {
                context: context(),
                media: media(),
                position_ms: 0,
            })
            .await
            .unwrap_err();
        assert_eq!(error, DisplayAdapterError::MediaIncompatible);
    }

    #[tokio::test]
    async fn stale_context_is_returned_as_candidate_evidence_not_global_mutation() {
        let adapter = adapter(fixture()).await;
        let old = DisplayContext::new("session-1", "old-item", 1, "tv-1", 2);
        let result = adapter.status(old.clone()).await.unwrap();
        assert_eq!(result.context, old);
        assert_eq!(result.context.display_generation, 2);
        sleep(Duration::from_millis(1)).await;
    }

    #[tokio::test]
    async fn unavailable_server_is_a_bounded_stable_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let adapter = JellyfinDisplayAdapter::new(
            ConfiguredJellyfinService::new("offline", Url::parse(&endpoint).unwrap()).unwrap(),
            JellyfinCredential::server_side(SENTINEL).unwrap(),
            JellyfinStrmEntry::new(ITEM_ID).unwrap(),
            AdapterTimeouts {
                request: Duration::from_millis(100),
                playback_confirmation: Duration::from_millis(50),
                poll_interval: Duration::from_millis(5),
            },
        )
        .unwrap();
        assert_eq!(
            adapter.status(context()).await.unwrap_err(),
            DisplayAdapterError::ServerUnavailable
        );
    }
}
