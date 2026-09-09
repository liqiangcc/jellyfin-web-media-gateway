import test from 'node:test';
import assert from 'node:assert/strict';
import { classifyError, classifyFailure, classifyNavigationLifecycle, classifyProcessTermination, classifyResponseMetadata, classifyUpstreamError, diagnosticCounters, DIAGNOSTIC_PHASES, LIFECYCLE_OUTCOMES, RESPONSE_METADATA_CLASSES, RESPONSE_ORIGINS, RESPONSE_REDIRECT_CLASSES, UPSTREAM_ERROR_CLASSES } from './diagnostic.mjs';

test('classifies every bounded phase with schema-safe output', () => {
  const inputs = [
    { phase_hint: 'dns_address_policy' },
    { code: 'ECONNREFUSED' },
    { code: 'ERR_SSL_PROTOCOL_ERROR' },
    { phase_hint: 'proxy_response' },
    { phase_hint: 'chromium_navigation' },
    { status: 503 },
    { error_code: 'not-an-allowlisted-code' },
  ];
  assert.deepEqual(inputs.map((input) => classifyFailure(input).phase), DIAGNOSTIC_PHASES);
  assert.equal(classifyFailure({ status: 404 }).status_class, '4xx');
  assert.equal(classifyFailure({ status: 404 }).reason, 'status_4xx');
});

test('maps browser TLS errors without retaining sensitive diagnostics', () => {
  const secretMessage = 'net::ERR_SSL_PROTOCOL_ERROR at https://example.invalid/?token=sentinel';
  const result = classifyError({ code: '', message: secretMessage }, undefined, { requests: 2 });
  assert.equal(result.phase, 'tls_handshake');
  assert.equal(result.reason, 'tls_protocol_error');
  assert.equal(JSON.stringify(result).includes(secretMessage), false);
  assert.equal(JSON.stringify(result).includes('https://'), false);
  assert.equal(JSON.stringify(result).includes('sentinel'), false);
});

test('counters are bounded and never preserve arbitrary values', () => {
  assert.deepEqual(diagnosticCounters({ requests: 201, responseBytes: -1, metadataBytes: 1048577 }), {
    request_count: 0, response_bytes: 0, metadata_bytes: 0,
  });
  assert.deepEqual(diagnosticCounters({ requests: 3, responseBytes: 128, metadataBytes: 64 }), {
    request_count: 3, response_bytes: 128, metadata_bytes: 64,
  });
});

test('success metadata is unaffected by diagnostic helpers', () => {
  const metadata = { part_match: true, source_kind: 'muxed', candidate_counts: { total: 1 } };
  const before = JSON.stringify(metadata);
  classifyFailure({ phase_hint: 'unknown', error_code: 'https://secret.invalid/body' });
  assert.equal(JSON.stringify(metadata), before);
});

test('response provenance and redirect metadata are finite and never inferred from status', () => {
  assert.deepEqual(RESPONSE_ORIGINS, ['broker_policy', 'upstream_http', 'navigation_status', 'unknown']);
  assert.deepEqual(RESPONSE_REDIRECT_CLASSES, ['none', 'redirect', 'unknown']);
  assert.deepEqual(RESPONSE_METADATA_CLASSES, ['none', 'status_only', 'safe_headers', 'unknown']);
  assert.deepEqual(classifyResponseMetadata({ response_origin: 'broker_policy', status: 403, request_count: 4, response_bytes: 20, metadata_bytes: 2 }), {
    schema_version: 2, response_origin: 'broker_policy', status_class: '4xx', redirect_class: 'none', response_metadata_class: 'status_only',
    request_count: 4, response_bytes: 20, metadata_bytes: 2,
  });
  assert.deepEqual(classifyResponseMetadata({ response_origin: 'upstream_http', status: 302 }), {
    schema_version: 2, response_origin: 'upstream_http', status_class: '3xx', redirect_class: 'redirect', response_metadata_class: 'status_only',
    request_count: 0, response_bytes: 0, metadata_bytes: 0,
  });
  const unknown = classifyResponseMetadata({ response_origin: 'https://secret.invalid', status: 499, redirect_class: 'attacker', body: 'token=sentinel' });
  assert.equal(unknown.response_origin, 'unknown');
  assert.equal(unknown.redirect_class, 'none');
  assert.equal(unknown.response_metadata_class, 'status_only');
  assert.doesNotMatch(JSON.stringify(unknown), /https?:\/\/|secret|token|sentinel/i);
  assert.equal(classifyFailure({ status: 404, response_origin: 'upstream_http' }).response_origin, 'upstream_http');
  assert.equal(classifyFailure({ status: 404 }).response_origin, 'unknown');
});

test('post-navigation lifecycle markers reduce to finite classes without raw error data', () => {
  const cases = [
    [{ name: 'TimeoutError', message: 'https://secret.invalid/?token=timeout' }, 'timeout'],
    [{ name: 'AbortError', message: 'secret abort' }, 'aborted'],
    [{ name: 'BrowserDisconnectedError', message: 'secret disconnect' }, 'browser_disconnected'],
    [{ name: 'PageClosedError', message: 'secret close' }, 'page_closed'],
    [{ name: 'PageCrashedError', message: 'secret crash' }, 'page_crashed'],
    [{ name: 'Error', message: 'secret rejection' }, 'rejected'],
  ];
  const outputs = cases.map(([error, expected]) => {
    const actual = classifyNavigationLifecycle(error);
    assert.equal(actual, expected);
    return actual;
  });
  assert.equal(classifyNavigationLifecycle(undefined), 'unknown');
  assert.equal(classifyNavigationLifecycle(undefined, 'browser_disconnected'), 'browser_disconnected');
  assert.equal(classifyProcessTermination('error'), 'process_error');
  assert.equal(classifyProcessTermination('signal'), 'process_signal');
  assert.deepEqual(LIFECYCLE_OUTCOMES, [
    'fulfilled', 'rejected', 'timeout', 'aborted', 'page_closed', 'page_crashed',
    'browser_disconnected', 'process_error', 'process_signal', 'unknown',
  ]);
  assert.doesNotMatch(JSON.stringify(outputs), /https?:\/\/|secret|token/i);
});

test('classifies an observed generic navigation rejection without inventing a cause', () => {
  const result = classifyError(
    { name: 'Error', message: 'opaque rejection at https://secret.invalid/?token=sentinel' },
    'chromium_navigation',
    { lifecycle_outcome: 'rejected', request_count: 13, response_bytes: 15224 },
  );
  assert.equal(result.phase, 'chromium_navigation');
  assert.equal(result.reason, 'navigation_promise_rejected');
  assert.equal(result.transport_stage, 'downstream_close');
  assert.equal(result.transport_outcome, 'failure');
  assert.equal(result.request_count, 13);
  assert.equal(result.response_bytes, 15224);
  assert.doesNotMatch(JSON.stringify(result), /https?:\/\/|secret|token|sentinel/i);
  assert.equal(classifyError({ name: 'TimeoutError' }, 'chromium_navigation').reason, 'navigation_timeout');
  assert.equal(classifyError({ name: 'AbortError' }, 'chromium_navigation').reason, 'navigation_aborted');
  assert.equal(classifyError({ name: 'TargetClosedError' }, 'chromium_navigation').reason, 'navigation_browser_disconnected');
  assert.equal(classifyFailure({ phase_hint: 'chromium_navigation', lifecycle_outcome: 'unknown' }).reason, 'chromium_navigation_failed');
});

test('classifies upstream errors into finite redacted classes', () => {
  const cases = [
    [{ code: 'ETIMEDOUT', message: 'https://secret.invalid/?token=timeout' }, 'timeout'],
    [{ name: 'AbortError', message: 'secret abort' }, 'aborted'],
    [{ code: 'ECONNRESET', message: 'secret reset' }, 'connection_reset'],
    [{ code: 'ECONNREFUSED', message: 'secret refused' }, 'connection_refused'],
    [{ code: 'ERR_TLS_CERT_ALTNAME_INVALID', message: 'secret cert' }, 'tls_failure'],
    [{ code: 'UNKNOWN_UPSTREAM', message: 'https://secret.invalid/?token=unknown' }, 'unknown'],
  ];
  for (const [error, expected] of cases) assert.equal(classifyUpstreamError(error), expected);
  assert.equal(classifyUpstreamError({}, 'response_closed_early'), 'response_closed_early');
  assert.deepEqual(UPSTREAM_ERROR_CLASSES, [
    'timeout', 'aborted', 'connection_reset', 'connection_refused', 'tls_failure',
    'response_closed_early', 'unknown',
  ]);
  const output = cases.map(([error]) => classifyUpstreamError(error));
  assert.doesNotMatch(JSON.stringify(output), /https?:\/\/|secret|token/i);
});
