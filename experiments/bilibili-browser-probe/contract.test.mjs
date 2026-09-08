import test from 'node:test';
import assert from 'node:assert/strict';
import { summarizeObservation, rejectSensitiveInput } from '../../plugins/bilibili/experimental_probe.mjs';

const base = {
  schema_version: 1,
  selector: { site_id: 'bilibili', content_id: 'BV-synthetic-165', part: 2 },
  part_match: true, source_kind: 'muxed', events: 3, resources: 1, metadata_bytes: 128,
  independent_after_browser_exit: true,
  candidates: [{ role: 'muxed', codec: 'avc1', container: 'mp4', http_status_class: '2xx', range_supported: true, header_names: ['content-type', 'cookie', 'accept-ranges'], egress_allowed: true, expiry_hint: 'none-observed' }],
};

test('normalizes muxed observation and strips secret header names', () => {
  const result = summarizeObservation(base);
  assert.equal(result.schema_version, 1);
  assert.deepEqual(result.candidate_counts, { total: 1, muxed: 1, video: 0, audio: 0 });
  assert.deepEqual(result.candidates[0].header_names, ['content-type', 'accept-ranges']);
});

test('preserves separate AV roles and opaque part identity', () => {
  const result = summarizeObservation({ ...base, source_kind: 'av_separated', candidates: [
    { ...base.candidates[0], role: 'video' }, { ...base.candidates[0], role: 'audio', container: 'm4a', codec: 'mp4a' },
  ] });
  assert.equal(result.selector.part, 2);
  assert.equal(result.candidate_counts.video, 1);
  assert.equal(result.candidate_counts.audio, 1);
});

test('rejects oversized, malformed, or secret-bearing observations', () => {
  assert.throws(() => summarizeObservation({ ...base, events: 201 }), /budget/);
  assert.throws(() => summarizeObservation({ ...base, selector: { ...base.selector, part: 0 } }), /part/);
  assert.throws(() => summarizeObservation({ ...base, candidates: [{ ...base.candidates[0], role: 'subtitle' }] }), /role/);
  assert.throws(() => rejectSensitiveInput({ upstream_url: 'https://cdn.invalid/a?token=secret' }), /sensitive/);
});

test('unknown optional fields remain explicit and bounded', () => {
  const result = summarizeObservation({ ...base, source_kind: 'unknown', independent_after_browser_exit: false, candidates: [
    { role: 'muxed', egress_allowed: false, range_supported: 'unknown' },
  ] });
  assert.equal(result.source_kind, 'unknown');
  assert.equal(result.candidates[0].codec, 'unknown');
  assert.equal(result.candidates[0].expiry_hint, 'unknown');
});
