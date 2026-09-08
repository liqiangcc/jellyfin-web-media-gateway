import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { classifyError } from './diagnostic.mjs';
import { createFinalizer, finalizeResult, terminationClass, TERMINATION_CLASSES } from './finalizer.mjs';

test('maps Chromium HTTP2 and connection markers without copying messages', () => {
  const fixtures = [
    ['net::ERR_HTTP2_PROTOCOL_ERROR at https://secret.invalid/?token=x', 'http2_protocol_error'],
    ['net::ERR_HTTP2_STREAM_ERROR', 'http2_stream_error'],
    ['net::ERR_CONNECTION_CLOSED', 'connection_closed'],
    ['net::ERR_CONNECTION_RESET', 'connection_reset'],
    ['net::ERR_CONNECTION_REFUSED', 'connection_refused'],
    ['net::ERR_CONNECTION_TIMED_OUT', 'connection_timeout'],
  ];
  for (const [message, reason] of fixtures) {
    const result = classifyError({ message, code: '' }, 'chromium_navigation');
    assert.equal(result.reason, reason);
    assert.doesNotMatch(JSON.stringify(result), /https?:\/\/|secret|token|[?&=]/i);
  }
});

test('marker collisions and malformed termination inputs collapse safely', () => {
  const collision = classifyError({ code: 'NOT_ERR_HTTP2_PROTOCOL_ERROR' }, 'chromium_navigation');
  assert.equal(collision.phase, 'chromium_navigation');
  assert.equal(collision.reason, 'chromium_navigation_failed');
  assert.equal(collision.transport_stage, 'downstream_close');
  assert.equal(terminationClass({ name: 'UnexpectedError' }, 'not-allowed'), 'error');
  assert.deepEqual(TERMINATION_CLASSES, ['normal', 'error', 'timeout', 'abort', 'signal', 'unknown']);
  assert.equal(terminationClass({ name: 'TimeoutError' }), 'timeout');
  assert.equal(terminationClass({ name: 'AbortError' }), 'abort');
  assert.equal(terminationClass(undefined), 'unknown');
});

test('finalizer publishes one bounded result and ignores late callbacks', () => {
  const finalizer = createFinalizer();
  const first = finalizer.finalize({ result: 'failure', termination: 'signal', diagnostic: {
    phase: 'chromium_navigation', reason: 'connection_closed', transport_stage: 'downstream_close', transport_outcome: 'failure',
    request_count: 4, response_bytes: 20, metadata_bytes: 2,
  }, activity: { page_navigation: true, click: true }, cleanup: {
    browser_exit: 'complete', broker_close: 'complete', temporary_profile: 'complete', ephemeral_candidates: 'complete', dns_pins: 'complete', staging: 'complete',
  } });
  const late = finalizer.finalize({ result: 'success', termination: 'normal', diagnostic: { phase: 'unknown' } });
  assert.strictEqual(late, first);
  assert.equal(first.result, 'failure');
  assert.equal(first.termination, 'signal');
  assert.equal(first.activity.click, true);
  assert.equal(first.cleanup.staging, 'complete');
  assert.doesNotMatch(JSON.stringify(first), /https?:\/\/|cookie|authorization|secret|token/i);
});

test('finalizer gives explicit unknown cleanup for impossible paths', () => {
  const result = finalizeResult({ result: 'unknown', termination: 'unknown', diagnostic: { phase: 'unknown', transport_stage: 'unknown' } });
  assert.equal(result.result, 'unknown');
  assert.equal(result.termination, 'unknown');
  assert.equal(result.cleanup.browser_exit, 'unknown');
  assert.equal(result.activity.consumer, false);
});

test('finalizer rejects lexical but non-enumerated diagnostic reasons', () => {
  const result = finalizeResult({ result: 'failure', termination: 'error', diagnostic: {
    phase: 'chromium_navigation', reason: 'caller_invented_reason', transport_stage: 'downstream_close',
  } });
  assert.equal(result.diagnostic.reason, 'chromium_navigation_failed');
});

test('finalizer bounds timeout, abort, and process-error cleanup paths', () => {
  for (const termination of ['timeout', 'abort', 'error']) {
    const result = finalizeResult({ result: 'failure', termination, diagnostic: { phase: 'unknown', reason: 'untrusted https://secret.invalid/?token=x' }, cleanup: { browser_exit: 'unknown' } });
    assert.equal(result.termination, termination);
    assert.equal(result.cleanup.browser_exit, 'unknown');
    assert.doesNotMatch(JSON.stringify(result), /https?:\/\/|secret|token|[?&=]/i);
  }
});

test('probe has one output owner and bounded process-level termination', () => {
  const probe = fs.readFileSync(new URL('./probe.mjs', import.meta.url), 'utf8');
  const live = fs.readFileSync(new URL('./live.mjs', import.meta.url), 'utf8');
  assert.equal((probe.match(/process\.stdout\.write/g) || []).length, 1);
  assert.doesNotMatch(live, /process\.stdout\.write/);
  assert.match(probe, /\['SIGINT', 'SIGTERM'\]/);
  assert.match(probe, /setTimeout\(\(\) => \{[\s\S]*process\.exit\(exitCode\)/);
});
