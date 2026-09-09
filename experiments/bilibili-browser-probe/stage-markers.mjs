/**
 * Closed, bounded lifecycle markers for the experimental browser probe.
 *
 * Marker values are deliberately coarse. This module is the only place that
 * constructs the durable marker DTO, so callback data cannot add free-form
 * errors, authorities, URLs, headers, or paths to diagnostic evidence.
 */

export const STAGE_MARKER_SCHEMA_VERSION = 1;
export const MAX_STAGE_MARKERS = 32;
export const STAGE_MARKER_EVENTS = Object.freeze([
  'browser_launch_start',
  'browser_launch_result',
  'navigation_start',
  'navigation_end',
  'navigation_status',
  'broker_request_start',
  'broker_request_result',
  'transport_outcome',
  'finalizer_entry',
]);
export const STAGE_MARKER_STATUS_CLASSES = Object.freeze(['1xx', '2xx', '3xx', '4xx', '5xx', 'unknown']);
export const STAGE_MARKER_TRANSPORT_STAGES = Object.freeze([
  'resolve_policy', 'tcp_connect', 'tls_handshake', 'proxy_response', 'downstream_close', 'unknown',
]);
export const STAGE_MARKER_TRANSPORT_OUTCOMES = Object.freeze(['success', 'failure', 'unknown']);

const eventSet = new Set(STAGE_MARKER_EVENTS);
const statusSet = new Set(STAGE_MARKER_STATUS_CLASSES);
const transportStageSet = new Set(STAGE_MARKER_TRANSPORT_STAGES);
const transportOutcomeSet = new Set(STAGE_MARKER_TRANSPORT_OUTCOMES);

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
    transport_stage: transportStage(input.transport_stage),
    transport_outcome: transportOutcome(input.transport_outcome),
    request_count: bounded(input.request_count, 200),
    response_bytes: bounded(input.response_bytes, 32 * 1024 * 1024),
    metadata_bytes: bounded(input.metadata_bytes, 1024 * 1024),
  });
}

/** Create an append-only marker stream that can be sealed by the finalizer. */
export function createStageTracker() {
  const markers = [];
  let sealed = false;
  return Object.freeze({
    record(event, fields = {}) {
      // Keep one slot available for the finalizer marker even if a noisy
      // broker emits the maximum number of request events.
      const reserve = event === 'finalizer_entry' ? 0 : 1;
      if (sealed || markers.length >= MAX_STAGE_MARKERS - reserve) return false;
      const marker = normalizeStageMarker({ ...fields, event }, markers.length + 1);
      if (!marker) return false;
      markers.push(marker);
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
