/**
 * Bounded, schema-safe failure classification for the experimental probe.
 *
 * This module deliberately consumes only coarse error fields.  It never
 * returns an Error message, URL, host, address, header, certificate, or body.
 */

export const DIAGNOSTIC_SCHEMA_VERSION = 1;
export const DIAGNOSTIC_PHASES = Object.freeze([
  'dns_address_policy',
  'broker_connect',
  'tls_handshake',
  'proxy_response',
  'chromium_navigation',
  'http_status',
  'unknown',
]);

const MAX_REQUESTS = 200;
const MAX_RESPONSE_BYTES = 32 * 1024 * 1024;
const MAX_METADATA_BYTES = 1024 * 1024;

const ERROR_CODES = Object.freeze([
  ['ERR_SSL_PROTOCOL_ERROR', 'tls_handshake', 'tls_protocol_error'],
  ['ERR_TLS_CERT_ALTNAME_INVALID', 'tls_handshake', 'tls_certificate_error'],
  ['CERT_HAS_EXPIRED', 'tls_handshake', 'tls_certificate_error'],
  ['UNABLE_TO_VERIFY_LEAF_SIGNATURE', 'tls_handshake', 'tls_certificate_error'],
  ['ERR_TLS_HANDSHAKE_TIMEOUT', 'tls_handshake', 'tls_timeout'],
  ['ERR_DNS_ADDRESS_POLICY', 'dns_address_policy', 'address_policy_denied'],
  ['ERR_DNS_PIN_CHANGED', 'dns_address_policy', 'address_pin_changed'],
  ['EAI_AGAIN', 'dns_address_policy', 'dns_lookup_failed'],
  ['EAI_NONAME', 'dns_address_policy', 'dns_lookup_failed'],
  ['ENOTFOUND', 'dns_address_policy', 'dns_lookup_failed'],
  ['BROKER_CONNECT', 'broker_connect', 'broker_connect_failed'],
  ['ECONNREFUSED', 'broker_connect', 'connection_refused'],
  ['ECONNRESET', 'broker_connect', 'connection_reset'],
  ['ETIMEDOUT', 'broker_connect', 'connection_timeout'],
  ['ERR_PROXY_CONNECTION_FAILED', 'proxy_response', 'proxy_connection_failed'],
  ['ERR_TUNNEL_CONNECTION_FAILED', 'proxy_response', 'proxy_tunnel_failed'],
  ['PROXY_RESPONSE', 'proxy_response', 'proxy_response_invalid'],
  ['ERR_HTTP_RESPONSE_CODE_FAILURE', 'http_status', 'http_status_error'],
  ['NAVIGATION_TIMEOUT', 'chromium_navigation', 'navigation_timeout'],
  ['ERR_ABORTED', 'chromium_navigation', 'navigation_aborted'],
]);

const PHASE_HINTS = new Set(DIAGNOSTIC_PHASES);

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
  return ERROR_CODES.find(([marker]) => normalized.includes(marker));
}

function hintMatch(value) {
  return typeof value === 'string' && PHASE_HINTS.has(value) ? value : undefined;
}

/**
 * Classify a coarse failure DTO.  Unknown fields are ignored and only
 * allowlisted codes become output, so passing an Error message cannot leak it.
 */
export function classifyFailure(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  const match = codeMatch(value.error_code) || codeMatch(value.code);
  const phase = match?.[1] || hintMatch(value.phase_hint) || (Number.isInteger(value.status) && value.status >= 300 ? 'http_status' : 'unknown');
  const reason = match?.[2] || (phase === 'http_status' ? `status_${statusClass(value.status)}` : phase === 'unknown' ? 'unclassified_failure' : `${phase}_failed`);
  return Object.freeze({
    schema_version: DIAGNOSTIC_SCHEMA_VERSION,
    phase,
    reason,
    status_class: statusClass(value.status),
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
  return classifyFailure({
    phase_hint: phaseHint,
    error_code: code || message,
    request_count: counters.request_count,
    response_bytes: counters.response_bytes,
    metadata_bytes: counters.metadata_bytes,
    status: counters.status,
  });
}

export function diagnosticCounters(state = {}) {
  return {
    request_count: boundedCounter(state.requests, MAX_REQUESTS),
    response_bytes: boundedCounter(state.responseBytes, MAX_RESPONSE_BYTES),
    metadata_bytes: boundedCounter(state.metadataBytes, MAX_METADATA_BYTES),
  };
}
