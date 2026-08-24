use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{
    ACCEPT_RANGES, AUTHORIZATION, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
    IF_RANGE, LAST_MODIFIED, RANGE,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use serde::Serialize;
use site_adapter_api::{ResolvedStream, StreamProtocol};
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::net::lookup_host;
use url::Url;
use uuid::Uuid;

const MAX_MANIFEST_BYTES: usize = 512 * 1024;
const MAX_REDIRECTS: usize = 5;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EgressScope {
    PublicWeb,
    FixtureLoopback,
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
    pub hls_path: Option<String>,
    pub secret_path: Option<String>,
    pub chain: String,
}

#[derive(Clone)]
struct GatewayState {
    store: Arc<CapabilityStore>,
    client: reqwest::Client,
    active_streams: Arc<AtomicUsize>,
    proof_paths: Arc<RwLock<ProofPaths>>,
    fixture_mp4: Arc<RwLock<Option<PathBuf>>>,
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

impl GatewayService {
    pub fn new(max_capabilities: usize) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client");
        Self {
            state: Arc::new(GatewayState {
                store: Arc::new(CapabilityStore::new(max_capabilities)),
                client,
                active_streams: Arc::new(AtomicUsize::new(0)),
                proof_paths: Arc::new(RwLock::new(ProofPaths {
                    chain: "SiteAdapterRegistry -> generic-direct -> ResolvedMedia -> MediaGateway -> WebDisplay".into(),
                    ..ProofPaths::default()
                })),
                fixture_mp4: Arc::new(RwLock::new(None)),
            }),
        }
    }

    pub fn resource_from_resolved(
        stream: &ResolvedStream,
        scope: EgressScope,
    ) -> Result<UpstreamResource, GatewayError> {
        let mut public_headers = HeaderMap::new();
        for (name, value) in &stream.public_headers {
            let lower = name.to_ascii_lowercase();
            if matches!(
                lower.as_str(),
                "cookie" | "authorization" | "proxy-authorization"
            ) {
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
            .route("/secret-display", get(secret_display_handler))
            .route(
                "/fixture/protected.mp4",
                get(fixture_handler).head(fixture_handler),
            )
            .route("/healthz", get(|| async { "ok" }))
            .with_state(self.state.clone())
    }
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
    let path = state
        .proof_paths
        .read()
        .expect("proof paths poisoned")
        .mp4_path
        .clone();
    video_page(path, "R001 public MP4 proof")
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
        validate_egress(&url, record.resource.egress_scope).await?;
        let mut request = state.client.request(method.clone(), url.clone());
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

async fn validate_egress(url: &Url, scope: EgressScope) -> Result<(), &'static str> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("EGRESS_SCHEME_REJECTED");
    }
    let host = url.host_str().ok_or("EGRESS_HOST_REJECTED")?;
    match scope {
        EgressScope::FixtureLoopback => {
            let is_loopback = host.eq_ignore_ascii_case("localhost")
                || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback());
            if !is_loopback {
                return Err("EGRESS_TARGET_REJECTED");
            }
        }
        EgressScope::PublicWeb => {
            let port = url.port_or_known_default().ok_or("EGRESS_PORT_REJECTED")?;
            if let Ok(ip) = host.parse::<IpAddr>() {
                if !is_public_ip(ip) {
                    return Err("EGRESS_TARGET_REJECTED");
                }
            } else {
                let addresses: Vec<_> = lookup_host((host, port))
                    .await
                    .map_err(|_| "EGRESS_DNS_FAILED")?
                    .map(|address| address.ip())
                    .collect();
                if addresses.is_empty() || addresses.iter().any(|ip| !is_public_ip(*ip)) {
                    return Err("EGRESS_TARGET_REJECTED");
                }
            }
        }
    }
    Ok(())
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => is_public_v6(ip),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || octets[0] == 0
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
        || octets[0] >= 240)
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (segments[0] & 0xfe00) == 0xfc00
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] == 0x2001 && segments[1] == 0x0db8))
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
    validate_egress(&url, parent.resource.egress_scope).await?;
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
            egress_scope: parent.resource.egress_scope,
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
    fn single_range_parser_rejects_multi_range() {
        assert_eq!(parse_single_range("bytes=2-5", 10), Some((2, 5)));
        assert_eq!(parse_single_range("bytes=2-", 10), Some((2, 9)));
        assert_eq!(parse_single_range("bytes=0-1,4-5", 10), None);
    }
}
