use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::header::{
    ACCEPT_RANGES, AUTHORIZATION, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
    HOST, IF_RANGE, LAST_MODIFIED, ORIGIN, RANGE,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use site_adapter_api::{ResolvedStream, StreamProtocol};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

pub mod auth;
pub mod browser;
pub mod security;
pub use auth::{
    AccountState, AuthBoundaryError, CandidateValidation, CleanupResult, PendingIntent,
    PendingPlaybackAction, PendingSourceLocator, ScopedHttpResponse, ScopedSiteHttpClient,
    SessionSwapResult, SessionVault, SiteAccessContext, SiteAccount, SiteSessionRef, VaultError,
};
pub use security::{
    EgressPolicy, EgressPolicyError, EgressScope, HttpAuthorityError, HttpAuthorityPolicy,
    SiteAccessCapability, SiteAccessError, ValidatedTarget,
};

const MAX_MANIFEST_BYTES: usize = 512 * 1024;
const MAX_REDIRECTS: usize = 5;
const MAX_HTTP_BODY_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub resource_id: String,
}

impl Binding {
    pub fn new(session: &str, item: &str, item_revision: u64, resource: &str) -> Self {
        Self {
            session_id: session.into(),
            item_id: item.into(),
            item_revision,
            resource_id: resource.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpstreamResource {
    pub url: Url,
    pub protocol: StreamProtocol,
    pub public_headers: HeaderMap,
    pub secret_headers: HeaderMap,
    pub egress_scope: EgressScope,
}

#[derive(Clone, Debug)]
struct CapabilityRecord {
    binding: Binding,
    resource: UpstreamResource,
    expires_at: Instant,
    created_seq: u64,
}

#[derive(Debug, Eq, PartialEq)]
enum CapabilityError {
    NotFound,
    Expired,
    BindingMismatch,
}

#[derive(Debug)]
struct StoreInner {
    records: HashMap<String, CapabilityRecord>,
    order: VecDeque<(u64, String)>,
}

#[derive(Debug)]
struct CapabilityStore {
    inner: Mutex<StoreInner>,
    max_entries: usize,
    sequence: AtomicU64,
}

impl CapabilityStore {
    fn new(max_entries: usize) -> Self {
        assert!(max_entries > 0);
        Self {
            inner: Mutex::new(StoreInner {
                records: HashMap::new(),
                order: VecDeque::new(),
            }),
            max_entries,
            sequence: AtomicU64::new(1),
        }
    }

    fn issue(&self, binding: Binding, resource: UpstreamResource, ttl: Duration) -> String {
        let now = Instant::now();
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        let token = Uuid::new_v4().simple().to_string();
        let mut inner = self.inner.lock().expect("capability store poisoned");
        inner.records.retain(|_, record| record.expires_at > now);
        while inner.records.len() >= self.max_entries {
            if let Some((old_seq, old_token)) = inner.order.pop_front() {
                let should_remove = inner
                    .records
                    .get(&old_token)
                    .is_some_and(|record| record.created_seq == old_seq);
                if should_remove {
                    inner.records.remove(&old_token);
                }
            } else {
                break;
            }
        }
        inner.order.push_back((seq, token.clone()));
        inner.records.insert(
            token.clone(),
            CapabilityRecord {
                binding,
                resource,
                expires_at: now + ttl,
                created_seq: seq,
            },
        );
        token
    }

    fn get(&self, token: &str, binding: &Binding) -> Result<CapabilityRecord, CapabilityError> {
        let now = Instant::now();
        let mut inner = self.inner.lock().expect("capability store poisoned");
        let Some(record) = inner.records.get(token).cloned() else {
            return Err(CapabilityError::NotFound);
        };
        if record.expires_at <= now {
            inner.records.remove(token);
            return Err(CapabilityError::Expired);
        }
        if &record.binding != binding {
            return Err(CapabilityError::BindingMismatch);
        }
        Ok(record)
    }

    fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("capability store poisoned")
            .records
            .len()
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ProofPaths {
    pub mp4_path: Option<String>,
    pub display_path: Option<String>,
    pub hls_path: Option<String>,
    pub secret_path: Option<String>,
    pub chain: String,
}

#[derive(Clone)]
struct GatewayState {
    store: Arc<CapabilityStore>,
    egress_policy: Arc<RwLock<EgressPolicy>>,
    http_authorities: Arc<RwLock<HttpAuthorityPolicy>>,
    active_streams: Arc<AtomicUsize>,
    proof_paths: Arc<RwLock<ProofPaths>>,
    fixture_mp4: Arc<RwLock<Option<PathBuf>>>,
    probe: Arc<ProbeStore>,
}

#[derive(Clone)]
pub struct GatewayService {
    state: Arc<GatewayState>,
}

#[derive(Debug)]
pub enum GatewayError {
    InvalidHeader,
    SecretHeader,
}

const MAX_PROBE_RECORDS: usize = 256;

#[derive(Clone, Debug, Serialize)]
struct ProbeCommand {
    sequence: u64,
    request_id: String,
    kind: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeTelemetry {
    sequence: u64,
    kind: String,
    command_id: Option<String>,
    attempt_id: Option<String>,
    result: Option<String>,
    error_name: Option<String>,
    error_message: Option<String>,
    muted: Option<bool>,
    volume: Option<f64>,
    detail: Option<String>,
}

#[derive(Debug, Default)]
struct ProbeInner {
    next_command: u64,
    commands: VecDeque<ProbeCommand>,
    request_ids: HashMap<String, u64>,
    next_telemetry: u64,
    telemetry: VecDeque<ProbeTelemetry>,
}

#[derive(Debug, Default)]
struct ProbeStore {
    inner: Mutex<ProbeInner>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlayCommandRequest {
    request_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeEventsQuery {
    after: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeTelemetryRequest {
    kind: String,
    command_id: Option<String>,
    attempt_id: Option<String>,
    result: Option<String>,
    error_name: Option<String>,
    error_message: Option<String>,
    muted: Option<bool>,
    volume: Option<f64>,
    detail: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeStateSnapshot {
    cursor: u64,
    commands: Vec<ProbeCommand>,
    telemetry: Vec<ProbeTelemetry>,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeEventsResponse {
    cursor: u64,
    events: Vec<ProbeCommand>,
    truncated: bool,
}

#[derive(Clone, Debug, Serialize)]
struct PlayCommandResponse {
    accepted: bool,
    duplicate: bool,
    command: ProbeCommand,
}

impl ProbeStore {
    fn issue_play(&self, request_id: String) -> (ProbeCommand, bool) {
        let mut inner = self.inner.lock().expect("probe store poisoned");
        if let Some(sequence) = inner.request_ids.get(&request_id).copied()
            && let Some(command) = inner
                .commands
                .iter()
                .find(|command| command.sequence == sequence)
        {
            return (command.clone(), true);
        }

        inner.next_command += 1;
        let command = ProbeCommand {
            sequence: inner.next_command,
            request_id: request_id.clone(),
            kind: "play",
        };
        inner.request_ids.insert(request_id, command.sequence);
        inner.commands.push_back(command.clone());
        while inner.commands.len() > MAX_PROBE_RECORDS {
            if let Some(old) = inner.commands.pop_front() {
                inner.request_ids.remove(&old.request_id);
            }
        }
        (command, false)
    }

    fn state_snapshot(&self) -> ProbeStateSnapshot {
        let inner = self.inner.lock().expect("probe store poisoned");
        ProbeStateSnapshot {
            cursor: inner.next_command,
            commands: inner.commands.iter().cloned().collect(),
            telemetry: inner.telemetry.iter().cloned().collect(),
        }
    }

    fn events_after(&self, after: u64) -> ProbeEventsResponse {
        let inner = self.inner.lock().expect("probe store poisoned");
        let oldest = inner
            .commands
            .front()
            .map_or(inner.next_command + 1, |e| e.sequence);
        ProbeEventsResponse {
            cursor: inner.next_command,
            events: inner
                .commands
                .iter()
                .filter(|event| event.sequence > after)
                .cloned()
                .collect(),
            truncated: after + 1 < oldest,
        }
    }

    fn append_telemetry(&self, input: ProbeTelemetryRequest) -> u64 {
        let mut inner = self.inner.lock().expect("probe store poisoned");
        inner.next_telemetry += 1;
        let telemetry = ProbeTelemetry {
            sequence: inner.next_telemetry,
            kind: sanitize_text(&input.kind, 48),
            command_id: sanitize_optional(input.command_id, 128),
            attempt_id: sanitize_optional(input.attempt_id, 128),
            result: sanitize_optional(input.result, 48),
            error_name: sanitize_optional(input.error_name, 96),
            error_message: sanitize_optional(input.error_message, 256),
            muted: input.muted,
            volume: input.volume.filter(|value| value.is_finite()),
            detail: sanitize_optional(input.detail, 256),
        };
        inner.telemetry.push_back(telemetry);
        while inner.telemetry.len() > MAX_PROBE_RECORDS {
            inner.telemetry.pop_front();
        }
        inner.next_telemetry
    }

    fn reset(&self) {
        *self.inner.lock().expect("probe store poisoned") = ProbeInner::default();
    }
}

fn sanitize_optional(value: Option<String>, max: usize) -> Option<String> {
    value.map(|value| sanitize_text(&value, max))
}

fn sanitize_text(value: &str, max: usize) -> String {
    let mut output = String::new();
    let mut redact_next = false;
    for token in value.split_whitespace() {
        if redact_next {
            output.push_str("[redacted]");
            redact_next = false;
        } else if token.eq_ignore_ascii_case("bearer")
            || token.eq_ignore_ascii_case("basic")
            || token.eq_ignore_ascii_case("cookie:")
        {
            output.push_str(if token.eq_ignore_ascii_case("cookie:") {
                "[redacted-header]"
            } else {
                token
            });
            redact_next = true;
        } else if token.eq_ignore_ascii_case("cookie") {
            output.push_str("[redacted-header]");
            redact_next = true;
        } else {
            output.push_str(token);
        }
        output.push(' ');
    }
    output
        .trim_end()
        .chars()
        .filter(|character| !character.is_control())
        .take(max)
        .collect()
}

impl GatewayService {
    pub fn new(max_capabilities: usize) -> Self {
        Self {
            state: Arc::new(GatewayState {
                store: Arc::new(CapabilityStore::new(max_capabilities)),
                egress_policy: Arc::new(RwLock::new(EgressPolicy::default())),
                http_authorities: Arc::new(RwLock::new(HttpAuthorityPolicy::default())),
                active_streams: Arc::new(AtomicUsize::new(0)),
                proof_paths: Arc::new(RwLock::new(ProofPaths {
                    chain: "SiteAdapterRegistry -> generic-direct -> ResolvedMedia -> MediaGateway -> WebDisplay".into(),
                    ..ProofPaths::default()
                })),
                fixture_mp4: Arc::new(RwLock::new(None)),
                probe: Arc::new(ProbeStore::default()),
            }),
        }
    }

    /// Register a private integration from deployment/admin configuration.
    /// User/plugin URLs cannot create this exception because validation also
    /// requires the named entry and its configured origin.
    pub fn configure_local_service(
        &self,
        name: impl Into<String>,
        origin: Url,
    ) -> Result<(), EgressPolicyError> {
        self.state
            .egress_policy
            .write()
            .expect("egress policy poisoned")
            .configure_local_service(name, &origin)
    }

    /// Configure the exact deployment authority accepted by the HTTP/control
    /// surface. This is deployment-owned; request Host/Origin values cannot
    /// create or widen the authority set.
    pub fn configure_http_authority(&self, origin: Url) -> Result<(), HttpAuthorityError> {
        self.state
            .http_authorities
            .write()
            .expect("http authority policy poisoned")
            .configure(&origin)
    }

    pub fn resource_from_resolved(
        stream: &ResolvedStream,
        scope: EgressScope,
    ) -> Result<UpstreamResource, GatewayError> {
        let mut public_headers = HeaderMap::new();
        for (name, value) in &stream.public_headers {
            if security::is_secret_header(name, value) {
                return Err(GatewayError::SecretHeader);
            }
            let name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|_| GatewayError::InvalidHeader)?;
            let value = HeaderValue::from_str(value).map_err(|_| GatewayError::InvalidHeader)?;
            public_headers.insert(name, value);
        }
        Ok(UpstreamResource {
            url: stream.url.clone(),
            protocol: stream.protocol,
            public_headers,
            secret_headers: HeaderMap::new(),
            egress_scope: scope,
        })
    }

    pub fn issue_path(
        &self,
        binding: Binding,
        resource: UpstreamResource,
        ttl: Duration,
    ) -> String {
        let token = self.state.store.issue(binding.clone(), resource, ttl);
        stream_path(&token, &binding)
    }

    pub fn configure_proof_paths(&self, paths: ProofPaths) {
        *self
            .state
            .proof_paths
            .write()
            .expect("proof paths poisoned") = paths;
    }

    pub fn configure_fixture_mp4(&self, path: Option<PathBuf>) {
        *self
            .state
            .fixture_mp4
            .write()
            .expect("fixture path poisoned") = path;
    }

    pub fn active_streams(&self) -> usize {
        self.state.active_streams.load(Ordering::SeqCst)
    }

    pub fn capability_count(&self) -> usize {
        self.state.store.len()
    }

    pub fn max_capabilities(&self) -> usize {
        self.state.store.max_entries
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route(
                "/stream/{token}/{session}/{item}/{revision}/{resource}",
                get(stream_handler).head(stream_handler),
            )
            .route("/metrics", get(metrics_handler))
            .route("/proof/paths", get(proof_paths_handler))
            .route("/display", get(display_handler))
            .route("/control", get(control_handler))
            .route("/api/v1/display-probe/state", get(probe_state_handler))
            .route("/api/v1/display-probe/events", get(probe_events_handler))
            .route(
                "/api/v1/display-probe/commands",
                post(probe_command_handler),
            )
            .route(
                "/api/v1/display-probe/telemetry",
                post(probe_telemetry_handler),
            )
            .route("/api/v1/display-probe/reset", post(probe_reset_handler))
            .route("/secret-display", get(secret_display_handler))
            .route(
                "/fixture/protected.mp4",
                get(fixture_handler).head(fixture_handler),
            )
            .route("/healthz", get(|| async { "ok" }))
            .layer(DefaultBodyLimit::max(MAX_HTTP_BODY_BYTES))
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                http_surface_guard,
            ))
            .with_state(self.state.clone())
    }
}

async fn http_surface_guard(
    State(state): State<Arc<GatewayState>>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let Some(host) = headers.get(HOST).and_then(|value| value.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST, "invalid_host").into_response();
    };
    let http_authorities = state
        .http_authorities
        .read()
        .expect("http authority policy poisoned")
        .clone();
    if !security::is_valid_http_host(host) {
        return (StatusCode::BAD_REQUEST, "invalid_host").into_response();
    }
    if !http_authorities.allows_host(host) {
        return (StatusCode::MISDIRECTED_REQUEST, "host_not_allowed").into_response();
    }

    if let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok())
        && !http_authorities.allows_origin_for_host(origin, host)
    {
        return (StatusCode::FORBIDDEN, "origin_mismatch").into_response();
    }

    if matches!(
        request.method(),
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    ) {
        let is_json = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.split(';').next().is_some_and(|media_type| {
                    media_type.trim().eq_ignore_ascii_case("application/json")
                })
            });
        if !is_json {
            return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "content_type_required").into_response();
        }
        let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok()) else {
            return (StatusCode::FORBIDDEN, "origin_required").into_response();
        };
        if !http_authorities.allows_origin_for_host(origin, host) {
            return (StatusCode::FORBIDDEN, "origin_mismatch").into_response();
        }
    }

    next.run(request).await
}

fn stream_path(token: &str, binding: &Binding) -> String {
    format!(
        "/stream/{}/{}/{}/{}/{}",
        token, binding.session_id, binding.item_id, binding.item_revision, binding.resource_id
    )
}

#[derive(Serialize)]
struct Metrics {
    active_streams: usize,
    capability_count: usize,
    capability_limit: usize,
}

async fn metrics_handler(State(state): State<Arc<GatewayState>>) -> Json<Metrics> {
    Json(Metrics {
        active_streams: state.active_streams.load(Ordering::SeqCst),
        capability_count: state.store.len(),
        capability_limit: state.store.max_entries,
    })
}

async fn proof_paths_handler(State(state): State<Arc<GatewayState>>) -> Json<ProofPaths> {
    Json(
        state
            .proof_paths
            .read()
            .expect("proof paths poisoned")
            .clone(),
    )
}

async fn display_handler(State(state): State<Arc<GatewayState>>) -> Response {
    let paths = state
        .proof_paths
        .read()
        .expect("proof paths poisoned")
        .clone();
    let path = paths.display_path.or(paths.mp4_path);
    probe_display_page(path)
}

async fn secret_display_handler(State(state): State<Arc<GatewayState>>) -> Response {
    let path = state
        .proof_paths
        .read()
        .expect("proof paths poisoned")
        .secret_path
        .clone();
    video_page(path, "R001 secret-boundary fixture")
}

fn video_page(path: Option<String>, title: &str) -> Response {
    let Some(path) = path else {
        return (StatusCode::NOT_FOUND, "proof media is not configured").into_response();
    };
    Html(format!(
        "<!doctype html><meta charset=\"utf-8\"><title>{title}</title><video id=\"player\" controls muted preload=\"auto\" src=\"{path}\"></video>"
    ))
    .into_response()
}

fn probe_display_page(path: Option<String>) -> Response {
    let Some(path) = path else {
        return (StatusCode::NOT_FOUND, "probe media is not configured").into_response();
    };
    Html(DISPLAY_PAGE.replace("__MEDIA_PATH__", &escape_html_attribute(&path))).into_response()
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const DISPLAY_PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
  <title>Web Display Remote Playback Probe</title>
  <style>
    :root { color-scheme: dark; font-family: system-ui,sans-serif; }
    * { box-sizing: border-box; }
    html, body { width:100%; height:100%; margin:0; background:#000; }
    body { overflow:hidden; }
    #display-shell { position:relative; width:100vw; height:100vh; min-height:100dvh; background:#000; }
    #player { width:100%; height:100%; object-fit:contain; background:#000; }
    #overlay { position:absolute; inset:auto 3vw 3vh; max-width:42rem; padding:1rem 1.2rem; border:1px solid #555; border-radius:.7rem; background:#111d; }
    #overlay.playing { opacity:0; pointer-events:none; transition:opacity .4s; }
    #overlay:hover, #overlay:focus-within { opacity:1; pointer-events:auto; }
    h1 { font-size:clamp(1.1rem,2.4vw,2rem); margin:.1rem 0 .4rem; }
    p { margin:.35rem 0; }
    button { min-height:2.8rem; margin:.35rem .35rem .2rem 0; padding:.5rem .8rem; color:#fff; background:#263b62; border:1px solid #8aa9e8; border-radius:.35rem; font:inherit; }
    button:focus-visible { outline:3px solid #ffca55; outline-offset:2px; }
    #diagnostics { max-height:12rem; overflow:auto; white-space:pre-wrap; font-size:.75rem; color:#b8c5d8; }
  </style>
</head>
<body>
  <main id="display-shell" class="viewport-immersive">
    <video id="player" playsinline preload="auto" src="__MEDIA_PATH__"></video>
    <section id="overlay" aria-live="polite">
      <h1>Remote playback ready</h1>
      <p id="status">Waiting for a remote play command.</p>
      <button id="activate" type="button">Press OK to enable remote playback</button>
      <button id="fullscreen" type="button">Try Fullscreen</button>
      <button id="reset" type="button">Reset / reload probe</button>
      <details><summary>Probe diagnostics</summary><pre id="diagnostics">starting…</pre></details>
    </section>
  </main>
  <script>
    (() => {
      const player = document.querySelector('#player');
      const overlay = document.querySelector('#overlay');
      const status = document.querySelector('#status');
      const diagnostics = document.querySelector('#diagnostics');
      const seenCommands = new Set();
      let cursor = 0;
      let transport = 'disconnected';

      const safeError = error => ({
        name: String(error?.name || 'UnknownError').slice(0, 96),
        message: String(error?.message || 'No browser error message').replace(/https?:\/\/[^\s]+/g, '[url-redacted]').slice(0, 256)
      });
      const attemptId = prefix => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
      const send = payload => fetch('/api/v1/display-probe/telemetry', {
        method: 'POST', headers: {'content-type': 'application/json'}, body: JSON.stringify(payload), keepalive: true
      }).catch(() => {});
      const show = text => {
        status.textContent = text;
        diagnostics.textContent = `${new Date().toISOString()} ${text}\n` + diagnostics.textContent.slice(0, 1800);
      };

      async function playOnce(source, commandId = null) {
        const id = attemptId(source);
        if (player.ended) player.currentTime = 0;
        player.muted = false;
        player.volume = 1;
        show(`${source} audible play attempt ${id}…`);
        let result = 'reject';
        let error = {};
        try {
          const playPromise = player.play();
          await playPromise;
          result = 'resolve';
          show(`${source} audible play resolved; muted=${player.muted} volume=${player.volume}`);
        } catch (caught) {
          error = safeError(caught);
          overlay.classList.remove('playing');
          show(`${source} audible play rejected: ${error.name}`);
        }
        await send({
          kind: 'play_attempt', attempt_id: id, command_id: commandId, result,
          error_name: error.name || null, error_message: error.message || null,
          muted: player.muted, volume: player.volume, detail: `source=${source}`
        });
        return result;
      }

      async function poll() {
        try {
          const response = await fetch(`/api/v1/display-probe/events?after=${cursor}`, {cache:'no-store'});
          if (!response.ok) throw new Error(`probe events HTTP ${response.status}`);
          const payload = await response.json();
          if (payload.truncated) {
            const snapshot = await (await fetch('/api/v1/display-probe/state', {cache:'no-store'})).json();
            cursor = snapshot.cursor;
          } else {
            cursor = payload.cursor;
            for (const event of payload.events) {
              if (event.kind === 'play' && !seenCommands.has(event.sequence)) {
                seenCommands.add(event.sequence);
                await playOnce('remote', event.request_id);
              }
            }
          }
          if (transport !== 'connected') {
            transport = 'connected';
            send({kind:'transport', result:'connected', detail:'remote event polling connected'});
            show('Remote event connection ready.');
          }
        } catch (error) {
          if (transport !== 'reconnecting') {
            transport = 'reconnecting';
            send({kind:'transport', result:'reconnecting', error_name:safeError(error).name, error_message:safeError(error).message});
          }
          show('Remote event connection retrying…');
        }
      }

      document.querySelector('#activate').addEventListener('click', async () => {
        const result = await playOnce('activation');
        if (result === 'resolve') {
          show('Activation completed. Remote playback may now be retried.');
          player.pause();
        }
      });
      document.querySelector('#fullscreen').addEventListener('click', async () => {
        if (!document.documentElement.requestFullscreen) {
          send({kind:'fullscreen', result:'unavailable', detail:'requestFullscreen is not exposed'});
          show('Fullscreen unavailable; viewport immersive mode remains active.');
          return;
        }
        try {
          await document.documentElement.requestFullscreen();
          send({kind:'fullscreen', result:'resolve'});
          show('Fullscreen enabled.');
        } catch (error) {
          const safe = safeError(error);
          send({kind:'fullscreen', result:'reject', error_name:safe.name, error_message:safe.message});
          show(`Fullscreen rejected (${safe.name}); viewport immersive mode remains active.`);
        }
      });
      document.querySelector('#reset').addEventListener('click', async () => {
        await fetch('/api/v1/display-probe/reset', {method:'POST'}).catch(() => {});
        window.location.reload();
      });
      player.addEventListener('ended', () => { overlay.classList.remove('playing'); send({kind:'media', result:'ended'}); show('Playback ended; waiting for another remote command.'); });
      player.addEventListener('error', () => { overlay.classList.remove('playing'); send({kind:'media', result:'error', detail:'HTMLMediaElement error'}); show('Media element reported an error.'); });
      document.addEventListener('visibilitychange', () => send({kind:'visibility', result:document.visibilityState}));
      window.addEventListener('pageshow', () => send({kind:'lifecycle', result:'pageshow'}));
      window.addEventListener('pagehide', () => send({kind:'lifecycle', result:'pagehide'}));
      window.addEventListener('online', () => { transport = 'reconnecting'; send({kind:'network', result:'online'}); });
      window.addEventListener('offline', () => { transport = 'reconnecting'; send({kind:'network', result:'offline'}); });
      document.addEventListener('fullscreenchange', () => send({kind:'fullscreen', result:document.fullscreenElement ? 'active' : 'inactive'}));
      document.addEventListener('keydown', event => {
        if (event.key === 'Enter' || event.key === 'OK') document.querySelector('#activate').click();
      });
      window.__r002Probe = { poll, playOnce, getCursor: () => cursor };
      (async () => {
        const snapshot = await (await fetch('/api/v1/display-probe/state', {cache:'no-store'})).json();
        cursor = snapshot.cursor;
        send({kind:'lifecycle', result:'ready', detail:`cursor=${cursor}`});
        await poll();
        window.setInterval(poll, 1000);
      })().catch(error => show(`Probe initialization failed: ${safeError(error).name}`));
    })();
  </script>
</body>
</html>"##;

const CONTROL_PAGE: &str = r##"<!doctype html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Remote Playback Control</title>
<style>body{font:1.1rem system-ui;max-width:38rem;margin:2rem auto;padding:1rem}button{font:inherit;padding:.8rem 1rem;margin:.4rem 0}pre{white-space:pre-wrap;background:#eee;padding:1rem}</style>
<h1>Remote Playback Control</h1>
<p>Send one audible play attempt to the Web Display probe.</p>
<button id="play" type="button">Play on Display</button>
<button id="reset" type="button">Reset probe diagnostics</button>
<pre id="result" aria-live="polite">Ready.</pre>
<script>
  const result = document.querySelector('#result');
  document.querySelector('#play').onclick = async () => {
    const request_id = `control-${Date.now()}-${Math.random().toString(36).slice(2,8)}`;
    const response = await fetch('/api/v1/display-probe/commands', {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({request_id})});
    result.textContent = JSON.stringify(await response.json(), null, 2);
  };
  document.querySelector('#reset').onclick = async () => {
    const response = await fetch('/api/v1/display-probe/reset', {method:'POST'});
    result.textContent = JSON.stringify(await response.json(), null, 2);
  };
</script>"##;

async fn control_handler() -> Response {
    Html(CONTROL_PAGE.to_string()).into_response()
}

async fn probe_state_handler(State(state): State<Arc<GatewayState>>) -> Json<ProbeStateSnapshot> {
    Json(state.probe.state_snapshot())
}

async fn probe_events_handler(
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<ProbeEventsQuery>,
) -> Json<ProbeEventsResponse> {
    Json(state.probe.events_after(query.after.unwrap_or(0)))
}

async fn probe_command_handler(
    State(state): State<Arc<GatewayState>>,
    Json(input): Json<PlayCommandRequest>,
) -> Response {
    let request_id = input
        .request_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    if request_id.len() > 128 {
        return (StatusCode::BAD_REQUEST, "request_id_too_long").into_response();
    }
    if !request_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
    {
        return (StatusCode::BAD_REQUEST, "request_id_invalid").into_response();
    }
    let (command, duplicate) = state.probe.issue_play(request_id);
    Json(PlayCommandResponse {
        accepted: true,
        duplicate,
        command,
    })
    .into_response()
}

async fn probe_telemetry_handler(
    State(state): State<Arc<GatewayState>>,
    Json(input): Json<ProbeTelemetryRequest>,
) -> Json<serde_json::Value> {
    let sequence = state.probe.append_telemetry(input);
    Json(serde_json::json!({ "accepted": true, "sequence": sequence }))
}

async fn probe_reset_handler(State(state): State<Arc<GatewayState>>) -> Json<serde_json::Value> {
    state.probe.reset();
    Json(serde_json::json!({ "reset": true }))
}

async fn fixture_handler(
    State(state): State<Arc<GatewayState>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let authorized = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "Bearer r001-fixture-secret");
    if !authorized {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let path = state
        .fixture_mp4
        .read()
        .expect("fixture path poisoned")
        .clone();
    let Some(path) = path else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    ranged_bytes_response(method, &headers, bytes)
}

fn ranged_bytes_response(method: Method, request_headers: &HeaderMap, bytes: Vec<u8>) -> Response {
    let len = bytes.len();
    let mut status = StatusCode::OK;
    let mut body = bytes;
    let mut content_range = None;
    if let Some(range) = request_headers
        .get(RANGE)
        .and_then(|value| value.to_str().ok())
    {
        if let Some((start, end)) = parse_single_range(range, len) {
            status = StatusCode::PARTIAL_CONTENT;
            content_range = Some(format!("bytes {start}-{end}/{len}"));
            body = body[start..=end].to_vec();
        } else {
            return StatusCode::RANGE_NOT_SATISFIABLE.into_response();
        }
    }
    let body_len = body.len();
    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "video/mp4")
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, body_len.to_string());
    if let Some(value) = content_range {
        builder = builder.header(CONTENT_RANGE, value);
    }
    builder
        .body(if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("fixture response")
}

fn parse_single_range(value: &str, len: usize) -> Option<(usize, usize)> {
    let range = value.strip_prefix("bytes=")?;
    if range.contains(',') {
        return None;
    }
    let (start, end) = range.split_once('-')?;
    let start: usize = start.parse().ok()?;
    if start >= len {
        return None;
    }
    let end = if end.is_empty() {
        len - 1
    } else {
        end.parse::<usize>().ok()?.min(len - 1)
    };
    (start <= end).then_some((start, end))
}

async fn stream_handler(
    State(state): State<Arc<GatewayState>>,
    Path((token, session, item, revision, resource)): Path<(String, String, String, u64, String)>,
    method: Method,
    request_headers: HeaderMap,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let binding = Binding::new(&session, &item, revision, &resource);
    let record = match state.store.get(&token, &binding) {
        Ok(record) => record,
        Err(CapabilityError::NotFound) => {
            return (StatusCode::NOT_FOUND, "INVALID_MEDIA_CAPABILITY").into_response();
        }
        Err(CapabilityError::Expired) => {
            return (StatusCode::GONE, "EXPIRED_MEDIA_CAPABILITY").into_response();
        }
        Err(CapabilityError::BindingMismatch) => {
            return (StatusCode::FORBIDDEN, "MEDIA_CAPABILITY_BINDING_MISMATCH").into_response();
        }
    };

    let (upstream, final_url) =
        match fetch_upstream(&state, &record, &method, &request_headers).await {
            Ok(value) => value,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };

    if request_headers.contains_key(RANGE) && upstream.status() == StatusCode::OK {
        return (StatusCode::BAD_GATEWAY, "UPSTREAM_RANGE_UNSUPPORTED").into_response();
    }

    let status = upstream.status();
    let response_headers = filtered_response_headers(upstream.headers());
    if !status.is_success() {
        return response_from_parts(status, response_headers, Body::empty());
    }
    if method == Method::HEAD {
        return response_from_parts(status, response_headers, Body::empty());
    }

    if record.resource.protocol == StreamProtocol::Hls
        && is_manifest(&final_url, upstream.headers())
    {
        let bytes = match collect_limited(upstream, MAX_MANIFEST_BYTES).await {
            Ok(bytes) => bytes,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };
        let manifest = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return (StatusCode::BAD_GATEWAY, "INVALID_HLS_MANIFEST").into_response(),
        };
        let rewritten = match rewrite_manifest(&state, &record, &final_url, &manifest).await {
            Ok(text) => text,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };
        let mut headers = response_headers;
        headers.remove(CONTENT_LENGTH);
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.apple.mpegurl"),
        );
        return response_from_parts(status, headers, Body::from(rewritten));
    }

    let guard = ActiveGuard::new(state.active_streams.clone());
    let upstream_stream = Box::pin(upstream.bytes_stream());
    let guarded = stream::unfold((upstream_stream, guard), |(mut source, guard)| async move {
        match source.next().await {
            Some(Ok(chunk)) => Some((Ok::<Bytes, reqwest::Error>(chunk), (source, guard))),
            Some(Err(error)) => Some((Err(error), (source, guard))),
            None => None,
        }
    });
    response_from_parts(status, response_headers, Body::from_stream(guarded))
}

async fn fetch_upstream(
    state: &GatewayState,
    record: &CapabilityRecord,
    method: &Method,
    request_headers: &HeaderMap,
) -> Result<(reqwest::Response, Url), &'static str> {
    let mut url = record.resource.url.clone();
    for hop in 0..=MAX_REDIRECTS {
        let egress_policy = state
            .egress_policy
            .read()
            .expect("egress policy poisoned")
            .clone();
        let validated_target = egress_policy
            .validate_and_resolve(&url, &record.resource.egress_scope)
            .await
            .map_err(egress_error_code)?;
        let client = validated_target
            .pinned_client()
            .map_err(|_| "UPSTREAM_CLIENT_FAILED")?;
        let mut request = client.request(method.clone(), url.clone());
        request = request.headers(record.resource.public_headers.clone());
        request = request.headers(record.resource.secret_headers.clone());
        if let Some(value) = request_headers.get(RANGE) {
            request = request.header(RANGE, value);
        }
        if let Some(value) = request_headers.get(IF_RANGE) {
            request = request.header(IF_RANGE, value);
        }
        let response = request
            .send()
            .await
            .map_err(|_| "UPSTREAM_REQUEST_FAILED")?;
        if response.status().is_redirection() {
            if hop == MAX_REDIRECTS {
                return Err("TOO_MANY_REDIRECTS");
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or("INVALID_REDIRECT")?;
            url = url.join(location).map_err(|_| "INVALID_REDIRECT")?;
            continue;
        }
        return Ok((response, url));
    }
    Err("TOO_MANY_REDIRECTS")
}

fn egress_error_code(error: EgressPolicyError) -> &'static str {
    match error {
        EgressPolicyError::InvalidScheme => "EGRESS_SCHEME_REJECTED",
        EgressPolicyError::MissingHost => "EGRESS_HOST_REJECTED",
        EgressPolicyError::InvalidPort => "EGRESS_PORT_REJECTED",
        EgressPolicyError::DnsLookupFailed => "EGRESS_DNS_FAILED",
        EgressPolicyError::TargetRejected
        | EgressPolicyError::LocalServiceOriginMismatch
        | EgressPolicyError::LocalServiceNotConfigured => "EGRESS_TARGET_REJECTED",
    }
}

fn is_manifest(url: &Url, headers: &HeaderMap) -> bool {
    if url.path().to_ascii_lowercase().ends_with(".m3u8") {
        return true;
    }
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("mpegurl") || value.contains("m3u"))
}

async fn collect_limited(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    if response
        .content_length()
        .is_some_and(|length| length as usize > limit)
    {
        return Err("HLS_MANIFEST_TOO_LARGE");
    }
    let mut output = Vec::new();
    let mut source = response.bytes_stream();
    while let Some(chunk) = source.next().await {
        let chunk = chunk.map_err(|_| "HLS_MANIFEST_INTERRUPTED")?;
        if output.len() + chunk.len() > limit {
            return Err("HLS_MANIFEST_TOO_LARGE");
        }
        output.extend_from_slice(&chunk);
    }
    Ok(output)
}

async fn rewrite_manifest(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    manifest: &str,
) -> Result<String, &'static str> {
    let mut output = String::with_capacity(manifest.len() + 128);
    for line in manifest.lines() {
        let rewritten = if line.trim().is_empty() {
            String::new()
        } else if line.starts_with('#') {
            rewrite_uri_attribute(state, parent, base, line).await?
        } else {
            issue_hls_child(state, parent, base, line.trim()).await?
        };
        output.push_str(&rewritten);
        output.push('\n');
    }
    Ok(output)
}

async fn rewrite_uri_attribute(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    line: &str,
) -> Result<String, &'static str> {
    let Some(start) = line.find("URI=\"") else {
        return Ok(line.to_string());
    };
    let value_start = start + 5;
    let rest = &line[value_start..];
    let Some(end_relative) = rest.find('"') else {
        return Err("INVALID_HLS_URI_ATTRIBUTE");
    };
    let value_end = value_start + end_relative;
    let rewritten = issue_hls_child(state, parent, base, &line[value_start..value_end]).await?;
    Ok(format!(
        "{}{}{}",
        &line[..value_start],
        rewritten,
        &line[value_end..]
    ))
}

async fn issue_hls_child(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    target: &str,
) -> Result<String, &'static str> {
    let url = base.join(target).map_err(|_| "INVALID_HLS_CHILD_URI")?;
    let egress_policy = state
        .egress_policy
        .read()
        .expect("egress policy poisoned")
        .clone();
    egress_policy
        .validate(&url, &parent.resource.egress_scope)
        .await
        .map_err(egress_error_code)?;
    let protocol = if url.path().to_ascii_lowercase().ends_with(".m3u8") {
        StreamProtocol::Hls
    } else {
        StreamProtocol::HttpFile
    };
    let child_id = format!(
        "hls-{}",
        state.store.sequence.fetch_add(1, Ordering::Relaxed)
    );
    let binding = Binding::new(
        &parent.binding.session_id,
        &parent.binding.item_id,
        parent.binding.item_revision,
        &child_id,
    );
    let ttl = parent.expires_at.saturating_duration_since(Instant::now());
    if ttl.is_zero() {
        return Err("EXPIRED_MEDIA_CAPABILITY");
    }
    let token = state.store.issue(
        binding.clone(),
        UpstreamResource {
            url,
            protocol,
            public_headers: parent.resource.public_headers.clone(),
            secret_headers: parent.resource.secret_headers.clone(),
            egress_scope: parent.resource.egress_scope.clone(),
        },
        ttl,
    );
    Ok(stream_path(&token, &binding))
}

fn filtered_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut output = HeaderMap::new();
    for name in [
        CONTENT_TYPE,
        CONTENT_LENGTH,
        CONTENT_RANGE,
        ACCEPT_RANGES,
        ETAG,
        LAST_MODIFIED,
        CACHE_CONTROL,
    ] {
        if let Some(value) = headers.get(&name) {
            output.insert(name, value.clone());
        }
    }
    output
}

fn response_from_parts(status: StatusCode, headers: HeaderMap, body: Body) -> Response {
    let mut response = Response::builder()
        .status(status)
        .body(body)
        .expect("response");
    *response.headers_mut() = headers;
    response
}

struct ActiveGuard(Arc<AtomicUsize>);

impl ActiveGuard {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self(counter)
    }
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn resolved_secret_headers_are_rejected() {
        let stream = ResolvedStream {
            id: "x".into(),
            protocol: StreamProtocol::HttpFile,
            url: Url::parse("https://example.com/a.mp4").unwrap(),
            public_headers: BTreeMap::from([("Authorization".into(), "Bearer leak".into())]),
            upstream_access_ref: None,
        };
        assert!(matches!(
            GatewayService::resource_from_resolved(&stream, EgressScope::PublicWeb),
            Err(GatewayError::SecretHeader)
        ));
    }

    #[test]
    fn bearer_like_values_and_secret_header_names_are_rejected() {
        for header in [
            ("x-trace", "Bearer hidden-secret"),
            ("Cookie", "session=hidden-secret"),
            ("X-Api-Key", "hidden-secret"),
        ] {
            let stream = ResolvedStream {
                id: "x".into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://example.com/a.mp4").unwrap(),
                public_headers: BTreeMap::from([(header.0.into(), header.1.into())]),
                upstream_access_ref: None,
            };
            assert!(matches!(
                GatewayService::resource_from_resolved(&stream, EgressScope::PublicWeb),
                Err(GatewayError::SecretHeader)
            ));
        }
    }

    #[test]
    fn single_range_parser_rejects_multi_range() {
        assert_eq!(parse_single_range("bytes=2-5", 10), Some((2, 5)));
        assert_eq!(parse_single_range("bytes=2-", 10), Some((2, 9)));
        assert_eq!(parse_single_range("bytes=0-1,4-5", 10), None);
    }

    #[test]
    fn probe_request_id_is_idempotent_and_cursor_is_replayable() {
        let probe = ProbeStore::default();
        let (first, duplicate) = probe.issue_play("case-a-1".into());
        assert!(!duplicate);
        let (second, duplicate) = probe.issue_play("case-a-1".into());
        assert!(duplicate);
        assert_eq!(first.sequence, second.sequence);
        assert_eq!(probe.events_after(0).events.len(), 1);
        assert_eq!(probe.events_after(first.sequence).events.len(), 0);
    }

    #[test]
    fn probe_telemetry_is_bounded_and_redacts_header_like_text() {
        let probe = ProbeStore::default();
        let sequence = probe.append_telemetry(ProbeTelemetryRequest {
            kind: "play_attempt".into(),
            command_id: Some("case-a-1".into()),
            attempt_id: Some("attempt-1".into()),
            result: Some("reject".into()),
            error_name: Some("NotAllowedError".into()),
            error_message: Some("Cookie Bearer secret".into()),
            muted: Some(false),
            volume: Some(1.0),
            detail: None,
        });
        assert_eq!(sequence, 1);
        let snapshot = probe.state_snapshot();
        assert_eq!(snapshot.telemetry.len(), 1);
        assert!(
            !snapshot.telemetry[0]
                .error_message
                .as_deref()
                .unwrap()
                .contains("Bearer secret")
        );
    }
}
pub mod playback;
