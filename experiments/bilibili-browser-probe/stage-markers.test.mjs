import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createStageTracker,
  MAX_STAGE_MARKERS,
  normalizeStageMarker,
  sanitizeStageMarkers,
  STAGE_MARKER_EVENTS,
} from './stage-markers.mjs';

test('stage vocabulary and values are finite and redact unknown fields', () => {
  assert.deepEqual(STAGE_MARKER_EVENTS, [
    'browser_launch_start', 'browser_launch_result', 'navigation_start', 'navigation_end',
    'navigation_status', 'broker_request_start', 'broker_request_result', 'transport_outcome',
    'finalizer_entry',
  ]);
  const marker = normalizeStageMarker({
    event: 'navigation_status', status_class: '4xx', transport_stage: 'proxy_response', transport_outcome: 'failure',
    request_count: 4, response_bytes: 20, metadata_bytes: 2,
    error: 'https://secret.invalid/?token=sentinel', url: 'https://secret.invalid/', headers: { authorization: 'x' },
  }, 3);
  assert.deepEqual(marker, {
    schema_version: 1, sequence: 3, event: 'navigation_status', status_class: '4xx',
    transport_stage: 'proxy_response', transport_outcome: 'failure', request_count: 4,
    response_bytes: 20, metadata_bytes: 2,
  });
  assert.doesNotMatch(JSON.stringify(marker), /https?:\/\/|secret|token|authorization|sentinel/i);
  assert.equal(normalizeStageMarker({ event: 'caller_controlled_stage' }), undefined);
  assert.equal(normalizeStageMarker({ event: 'navigation_status', status_class: '999' }).status_class, 'unknown');
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
