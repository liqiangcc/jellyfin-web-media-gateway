//! Site-neutral broker capability owned by the Gateway execution layer.
//!
//! Concrete Site Plugins consume this abstraction; they do not own or
//! reimplement the central R008 egress and HTTP authority.

use futures_util::StreamExt;
use gateway_core::{EgressPolicy, EgressScope};
use serde::{Deserialize, Serialize};
use site_adapter_api::security::is_secret_header;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use url::Url;

pub const MAX_BODY_BYTES: usize = 96 * 1024;
pub const MAX_HEADERS: usize = 32;
pub const MAX_HEADER_NAME_BYTES: usize = 128;
pub const MAX_HEADER_VALUE_BYTES: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrokerRequest {
    pub operation: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrokerResponse {
    pub status: u16,
    pub reason: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Vec<u8>,
    #[serde(default)]
    pub error: Option<String>,
}

impl BrokerResponse {
    pub fn denied(code: &'static str) -> Self {
        Self {
            status: 400,
            reason: "Bad Request".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            error: Some(code.into()),
        }
    }
}

/// Cancellation shared by the runner and a broker operation.
#[derive(Clone, Default)]
pub struct BrokerCancellation(Arc<AtomicBool>);

impl BrokerCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// The site-neutral capability consumed by a worker runtime.
pub trait BrokerBackend: Send + Sync {
    fn handle(&self, request: BrokerRequest, cancellation: BrokerCancellation) -> BrokerResponse;
}

/// Gateway-owned HTTP(S) broker. Every request goes through the accepted
/// R008 resolver and checked-address pinned client. Redirects are returned,
/// never followed by this capability.
pub struct R008Broker {
    policy: EgressPolicy,
    timeout: Duration,
}

impl R008Broker {
    pub fn new(policy: EgressPolicy, timeout: Duration) -> Self {
        Self { policy, timeout }
    }

    async fn handle_async(
        &self,
        request: BrokerRequest,
        cancellation: &BrokerCancellation,
    ) -> BrokerResponse {
        if request.operation != "http" || !matches!(request.method.as_str(), "GET" | "HEAD") {
            return BrokerResponse::denied("BROKER_OPERATION_REJECTED");
        }
        if request.body.len() > MAX_BODY_BYTES || request.headers.len() > MAX_HEADERS {
            return BrokerResponse::denied("BROKER_REQUEST_TOO_LARGE");
        }
        for (name, value) in &request.headers {
            if name.is_empty()
                || name.len() > MAX_HEADER_NAME_BYTES
                || value.len() > MAX_HEADER_VALUE_BYTES
                || name.chars().any(char::is_control)
                || value.chars().any(char::is_control)
                || is_secret_header(name, value)
                || secretish_name(name)
            {
                return BrokerResponse::denied("BROKER_SECRET_HEADER_REJECTED");
            }
        }

        let Ok(url) = Url::parse(&request.url) else {
            return BrokerResponse::denied("BROKER_URL_REJECTED");
        };
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return BrokerResponse::denied("BROKER_URL_REJECTED");
        }
        if cancellation.is_cancelled() {
            return BrokerResponse::denied("BROKER_CANCELLED");
        }

        // The deadline starts before R008 validation.  In particular, DNS
        // resolution performed by validate_and_resolve is part of the broker
        // operation and must not outlive the worker attempt.
        let deadline = tokio::time::Instant::now() + self.timeout;
        let target = match await_broker_stage(
            self.policy
                .validate_and_resolve(&url, &EgressScope::PublicWeb),
            cancellation,
            deadline,
        )
        .await
        {
            Ok(Ok(target)) => target,
            Ok(Err(_)) => return BrokerResponse::denied("BROKER_EGRESS_REJECTED"),
            Err(BrokerAbort::Cancelled) => return BrokerResponse::denied("BROKER_CANCELLED"),
            Err(BrokerAbort::TimedOut) => return BrokerResponse::denied("BROKER_TIMEOUT"),
        };
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return BrokerResponse::denied("BROKER_TIMEOUT");
        }
        let Ok(client) = target.pinned_client_with_timeout(Some(remaining)) else {
            return BrokerResponse::denied("BROKER_CLIENT_REJECTED");
        };
        let method = match request.method.as_str() {
            "GET" => reqwest::Method::GET,
            "HEAD" => reqwest::Method::HEAD,
            _ => unreachable!(),
        };
        let mut builder = client.request(method, url);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        let send = builder.body(request.body).send();
        tokio::pin!(send);
        let response = tokio::select! {
            response = &mut send => response,
            _ = wait_for_cancel(cancellation.clone()) => {
                return BrokerResponse::denied("BROKER_CANCELLED");
            },
            _ = tokio::time::sleep_until(deadline) => {
                return BrokerResponse::denied("BROKER_TIMEOUT");
            },
        };
        let Ok(response) = response else {
            return if cancellation.is_cancelled() {
                BrokerResponse::denied("BROKER_CANCELLED")
            } else {
                BrokerResponse::denied("BROKER_TRANSPORT_FAILED")
            };
        };
        let status = response.status();
        let reason = status.canonical_reason().unwrap_or("Unknown").to_string();
        let mut headers = BTreeMap::new();
        let mut response_header_count = 0;
        for (name, value) in response.headers() {
            let Ok(value) = value.to_str() else {
                return BrokerResponse::denied("BROKER_RESPONSE_HEADER_REJECTED");
            };
            match admit_response_header(
                &mut headers,
                &mut response_header_count,
                name.as_str(),
                value,
            ) {
                Ok(ResponseHeaderDisposition::Public) => {}
                Ok(ResponseHeaderDisposition::ContainedSecret) => continue,
                Err(ResponseHeaderError::Malformed) => {
                    return BrokerResponse::denied("BROKER_RESPONSE_HEADER_REJECTED");
                }
                Err(ResponseHeaderError::BoundExceeded) => {
                    return BrokerResponse::denied("BROKER_RESPONSE_SECRET_REJECTED");
                }
            }
        }
        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        loop {
            let next = tokio::select! {
                chunk = stream.next() => chunk,
                _ = wait_for_cancel(cancellation.clone()) => {
                    return BrokerResponse::denied("BROKER_CANCELLED");
                },
                _ = tokio::time::sleep_until(deadline) => {
                    return BrokerResponse::denied("BROKER_TIMEOUT");
                },
            };
            let Some(chunk) = next else { break };
            let Ok(chunk) = chunk else {
                return BrokerResponse::denied("BROKER_RESPONSE_READ_FAILED");
            };
            if body.len().saturating_add(chunk.len()) > MAX_BODY_BYTES {
                return BrokerResponse::denied("BROKER_RESPONSE_TOO_LARGE");
            }
            body.extend_from_slice(&chunk);
        }
        BrokerResponse {
            status: status.as_u16(),
            reason,
            headers,
            body,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResponseHeaderDisposition {
    Public,
    ContainedSecret,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResponseHeaderError {
    Malformed,
    BoundExceeded,
}

/// Validate and classify one origin response header before it is copied into
/// the broker IPC response. Secret headers still consume the bounded origin
/// header budget, but their names and values are never admitted to the public
/// map. This keeps response-only Secret material contained without creating a
/// cookie/auth replay capability or an unbounded filtering sink.
fn admit_response_header(
    public_headers: &mut BTreeMap<String, String>,
    total_headers: &mut usize,
    name: &str,
    value: &str,
) -> Result<ResponseHeaderDisposition, ResponseHeaderError> {
    *total_headers = total_headers
        .checked_add(1)
        .ok_or(ResponseHeaderError::BoundExceeded)?;
    if *total_headers > MAX_HEADERS {
        return Err(ResponseHeaderError::BoundExceeded);
    }
    if name.is_empty()
        || name.len() > MAX_HEADER_NAME_BYTES
        || value.len() > MAX_HEADER_VALUE_BYTES
        || name.chars().any(char::is_control)
        || value.chars().any(char::is_control)
    {
        return Err(ResponseHeaderError::Malformed);
    }
    if is_secret_header(name, value) {
        return Ok(ResponseHeaderDisposition::ContainedSecret);
    }
    public_headers.insert(name.into(), value.into());
    Ok(ResponseHeaderDisposition::Public)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrokerAbort {
    Cancelled,
    TimedOut,
}

async fn await_broker_stage<T, E, F>(
    future: F,
    cancellation: &BrokerCancellation,
    deadline: tokio::time::Instant,
) -> Result<Result<T, E>, BrokerAbort>
where
    F: Future<Output = Result<T, E>>,
{
    tokio::pin!(future);
    tokio::select! {
        result = &mut future => Ok(result),
        _ = wait_for_cancel(cancellation.clone()) => Err(BrokerAbort::Cancelled),
        _ = tokio::time::sleep_until(deadline) => Err(BrokerAbort::TimedOut),
    }
}

impl Default for R008Broker {
    fn default() -> Self {
        Self::new(EgressPolicy::default(), Duration::from_secs(10))
    }
}

async fn wait_for_cancel(cancellation: BrokerCancellation) {
    while !cancellation.is_cancelled() {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

impl BrokerBackend for R008Broker {
    fn handle(&self, request: BrokerRequest, cancellation: BrokerCancellation) -> BrokerResponse {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return BrokerResponse::denied("BROKER_RUNTIME_FAILED");
        };
        runtime.block_on(self.handle_async(request, &cancellation))
    }
}

fn secretish_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase().replace('_', "-");
    ["token", "credential", "password", "proxy-auth", "api-key"]
        .iter()
        .any(|needle| name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{
        BrokerAbort, BrokerBackend, BrokerCancellation, BrokerRequest, BrokerResponse, R008Broker,
        ResponseHeaderDisposition, ResponseHeaderError, admit_response_header, await_broker_stage,
    };
    use gateway_core::{EgressDnsResolver, EgressPolicy, EgressResolutionFuture};
    use std::collections::BTreeMap;
    use std::future::pending;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::{Context, Poll};
    use std::thread;
    use std::time::Duration;

    #[derive(Debug)]
    struct PendingResolution {
        dropped: Arc<AtomicBool>,
    }

    impl std::future::Future for PendingResolution {
        type Output = Result<Vec<std::net::SocketAddr>, ()>;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }

    impl Drop for PendingResolution {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::Release);
        }
    }

    #[derive(Debug)]
    struct BlockingResolver {
        started: Arc<AtomicBool>,
        dropped: Arc<AtomicBool>,
    }

    impl EgressDnsResolver for BlockingResolver {
        fn resolve<'a>(&'a self, _: &'a str, _: u16) -> EgressResolutionFuture<'a> {
            self.started.store(true, Ordering::Release);
            Box::pin(PendingResolution {
                dropped: Arc::clone(&self.dropped),
            })
        }
    }

    fn pending_request() -> BrokerRequest {
        BrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "https://resolver-cancellation.example/media".into(),
            headers: Default::default(),
            body: Vec::new(),
        }
    }

    #[test]
    fn response_secret_headers_are_contained_while_public_response_continues() {
        let fixtures = [
            ("Set-Cookie", "fixture-set-cookie-secret"),
            ("WWW-Authenticate", "Basic fixture-basic-secret"),
            ("X-Challenge", "Bearer fixture-bearer-secret"),
            ("X-API-Key", "fixture-api-key-secret"),
            ("Location", "https://public.example/next"),
            ("Content-Type", "application/json"),
        ];
        let mut public_headers = BTreeMap::new();
        let mut total_headers = 0;
        let mut contained = 0;
        for (name, value) in fixtures {
            match admit_response_header(&mut public_headers, &mut total_headers, name, value)
                .expect("synthetic response header fixture should be bounded")
            {
                ResponseHeaderDisposition::Public => {}
                ResponseHeaderDisposition::ContainedSecret => contained += 1,
            }
        }

        assert_eq!(contained, 4);
        assert_eq!(total_headers, 6);
        assert_eq!(
            public_headers,
            BTreeMap::from([
                ("Location".into(), "https://public.example/next".into()),
                ("Content-Type".into(), "application/json".into()),
            ])
        );
        let response = BrokerResponse {
            status: 302,
            reason: "Found".into(),
            headers: public_headers,
            body: b"public-body".to_vec(),
            error: None,
        };
        let wire = serde_json::to_string(&response).unwrap();
        for sentinel in [
            "fixture-set-cookie-secret",
            "fixture-basic-secret",
            "fixture-bearer-secret",
            "fixture-api-key-secret",
        ] {
            assert!(!wire.contains(sentinel), "response leaked {sentinel}");
        }
        assert_eq!(response.body, b"public-body");
        assert!(wire.contains("public.example/next"));
    }

    #[test]
    fn contained_secret_headers_do_not_bypass_response_header_bound() {
        let mut public_headers = BTreeMap::new();
        let mut total_headers = 0;
        for index in 0..super::MAX_HEADERS {
            let name = format!("X-Public-{index}");
            assert_eq!(
                admit_response_header(
                    &mut public_headers,
                    &mut total_headers,
                    &name,
                    "bounded-value",
                ),
                Ok(ResponseHeaderDisposition::Public)
            );
        }
        assert_eq!(
            admit_response_header(
                &mut public_headers,
                &mut total_headers,
                "Set-Cookie",
                "fixture-overflow-secret",
            ),
            Err(ResponseHeaderError::BoundExceeded)
        );
        assert_eq!(total_headers, super::MAX_HEADERS + 1);
    }

    #[test]
    fn malformed_response_headers_fail_closed() {
        let mut public_headers = BTreeMap::new();
        let mut total_headers = 0;
        assert_eq!(
            admit_response_header(
                &mut public_headers,
                &mut total_headers,
                "X-Bad\nHeader",
                "value",
            ),
            Err(ResponseHeaderError::Malformed)
        );
        assert!(public_headers.is_empty());
    }

    #[derive(Debug)]
    struct CountingResolver {
        called: Arc<AtomicBool>,
    }

    impl EgressDnsResolver for CountingResolver {
        fn resolve<'a>(&'a self, _: &'a str, _: u16) -> EgressResolutionFuture<'a> {
            self.called.store(true, Ordering::Release);
            Box::pin(async { Err(()) })
        }
    }

    #[test]
    fn request_secret_material_is_rejected_before_egress() {
        let called = Arc::new(AtomicBool::new(false));
        let broker = R008Broker::new(
            EgressPolicy::with_resolver(Arc::new(CountingResolver {
                called: Arc::clone(&called),
            })),
            Duration::from_secs(1),
        );
        for (name, value) in [
            ("Cookie", "fixture-cookie-secret"),
            ("Authorization", "Bearer fixture-authorization-secret"),
            ("Proxy-Authorization", "Basic fixture-proxy-secret"),
            ("X-API-Key", "fixture-api-key-secret"),
            ("X-Trace", "Basic fixture-basic-secret"),
            ("X-Trace", "Bearer fixture-bearer-secret"),
        ] {
            let response = broker.handle(
                BrokerRequest {
                    operation: "http".into(),
                    method: "GET".into(),
                    url: "https://public.example/media".into(),
                    headers: BTreeMap::from([(name.into(), value.into())]),
                    body: Vec::new(),
                },
                BrokerCancellation::default(),
            );
            assert_eq!(
                response.error.as_deref(),
                Some("BROKER_SECRET_HEADER_REJECTED")
            );
        }
        assert!(!called.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn pre_transport_stage_cancellation_aborts_blocked_resolution() {
        let cancellation = BrokerCancellation::default();
        cancellation.cancel();
        let result = await_broker_stage::<(), (), _>(
            pending(),
            &cancellation,
            tokio::time::Instant::now() + Duration::from_secs(30),
        )
        .await;
        assert_eq!(result, Err(BrokerAbort::Cancelled));
    }

    #[tokio::test]
    async fn pre_transport_stage_deadline_aborts_blocked_resolution() {
        let cancellation = BrokerCancellation::default();
        let result = await_broker_stage::<(), (), _>(
            pending(),
            &cancellation,
            tokio::time::Instant::now() + Duration::from_millis(20),
        )
        .await;
        assert_eq!(result, Err(BrokerAbort::TimedOut));
    }

    #[test]
    fn r008_cancellation_drops_actual_resolver_future_and_joins_broker() {
        let started = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicBool::new(false));
        let policy = EgressPolicy::with_resolver(Arc::new(BlockingResolver {
            started: Arc::clone(&started),
            dropped: Arc::clone(&dropped),
        }));
        let broker = Arc::new(R008Broker::new(policy, Duration::from_secs(30)));
        let cancellation = BrokerCancellation::default();
        let worker_cancel = cancellation.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker_broker = Arc::clone(&broker);
        let worker = thread::spawn(move || {
            let response = worker_broker.handle(pending_request(), worker_cancel);
            sender.send(response).unwrap();
        });

        for _ in 0..100 {
            if started.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(started.load(Ordering::Acquire));
        cancellation.cancel();
        let response = receiver
            .recv_timeout(Duration::from_millis(500))
            .expect("R008 broker cancellation must finish promptly");
        assert_eq!(response.error.as_deref(), Some("BROKER_CANCELLED"));
        worker.join().unwrap();
        assert!(dropped.load(Ordering::Acquire));
    }

    #[test]
    fn r008_deadline_drops_actual_resolver_future_and_joins_broker() {
        let started = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicBool::new(false));
        let policy = EgressPolicy::with_resolver(Arc::new(BlockingResolver {
            started: Arc::clone(&started),
            dropped: Arc::clone(&dropped),
        }));
        let broker = Arc::new(R008Broker::new(policy, Duration::from_millis(25)));
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker_broker = Arc::clone(&broker);
        let worker = thread::spawn(move || {
            let response = worker_broker.handle(pending_request(), BrokerCancellation::default());
            sender.send(response).unwrap();
        });

        assert!(
            receiver
                .recv_timeout(Duration::from_millis(500))
                .expect("R008 broker deadline must finish promptly")
                .error
                .as_deref()
                == Some("BROKER_TIMEOUT")
        );
        worker.join().unwrap();
        assert!(started.load(Ordering::Acquire));
        assert!(dropped.load(Ordering::Acquire));
    }
}
