import test from 'node:test';
import assert from 'node:assert/strict';
import { classifyError, classifyFailure, diagnosticCounters, DIAGNOSTIC_PHASES } from './diagnostic.mjs';

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
