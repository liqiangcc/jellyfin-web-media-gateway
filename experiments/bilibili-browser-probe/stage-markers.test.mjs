import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createStageTracker,
  MAX_STAGE_MARKERS,
  normalizeStageMarker,
  sanitizeStageMarkers,
  STAGE_MARKER_EVENTS,
  STAGE_MARKER_LIFECYCLE_OUTCOMES,
  STAGE_MARKER_UPSTREAM_ERROR_CLASSES,
  STAGE_MARKER_UPSTREAM_STATES,
  STAGE_MARKER_RESPONSE_ORIGINS,
  STAGE_MARKER_REDIRECT_CLASSES,
  STAGE_MARKER_RESPONSE_METADATA_CLASSES,
} from './stage-markers.mjs';

test('stage vocabulary and values are finite and redact unknown fields', () => {
  assert.deepEqual(STAGE_MARKER_EVENTS, [
    'browser_launch_start', 'browser_launch_result', 'navigation_start', 'navigation_end',
    'navigation_status', 'navigation_promise_result', 'page_lifecycle_result', 'browser_disconnect',
    'process_termination', 'broker_request_start', 'broker_request_result', 'upstream_response_start',
    'upstream_response_body_start', 'upstream_response_end', 'upstream_socket_close', 'upstream_timeout',
    'upstream_abort', 'upstream_error', 'transport_outcome', 'finalizer_entry',
  ]);
  assert.deepEqual(STAGE_MARKER_LIFECYCLE_OUTCOMES, [
    'fulfilled', 'rejected', 'timeout', 'aborted', 'page_closed', 'page_crashed',
    'browser_disconnected', 'process_error', 'process_signal', 'unknown',
  ]);
  assert.deepEqual(STAGE_MARKER_UPSTREAM_STATES, [
    'response_started', 'body_started', 'body_complete', 'closed_early',
    'closed_after_body', 'timeout', 'aborted', 'error', 'unknown',
  ]);
  assert.deepEqual(STAGE_MARKER_UPSTREAM_ERROR_CLASSES, [
    'timeout', 'aborted', 'connection_reset', 'connection_refused', 'tls_failure',
    'response_closed_early', 'unknown',
  ]);
  assert.deepEqual(STAGE_MARKER_RESPONSE_ORIGINS, ['broker_policy', 'upstream_http', 'navigation_status', 'unknown']);
  assert.deepEqual(STAGE_MARKER_REDIRECT_CLASSES, ['none', 'redirect', 'unknown']);
  assert.deepEqual(STAGE_MARKER_RESPONSE_METADATA_CLASSES, ['none', 'status_only', 'safe_headers', 'unknown']);
  const marker = normalizeStageMarker({
    event: 'navigation_status', status_class: '4xx', transport_stage: 'proxy_response', transport_outcome: 'failure',
    request_count: 4, response_bytes: 20, metadata_bytes: 2,
    error: 'https://secret.invalid/?token=sentinel', url: 'https://secret.invalid/', headers: { authorization: 'x' },
  }, 3);
  assert.deepEqual(marker, {
    schema_version: 2, sequence: 3, event: 'navigation_status', status_class: '4xx',
    response_origin: 'unknown', redirect_class: 'none', response_metadata_class: 'safe_headers',
    transport_stage: 'proxy_response', transport_outcome: 'failure', request_count: 4,
    lifecycle_outcome: 'unknown', upstream_state: 'unknown', upstream_error_class: 'unknown', response_bytes: 20, metadata_bytes: 2,
  });
  assert.doesNotMatch(JSON.stringify(marker), /https?:\/\/|secret|token|authorization|sentinel/i);
  assert.equal(normalizeStageMarker({ event: 'caller_controlled_stage' }), undefined);
  assert.equal(normalizeStageMarker({ event: 'navigation_status', status_class: '999' }).status_class, 'unknown');
  assert.equal(normalizeStageMarker({ event: 'navigation_status', status_class: '3xx' }).redirect_class, 'redirect');
  assert.equal(normalizeStageMarker({ event: 'navigation_status', response_origin: 'https://secret.invalid' }).response_origin, 'unknown');
  assert.equal(normalizeStageMarker({ event: 'navigation_status', status_class: '2xx', metadata_bytes: 2 }).response_metadata_class, 'safe_headers');
});

test('tracker emits monotonic bounded sequences and seals against late callbacks', () => {
  const tracker = createStageTracker();
  assert.equal(tracker.record('browser_launch_start'), true);
  assert.equal(tracker.record('browser_launch_result', { status_class: '2xx', transport_outcome: 'success' }), true);
  assert.deepEqual(tracker.snapshot().map(({ sequence, event }) => ({ sequence, event })), [
    { sequence: 1, event: 'browser_launch_start' }, { sequence: 2, event: 'browser_launch_result' },
  ]);
  for (let index = 2; index < MAX_STAGE_MARKERS - 1; index += 1) assert.equal(tracker.record('broker_request_start'), true);
  assert.equal(tracker.snapshot().length, MAX_STAGE_MARKERS - 1);
  assert.equal(tracker.record('finalizer_entry'), true);
  assert.equal(tracker.snapshot().length, MAX_STAGE_MARKERS);
  const sealed = tracker.seal();
  assert.equal(tracker.sealed, true);
  assert.equal(tracker.record('finalizer_entry'), false);
  assert.deepEqual(tracker.snapshot(), sealed);
});

test('tracker reserves post-navigation lifecycle markers before sealing', () => {
  const tracker = createStageTracker();
  for (let index = 0; index < 24; index += 1) tracker.record('broker_request_start');
  assert.equal(tracker.record('navigation_start'), true);
  assert.equal(tracker.record('navigation_promise_result', { lifecycle_outcome: 'rejected' }), true);
  assert.equal(tracker.record('page_lifecycle_result', { lifecycle_outcome: 'page_crashed' }), true);
  assert.equal(tracker.record('browser_disconnect', { lifecycle_outcome: 'browser_disconnected' }), true);
  assert.equal(tracker.record('navigation_status', { status_class: 'unknown' }), true);
  assert.equal(tracker.record('navigation_end', { transport_outcome: 'failure' }), true);
  assert.equal(tracker.record('finalizer_entry'), true);
  assert.deepEqual(tracker.snapshot().slice(-6).map(({ event, lifecycle_outcome }) => ({ event, lifecycle_outcome })), [
    { event: 'navigation_promise_result', lifecycle_outcome: 'rejected' },
    { event: 'page_lifecycle_result', lifecycle_outcome: 'page_crashed' },
    { event: 'browser_disconnect', lifecycle_outcome: 'browser_disconnected' },
    { event: 'navigation_status', lifecycle_outcome: 'unknown' },
    { event: 'navigation_end', lifecycle_outcome: 'unknown' },
    { event: 'finalizer_entry', lifecycle_outcome: 'unknown' },
  ]);
});

test('tracker retains upstream boundaries through noisy broker activity', () => {
  const tracker = createStageTracker();
  for (let index = 0; index < 24; index += 1) assert.equal(tracker.record('broker_request_start'), true);
  assert.equal(tracker.record('navigation_start'), true);
  const upstreamEvents = [
    'upstream_response_start', 'upstream_response_body_start', 'upstream_response_end',
    'upstream_socket_close', 'upstream_timeout', 'upstream_abort', 'upstream_error',
  ];
  for (const event of upstreamEvents) assert.equal(tracker.record(event), true);
  for (const event of [
    'navigation_promise_result', 'page_lifecycle_result', 'browser_disconnect',
    'navigation_status', 'navigation_end', 'process_termination',
  ]) assert.equal(tracker.record(event), true);
  assert.equal(tracker.record('finalizer_entry'), true);

  const markers = tracker.snapshot();
  assert.equal(markers.length, MAX_STAGE_MARKERS);
  assert.deepEqual(markers.filter(({ event }) => event.startsWith('upstream_')).map(({ event }) => event), upstreamEvents);
  assert.deepEqual(markers.filter(({ event }) => event === 'navigation_start' || event === 'finalizer_entry').map(({ event }) => event), [
    'navigation_start', 'finalizer_entry',
  ]);
  assert.deepEqual(markers.map(({ sequence }) => sequence), Array.from({ length: MAX_STAGE_MARKERS }, (_, index) => index + 1));
  assert.equal(tracker.record('upstream_error'), false);
  const sealed = tracker.seal();
  assert.deepEqual(tracker.snapshot(), sealed);
  assert.equal(tracker.record('upstream_socket_close'), false);
});

test('sanitizer drops malformed markers and reindexes bounded output', () => {
  const markers = sanitizeStageMarkers([
    { event: 'navigation_start', sequence: 99 },
    { event: 'attacker_event', status_class: '2xx' },
    { event: 'finalizer_entry', sequence: 0, request_count: 201, response_bytes: -1 },
  ]);
  assert.deepEqual(markers.map(({ sequence, event, request_count, response_bytes }) => ({ sequence, event, request_count, response_bytes })), [
    { sequence: 1, event: 'navigation_start', request_count: 0, response_bytes: 0 },
    { sequence: 2, event: 'finalizer_entry', request_count: 0, response_bytes: 0 },
  ]);
});
