import test from 'node:test';
import assert from 'node:assert/strict';
import { summarizeObservation, rejectSensitiveInput } from '../../plugins/bilibili/experimental_probe.mjs';
import { parseSelector, navigationDescriptor, validateLiveInvocation } from '../../plugins/bilibili/live_selector.mjs';

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

test('plugin owns opaque live selector parsing and navigation', () => {
  const selector = parseSelector('bilibili:BV14V411W7r5:part-2');
  assert.deepEqual(selector, {
    site_id: 'bilibili', locator_version: 1, bvid: 'BV14V411W7r5', part: 2,
    opaque: 'bilibili:BV14V411W7r5:part-2',
  });
  assert.deepEqual(navigationDescriptor(selector), {
    site_id: 'bilibili', locator_version: 1, host: 'www.bilibili.com',
    url: 'https://www.bilibili.com/video/BV14V411W7r5?p=2', bvid: 'BV14V411W7r5', part: 2,
  });
});

test('live admission rejects caller-controlled authority and malformed selectors', () => {
  assert.equal(validateLiveInvocation({ mode: 'live', selector: 'bilibili:BV14V411W7r5:part-2' }).navigation.host, 'www.bilibili.com');
  for (const selector of [
    'https://www.bilibili.com/video/BV14V411W7r5',
    'bilibili:BV14V411W7r5:part-0',
    'bilibili:BV14V411W7r5:part-10000',
    'bilibili:BV14V411W7r5:part-2?cookie=x',
    'bilibili:BV14V411W7r5:part-2:profile',
  ]) assert.throws(() => parseSelector(selector));
  assert.throws(() => validateLiveInvocation({ mode: 'live', selector: 'bilibili:BV14V411W7r5:part-2', url: 'https://evil.invalid' }), /rejected/);
  assert.throws(() => validateLiveInvocation({ mode: 'live', selector: 'bilibili:BV14V411W7r5:part-2', headers: {} }), /rejected/);
});
