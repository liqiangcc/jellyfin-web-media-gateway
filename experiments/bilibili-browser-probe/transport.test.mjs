import test from 'node:test';
import assert from 'node:assert/strict';
import {
  classifyFailure, classifyTransport, DIAGNOSTIC_SCHEMA_VERSION,
  TRANSPORT_OUTCOMES, TRANSPORT_STAGES,
} from './diagnostic.mjs';

test('resolver, socket, TLS, proxy and downstream fixtures retain earliest bounded stage', () => {
  const fixtures = [
    ['resolver policy', { code: 'ERR_DNS_ADDRESS_POLICY' }, 'resolve_policy'],
    ['TCP socket', { code: 'ECONNREFUSED' }, 'tcp_connect'],
    ['TLS socket', { code: 'ERR_SSL_PROTOCOL_ERROR' }, 'tls_handshake'],
    ['proxy response', { code: 'ERR_TUNNEL_CONNECTION_FAILED' }, 'proxy_response'],
    ['downstream close', { phase_hint: 'chromium_navigation', transport_stage: 'downstream_close' }, 'downstream_close'],
  ];
  for (const [name, input, stage] of fixtures) {
    const result = classifyFailure(input);
    assert.equal(result.transport_stage, stage, name);
    assert.equal(result.transport_outcome, 'failure', name);
    assert.equal(result.schema_version, DIAGNOSTIC_SCHEMA_VERSION, name);
  }
});

test('generic broker failure is explicitly unknown rather than guessed as TCP or TLS', () => {
  const result = classifyFailure({ code: 'BROKER_CONNECT', error_code: 'https://private.invalid/?token=sentinel' });
  assert.equal(result.phase, 'broker_connect');
  assert.equal(result.transport_stage, 'unknown');
  assert.equal(result.transport_outcome, 'failure');
  assert.doesNotMatch(JSON.stringify(result), /https?:\/\/|private|sentinel|token/i);
});

test('success and unknown transport observations are finite and bounded', () => {
  const success = classifyTransport({ stage: 'downstream_close', outcome: 'success', status: 200, request_count: 200, response_bytes: 33554432, metadata_bytes: 1048576 });
  assert.deepEqual(success, {
    schema_version: DIAGNOSTIC_SCHEMA_VERSION, transport_stage: 'downstream_close', transport_outcome: 'success',
    status_class: '2xx', request_count: 200, response_bytes: 33554432, metadata_bytes: 1048576,
  });
  const unknown = classifyTransport({ stage: 'attacker-stage', outcome: 'attacker-outcome', request_count: 201, response_bytes: -1, metadata_bytes: 1048577 });
  assert.equal(unknown.transport_stage, 'unknown');
  assert.equal(unknown.transport_outcome, 'unknown');
  assert.deepEqual(TRANSPORT_STAGES, ['resolve_policy', 'tcp_connect', 'tls_handshake', 'proxy_response', 'downstream_close', 'unknown']);
  assert.deepEqual(TRANSPORT_OUTCOMES, ['success', 'failure', 'unknown']);
});
