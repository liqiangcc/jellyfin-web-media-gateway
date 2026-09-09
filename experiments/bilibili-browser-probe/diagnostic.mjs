/**
 * Bounded, schema-safe failure classification for the experimental probe.
 *
 * This module deliberately consumes only coarse error fields.  It never
 * returns an Error message, URL, host, address, header, certificate, or body.
 */

export const DIAGNOSTIC_SCHEMA_VERSION = 2;
export const DIAGNOSTIC_PHASES = Object.freeze([
  'dns_address_policy',
  'broker_connect',
  'tls_handshake',
  'proxy_response',
  'chromium_navigation',
  'http_status',
  'unknown',
]);

// Transport stages are deliberately finite.  They describe the first
// boundary at which the broker can make a bounded observation; they never
// expose an address, certificate, URL, or network error text.
export const TRANSPORT_STAGES = Object.freeze([
  'resolve_policy',
  'tcp_connect',
  'tls_handshake',
  'proxy_response',
  'downstream_close',
  'unknown',
]);
export const TRANSPORT_OUTCOMES = Object.freeze(['success', 'failure', 'unknown']);
export const LIFECYCLE_OUTCOMES = Object.freeze([
  'fulfilled', 'rejected', 'timeout', 'aborted', 'page_closed', 'page_crashed',
  'browser_disconnected', 'process_error', 'process_signal', 'unknown',
]);

const MAX_REQUESTS = 200;
const MAX_RESPONSE_BYTES = 32 * 1024 * 1024;
const MAX_METADATA_BYTES = 1024 * 1024;

const ERROR_CODES = Object.freeze([
  ['ERR_HTTP2_PROTOCOL_ERROR', 'chromium_navigation', 'http2_protocol_error', 'downstream_close'],
  ['ERR_HTTP2_STREAM_ERROR', 'chromium_navigation', 'http2_stream_error', 'downstream_close'],
  ['ERR_HTTP2_GOAWAY_SESSION', 'chromium_navigation', 'http2_session_closed', 'downstream_close'],
  ['ERR_SSL_PROTOCOL_ERROR', 'tls_handshake', 'tls_protocol_error', 'tls_handshake'],
  ['ERR_TLS_CERT_ALTNAME_INVALID', 'tls_handshake', 'tls_certificate_error', 'tls_handshake'],
  ['CERT_HAS_EXPIRED', 'tls_handshake', 'tls_certificate_error', 'tls_handshake'],
  ['UNABLE_TO_VERIFY_LEAF_SIGNATURE', 'tls_handshake', 'tls_certificate_error', 'tls_handshake'],
  ['ERR_TLS_HANDSHAKE_TIMEOUT', 'tls_handshake', 'tls_timeout', 'tls_handshake'],
  ['ERR_DNS_ADDRESS_POLICY', 'dns_address_policy', 'address_policy_denied', 'resolve_policy'],
  ['ERR_DNS_PIN_CHANGED', 'dns_address_policy', 'address_pin_changed', 'resolve_policy'],
  ['EAI_AGAIN', 'dns_address_policy', 'dns_lookup_failed', 'resolve_policy'],
  ['EAI_NONAME', 'dns_address_policy', 'dns_lookup_failed', 'resolve_policy'],
  ['ENOTFOUND', 'dns_address_policy', 'dns_lookup_failed', 'resolve_policy'],
  ['BROKER_CONNECT', 'broker_connect', 'broker_connect_failed', 'unknown'],
  ['ERR_CONNECTION_CLOSED', 'chromium_navigation', 'connection_closed', 'downstream_close'],
  ['ERR_CONNECTION_ABORTED', 'chromium_navigation', 'connection_aborted', 'downstream_close'],
  ['ERR_CONNECTION_RESET', 'chromium_navigation', 'connection_reset', 'tcp_connect'],
  ['ERR_CONNECTION_REFUSED', 'chromium_navigation', 'connection_refused', 'tcp_connect'],
  ['ERR_CONNECTION_TIMED_OUT', 'chromium_navigation', 'connection_timeout', 'tcp_connect'],
  ['ECONNREFUSED', 'broker_connect', 'connection_refused', 'tcp_connect'],
  ['ECONNRESET', 'broker_connect', 'connection_reset', 'tcp_connect'],
  ['ETIMEDOUT', 'broker_connect', 'connection_timeout', 'tcp_connect'],
  ['ERR_PROXY_CONNECTION_FAILED', 'proxy_response', 'proxy_connection_failed', 'proxy_response'],
  ['ERR_TUNNEL_CONNECTION_FAILED', 'proxy_response', 'proxy_tunnel_failed', 'proxy_response'],
  ['PROXY_RESPONSE', 'proxy_response', 'proxy_response_invalid', 'proxy_response'],
  ['ERR_HTTP_RESPONSE_CODE_FAILURE', 'http_status', 'http_status_error', 'downstream_close'],
  ['NAVIGATION_TIMEOUT', 'chromium_navigation', 'navigation_timeout', 'downstream_close'],
  ['ERR_ABORTED', 'chromium_navigation', 'navigation_aborted', 'downstream_close'],
]);

// Durable diagnostic reasons are a closed vocabulary.  This is exported so
// the finalizer can reject caller-supplied labels instead of treating a
// merely lexical string as trusted evidence.
export const DIAGNOSTIC_REASONS = Object.freeze([
  ...new Set([
    ...ERROR_CODES.map(([, , reason]) => reason),
    ...DIAGNOSTIC_PHASES.filter((phase) => phase !== 'unknown').map((phase) => `${phase}_failed`),
    'navigation_promise_rejected',
    'navigation_target_closed',
    'navigation_target_crashed',
    'navigation_browser_disconnected',
    'unclassified_failure',
    'status_1xx', 'status_2xx', 'status_3xx', 'status_4xx', 'status_5xx',
  ]),
]);
const DIAGNOSTIC_REASON_SET = new Set(DIAGNOSTIC_REASONS);

const PHASE_HINTS = new Set(DIAGNOSTIC_PHASES);
const TRANSPORT_STAGE_SET = new Set(TRANSPORT_STAGES);
const TRANSPORT_OUTCOME_SET = new Set(TRANSPORT_OUTCOMES);
const LIFECYCLE_OUTCOME_SET = new Set(LIFECYCLE_OUTCOMES);

function boundedCounter(value, maximum) {
  return Number.isSafeInteger(value) && value >= 0 && value <= maximum ? value : 0;
}

function statusClass(status) {
  if (!Number.isInteger(status) || status < 100 || status > 599) return 'unknown';
  return `${Math.floor(status / 100)}xx`;
}

function codeMatch(value) {
  if (typeof value !== 'string' || value.length === 0 || value.length > 128) return undefined;
  const normalized = value.toUpperCase();
  return ERROR_CODES.find(([marker]) => {
    if (normalized === marker) return true;
    // Chromium emits markers as `net::ERR_*`; Node emits them after a
    // whitespace boundary.  Avoid matching arbitrary query/header text or
    // identifiers such as NOT_ERR_* supplied by an untrusted caller.
    const escaped = marker.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return new RegExp(`(?:^|NET::|\\s)${escaped}(?:$|\\s|[^A-Z0-9_])`).test(normalized);
  });
}

function hintMatch(value) {
  return typeof value === 'string' && PHASE_HINTS.has(value) ? value : undefined;
}

function transportStage(value) {
  return typeof value === 'string' && TRANSPORT_STAGE_SET.has(value) ? value : undefined;
}

function transportOutcome(value) {
  return typeof value === 'string' && TRANSPORT_OUTCOME_SET.has(value) ? value : undefined;
}

function lifecycleOutcome(value) {
  return typeof value === 'string' && LIFECYCLE_OUTCOME_SET.has(value) ? value : undefined;
}

/**
 * Reduce Playwright/Node lifecycle signals to a closed vocabulary. Only
 * fixed name/code markers are inspected; error messages never leave the
 * process and are not used for classification.
 */
export function classifyNavigationLifecycle(error, observed) {
  if (lifecycleOutcome(observed)) return observed;
  const name = typeof error?.name === 'string' ? error.name : '';
  const code = typeof error?.code === 'string' ? error.code.toUpperCase() : '';
  if (name === 'TimeoutError' || code === 'ETIMEDOUT' || code === 'NAVIGATION_TIMEOUT') return 'timeout';
  if (name === 'AbortError' || code === 'ERR_ABORTED') return 'aborted';
  if (name === 'BrowserDisconnectedError' || name === 'TargetClosedError' || code === 'BROWSER_DISCONNECTED') return 'browser_disconnected';
  if (name === 'PageClosedError' || code === 'PAGE_CLOSED') return 'page_closed';
  if (name === 'PageCrashedError' || code === 'PAGE_CRASHED') return 'page_crashed';
  return error ? 'rejected' : 'unknown';
}

export function classifyProcessTermination(hint) {
  if (hint === 'signal') return 'process_signal';
  if (hint === 'error' || hint === 'timeout' || hint === 'abort') return 'process_error';
  return 'unknown';
}

function stageFor(phase, match, explicit) {
  return transportStage(explicit) || match?.[3] || ({
    dns_address_policy: 'resolve_policy',
    tls_handshake: 'tls_handshake',
    proxy_response: 'proxy_response',
    chromium_navigation: 'downstream_close',
    http_status: 'downstream_close',
  }[phase] || 'unknown');
}

const NAVIGATION_LIFECYCLE_REASONS = Object.freeze({
  rejected: 'navigation_promise_rejected',
  timeout: 'navigation_timeout',
  aborted: 'navigation_aborted',
  page_closed: 'navigation_target_closed',
  page_crashed: 'navigation_target_crashed',
  browser_disconnected: 'navigation_browser_disconnected',
});

/**
 * Classify a coarse failure DTO.  Unknown fields are ignored and only
 * allowlisted codes become output, so passing an Error message cannot leak it.
 */
export function classifyFailure(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  const match = codeMatch(value.error_code) || codeMatch(value.code);
  const phase = match?.[1] || hintMatch(value.phase_hint) || (Number.isInteger(value.status) && value.status >= 300 ? 'http_status' : 'unknown');
  const lifecycle = lifecycleOutcome(value.lifecycle_outcome);
  const reason = match?.[2] || (phase === 'chromium_navigation' && lifecycle ? NAVIGATION_LIFECYCLE_REASONS[lifecycle] : undefined) ||
    (phase === 'http_status' ? `status_${statusClass(value.status)}` : phase === 'unknown' ? 'unclassified_failure' : `${phase}_failed`);
  return Object.freeze({
    schema_version: DIAGNOSTIC_SCHEMA_VERSION,
    phase,
    reason,
    status_class: statusClass(value.status),
    transport_stage: stageFor(phase, match, value.transport_stage),
    transport_outcome: transportOutcome(value.transport_outcome) || 'failure',
    request_count: boundedCounter(value.request_count, MAX_REQUESTS),
    response_bytes: boundedCounter(value.response_bytes, MAX_RESPONSE_BYTES),
    metadata_bytes: boundedCounter(value.metadata_bytes, MAX_METADATA_BYTES),
  });
}

/**
 * Inspect an Error only for fixed markers; the returned value remains safe for
 * durable evidence.  The original message is never copied to the result.
 */
export function classifyError(error, phaseHint, counters = {}) {
  const code = error && typeof error.code === 'string' ? error.code : '';
  const message = error && typeof error.message === 'string' ? error.message : '';
  const observedLifecycle = lifecycleOutcome(counters.lifecycle_outcome) ||
    (phaseHint === 'chromium_navigation' ? classifyNavigationLifecycle(error) : undefined);
  return classifyFailure({
    phase_hint: phaseHint,
    error_code: code || message,
    request_count: counters.request_count,
    response_bytes: counters.response_bytes,
    metadata_bytes: counters.metadata_bytes,
    status: counters.status,
    transport_stage: counters.transport_stage,
    lifecycle_outcome: observedLifecycle,
  });
}

/** Create a monotonic, idempotent transport observation for the broker. */
export function classifyTransport(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  const stage = transportStage(value.stage) || 'unknown';
  const outcome = transportOutcome(value.outcome) || 'unknown';
  return Object.freeze({
    schema_version: DIAGNOSTIC_SCHEMA_VERSION,
    transport_stage: stage,
    transport_outcome: outcome,
    status_class: statusClass(value.status),
    request_count: boundedCounter(value.request_count, MAX_REQUESTS),
    response_bytes: boundedCounter(value.response_bytes, MAX_RESPONSE_BYTES),
    metadata_bytes: boundedCounter(value.metadata_bytes, MAX_METADATA_BYTES),
  });
}

export function diagnosticCounters(state = {}) {
  return {
    request_count: boundedCounter(state.requests, MAX_REQUESTS),
    response_bytes: boundedCounter(state.responseBytes, MAX_RESPONSE_BYTES),
    metadata_bytes: boundedCounter(state.metadataBytes, MAX_METADATA_BYTES),
  };
}

export function isDiagnosticReason(value) {
  return typeof value === 'string' && DIAGNOSTIC_REASON_SET.has(value);
}
