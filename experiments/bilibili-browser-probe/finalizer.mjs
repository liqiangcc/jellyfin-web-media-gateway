import { DIAGNOSTIC_PHASES, DIAGNOSTIC_REASONS, LIFECYCLE_OUTCOMES, TRANSPORT_OUTCOMES, TRANSPORT_STAGES, classifyFailure } from './diagnostic.mjs';
import { sanitizeStageMarkers } from './stage-markers.mjs';

export const TERMINATION_CLASSES = Object.freeze(['normal', 'error', 'timeout', 'abort', 'signal', 'unknown']);
export const RESULT_STATUSES = Object.freeze(['success', 'failure', 'unknown', 'blocked']);
const phaseSet = new Set(DIAGNOSTIC_PHASES);
const stageSet = new Set(TRANSPORT_STAGES);
const outcomeSet = new Set(TRANSPORT_OUTCOMES);
const lifecycleSet = new Set(LIFECYCLE_OUTCOMES);
const terminationSet = new Set(TERMINATION_CLASSES);
const resultSet = new Set(RESULT_STATUSES);
const reasonSet = new Set(DIAGNOSTIC_REASONS);

const bounded = (value, maximum) => Number.isSafeInteger(value) && value >= 0 && value <= maximum ? value : 0;
const bool = (value) => value === true;
const enumOr = (value, allowed, fallback) => typeof value === 'string' && allowed.has(value) ? value : fallback;

export function terminationClass(error, hint) {
  if (terminationSet.has(hint)) return hint;
  if (error?.name === 'TimeoutError' || error?.code === 'ETIMEDOUT') return 'timeout';
  if (error?.name === 'AbortError' || error?.code === 'ERR_ABORTED') return 'abort';
  return error ? 'error' : 'unknown';
}

function safeDiagnostic(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  const phase = enumOr(value.phase, phaseSet, 'unknown');
  const classified = classifyFailure({
    phase_hint: phase,
    status: typeof value.status_class === 'string' && /^[1-5]xx$/.test(value.status_class) ? Number(value.status_class[0]) * 100 : undefined,
    transport_stage: value.transport_stage,
    request_count: value.request_count,
    response_bytes: value.response_bytes,
    metadata_bytes: value.metadata_bytes,
    lifecycle_outcome: value.lifecycle_outcome,
  });
  return Object.freeze({
    schema_version: 2,
    phase,
    reason: typeof value.reason === 'string' && reasonSet.has(value.reason) ? value.reason : classified.reason,
    status_class: classified.status_class,
    transport_stage: enumOr(value.transport_stage, stageSet, classified.transport_stage),
    transport_outcome: enumOr(value.transport_outcome, outcomeSet, classified.transport_outcome),
    lifecycle_outcome: enumOr(value.lifecycle_outcome, lifecycleSet, 'unknown'),
    request_count: bounded(value.request_count, 200),
    response_bytes: bounded(value.response_bytes, 32 * 1024 * 1024),
    metadata_bytes: bounded(value.metadata_bytes, 1024 * 1024),
    stage_markers: Object.freeze(sanitizeStageMarkers(value.stage_markers)),
  });
}

function safeCleanup(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  const state = (name) => enumOr(value[name], new Set(['complete', 'pending', 'unknown']), 'unknown');
  return Object.freeze({
    browser_exit: state('browser_exit'),
    broker_close: state('broker_close'),
    temporary_profile: state('temporary_profile'),
    ephemeral_candidates: state('ephemeral_candidates'),
    dns_pins: state('dns_pins'),
    staging: state('staging'),
  });
}

export function finalizeResult(input = {}) {
  const value = input && typeof input === 'object' && !Array.isArray(input) ? input : {};
  return Object.freeze({
    schema_version: 2,
    result: enumOr(value.result, resultSet, 'unknown'),
    termination: enumOr(value.termination, terminationSet, 'unknown'),
    diagnostic: safeDiagnostic(value.diagnostic),
    activity: Object.freeze({
      page_navigation: bool(value.activity?.page_navigation), media_request: bool(value.activity?.media_request),
      click: bool(value.activity?.click), play: bool(value.activity?.play), full_preload: bool(value.activity?.full_preload),
      consumer: bool(value.activity?.consumer),
    }),
    cleanup: safeCleanup(value.cleanup),
  });
}

export function createFinalizer(stageTracker) {
  let finalized;
  return Object.freeze({
    finalize(input) {
      if (!finalized) {
        stageTracker?.record('finalizer_entry');
        const markers = stageTracker?.seal() || input?.diagnostic?.stage_markers;
        const diagnostic = { ...(input?.diagnostic || {}), stage_markers: markers };
        finalized = finalizeResult({ ...input, diagnostic });
      }
      return finalized;
    },
    get finalized() { return finalized; },
  });
}
