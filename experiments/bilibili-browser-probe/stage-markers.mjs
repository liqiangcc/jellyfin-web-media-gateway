/**
 * Closed, bounded lifecycle markers for the experimental browser probe.
 *
 * Marker values are deliberately coarse. This module is the only place that
 * constructs the durable marker DTO, so callback data cannot add free-form
 * errors, authorities, URLs, headers, or paths to diagnostic evidence.
 */

export const STAGE_MARKER_SCHEMA_VERSION = 2;
export const MAX_STAGE_MARKERS = 32;
export const STAGE_MARKER_EVENTS = Object.freeze([
  'browser_launch_start',
  'browser_launch_result',
  'navigation_start',
  'navigation_end',
  'navigation_status',
  'navigation_promise_result',
  'page_lifecycle_result',
  'browser_disconnect',
  'process_termination',
  'broker_request_start',
  'broker_request_result',
  'upstream_response_start',
  'upstream_response_body_start',
  'upstream_response_end',
  'upstream_socket_close',
  'upstream_timeout',
  'upstream_abort',
  'upstream_error',
  'transport_outcome',
  'finalizer_entry',
]);
export const STAGE_MARKER_STATUS_CLASSES = Object.freeze(['1xx', '2xx', '3xx', '4xx', '5xx', 'unknown']);
export const STAGE_MARKER_TRANSPORT_STAGES = Object.freeze([
  'resolve_policy', 'tcp_connect', 'tls_handshake', 'proxy_response', 'downstream_close', 'unknown',
]);
export const STAGE_MARKER_TRANSPORT_OUTCOMES = Object.freeze(['success', 'failure', 'unknown']);
export const STAGE_MARKER_LIFECYCLE_OUTCOMES = Object.freeze([
  'fulfilled', 'rejected', 'timeout', 'aborted', 'page_closed', 'page_crashed',
  'browser_disconnected', 'process_error', 'process_signal', 'unknown',
]);
export const STAGE_MARKER_UPSTREAM_STATES = Object.freeze([
  'response_started', 'body_started', 'body_complete', 'closed_early',
  'closed_after_body', 'timeout', 'aborted', 'error', 'unknown',
]);
export const STAGE_MARKER_UPSTREAM_ERROR_CLASSES = Object.freeze([
  'timeout', 'aborted', 'connection_reset', 'connection_refused', 'tls_failure',
  'response_closed_early', 'unknown',
]);
export const STAGE_MARKER_RESPONSE_ORIGINS = Object.freeze([
  'broker_policy', 'upstream_http', 'navigation_status', 'unknown',
]);
export const STAGE_MARKER_REDIRECT_CLASSES = Object.freeze(['none', 'redirect', 'unknown']);
export const STAGE_MARKER_RESPONSE_METADATA_CLASSES = Object.freeze(['none', 'status_only', 'safe_headers', 'unknown']);

const eventSet = new Set(STAGE_MARKER_EVENTS);
const statusSet = new Set(STAGE_MARKER_STATUS_CLASSES);
const transportStageSet = new Set(STAGE_MARKER_TRANSPORT_STAGES);
const transportOutcomeSet = new Set(STAGE_MARKER_TRANSPORT_OUTCOMES);
const lifecycleOutcomeSet = new Set(STAGE_MARKER_LIFECYCLE_OUTCOMES);
const upstreamStateSet = new Set(STAGE_MARKER_UPSTREAM_STATES);
const upstreamErrorClassSet = new Set(STAGE_MARKER_UPSTREAM_ERROR_CLASSES);
const responseOriginSet = new Set(STAGE_MARKER_RESPONSE_ORIGINS);
const redirectClassSet = new Set(STAGE_MARKER_REDIRECT_CLASSES);
const responseMetadataClassSet = new Set(STAGE_MARKER_RESPONSE_METADATA_CLASSES);

function bounded(value, maximum) {
  return Number.isSafeInteger(value) && value >= 0 && value <= maximum ? value : 0;
}

function statusClass(value) {
  return typeof value === 'string' && statusSet.has(value) ? value : 'unknown';
}

function transportStage(value) {
  return typeof value === 'string' && transportStageSet.has(value) ? value : 'unknown';
}

function transportOutcome(value) {
  return typeof value === 'string' && transportOutcomeSet.has(value) ? value : 'unknown';
}

function lifecycleOutcome(value) {
  return typeof value === 'string' && lifecycleOutcomeSet.has(value) ? value : 'unknown';
}

function upstreamState(value) {
  return typeof value === 'string' && upstreamStateSet.has(value) ? value : 'unknown';
}

function upstreamErrorClass(value) {
  return typeof value === 'string' && upstreamErrorClassSet.has(value) ? value : 'unknown';
}

function responseOrigin(value) {
  return typeof value === 'string' && responseOriginSet.has(value) ? value : 'unknown';
}

function redirectClass(value, status) {
  if (typeof value === 'string' && redirectClassSet.has(value)) return value;
  return status === '3xx' ? 'redirect' : status === 'unknown' ? 'unknown' : 'none';
}

function responseMetadataClass(value, status, metadataBytes) {
  if (typeof value === 'string' && responseMetadataClassSet.has(value)) return value;
  if (status === 'unknown') return 'unknown';
  if (Number.isSafeInteger(metadataBytes) && metadataBytes > 0) return 'safe_headers';
  return 'status_only';
}

/**
 * Normalize a marker into the closed durable DTO. Invalid event names are
 * rejected. Unknown fields are ignored and all bounded values have a safe
 * fallback, including when the input contains sensitive strings.
 */
export function normalizeStageMarker(input = {}, sequence = 1) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return undefined;
  if (!eventSet.has(input.event)) return undefined;
  return Object.freeze({
    schema_version: STAGE_MARKER_SCHEMA_VERSION,
    sequence: bounded(sequence, MAX_STAGE_MARKERS),
    event: input.event,
    status_class: statusClass(input.status_class),
    response_origin: responseOrigin(input.response_origin),
    redirect_class: redirectClass(input.redirect_class, input.status_class),
    response_metadata_class: responseMetadataClass(input.response_metadata_class, input.status_class, input.metadata_bytes),
    transport_stage: transportStage(input.transport_stage),
    transport_outcome: transportOutcome(input.transport_outcome),
    lifecycle_outcome: lifecycleOutcome(input.lifecycle_outcome),
    upstream_state: upstreamState(input.upstream_state),
    upstream_error_class: upstreamErrorClass(input.upstream_error_class),
    request_count: bounded(input.request_count, 200),
    response_bytes: bounded(input.response_bytes, 32 * 1024 * 1024),
    metadata_bytes: bounded(input.metadata_bytes, 1024 * 1024),
  });
}

const POST_NAVIGATION_PRIORITY_EVENTS = Object.freeze([
  'navigation_status', 'navigation_end', 'navigation_promise_result',
  'page_lifecycle_result', 'browser_disconnect', 'process_termination',
  'upstream_response_start', 'upstream_response_body_start', 'upstream_response_end',
  'upstream_socket_close', 'upstream_timeout', 'upstream_abort', 'upstream_error',
]);
const evictableEvents = new Set(['broker_request_start', 'broker_request_result']);

function reindexMarkers(markers) {
  for (let index = 0; index < markers.length; index += 1) {
    markers[index] = normalizeStageMarker(markers[index], index + 1);
  }
}

function evictBrokerNoise(markers) {
  const index = markers.findIndex(({ event }) => evictableEvents.has(event));
  if (index < 0) return false;
  markers.splice(index, 1);
  reindexMarkers(markers);
  return true;
}

/** Create a finite retained marker stream that can be sealed by the finalizer. */
export function createStageTracker() {
  const markers = [];
  let sealed = false;
  let postNavigationReserve = false;
  let pendingPriorityEvents = new Set();
  return Object.freeze({
    record(event, fields = {}) {
      if (sealed) return false;
      if (event === 'navigation_start' && !postNavigationReserve) {
        postNavigationReserve = true;
        pendingPriorityEvents = new Set(POST_NAVIGATION_PRIORITY_EVENTS);
      }
      // Keep navigation, upstream response/socket and finalizer boundaries
      // observable even if a noisy broker fills the stream first. Once
      // navigation starts, only broker noise is evictable; the finite budget
      // still rejects all other unreserved events.
      const priority = event === 'navigation_start'
        || (postNavigationReserve && pendingPriorityEvents.has(event));
      const remaining = pendingPriorityEvents.size - (pendingPriorityEvents.has(event) ? 1 : 0);
      const reserve = event === 'finalizer_entry' ? 0 : 1 + (postNavigationReserve ? Math.max(0, remaining) : 0);
      if (priority || event === 'finalizer_entry') {
        if (markers.length >= MAX_STAGE_MARKERS && !evictBrokerNoise(markers)) return false;
      } else if (markers.length >= MAX_STAGE_MARKERS - reserve) {
        return false;
      }
      const marker = normalizeStageMarker({ ...fields, event }, markers.length + 1);
      if (!marker) return false;
      markers.push(marker);
      if (postNavigationReserve && pendingPriorityEvents.has(event)) pendingPriorityEvents.delete(event);
      return true;
    },
    snapshot() {
      return markers.slice();
    },
    seal() {
      sealed = true;
      return markers.slice();
    },
    get sealed() {
      return sealed;
    },
  });
}

export function sanitizeStageMarkers(value) {
  if (!Array.isArray(value)) return [];
  const output = [];
  for (const item of value.slice(0, MAX_STAGE_MARKERS)) {
    const marker = normalizeStageMarker(item, output.length + 1);
    if (marker) output.push(marker);
  }
  return output;
}
