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
        for (name, value) in response.headers() {
            let Ok(value) = value.to_str() else {
                return BrokerResponse::denied("BROKER_RESPONSE_HEADER_REJECTED");
            };
            if headers.len() >= MAX_HEADERS
                || name.as_str().len() > MAX_HEADER_NAME_BYTES
                || value.len() > MAX_HEADER_VALUE_BYTES
                || is_secret_header(name.as_str(), value)
            {
                return BrokerResponse::denied("BROKER_RESPONSE_SECRET_REJECTED");
            }
            headers.insert(name.as_str().into(), value.into());
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
    use super::{BrokerAbort, BrokerCancellation, await_broker_stage};
    use std::future::pending;
    use std::time::Duration;

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
}
