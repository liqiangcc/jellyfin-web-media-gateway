/**
 * Bilibili's opaque selector and public navigation descriptor for the
 * experimental browser probe. This is plugin knowledge: the generic browser
 * runtime receives only the resulting descriptor and never parses it.
 */

export const SELECTOR_VERSION = 1;
const BVID = /^BV[0-9A-Za-z]{10}$/;
const SELECTOR = /^bilibili:(BV[0-9A-Za-z]{10}):part-([1-9][0-9]{0,3})$/;

function reject(text, label = 'selector') {
  if (typeof text !== 'string' || text.length === 0 || text.length > 128) {
    throw new Error(`${label} must be a bounded string`);
  }
  if (/https?:\/\/|blob:|file:|cookie|authorization|token|signed|sessdata|bearer|[?&#=]/i.test(text)) {
    throw new Error(`${label} must be an opaque selector`);
  }
}

export function parseSelector(value) {
  reject(value);
  const match = SELECTOR.exec(value);
  if (!match || !BVID.test(match[1])) throw new Error('invalid Bilibili selector');
  const part = Number(match[2]);
  if (!Number.isSafeInteger(part) || part < 1 || part > 9999) throw new Error('invalid Bilibili part');
  return Object.freeze({ site_id: 'bilibili', locator_version: SELECTOR_VERSION, bvid: match[1], part, opaque: value });
}

export function navigationDescriptor(selector) {
  const parsed = typeof selector === 'string' ? parseSelector(selector) : selector;
  if (!parsed || parsed.site_id !== 'bilibili' || parsed.locator_version !== SELECTOR_VERSION || !BVID.test(parsed.bvid)) {
    throw new Error('selector is not owned by the Bilibili plugin');
  }
  const part = Number(parsed.part);
  if (!Number.isSafeInteger(part) || part < 1 || part > 9999) throw new Error('invalid Bilibili part');
  return Object.freeze({
    site_id: 'bilibili',
    locator_version: SELECTOR_VERSION,
    host: 'www.bilibili.com',
    // This URL is server-only. It must never be included in sanitized output.
    url: `https://www.bilibili.com/video/${parsed.bvid}?p=${part}`,
    bvid: parsed.bvid,
    part,
  });
}

export function validateLiveInvocation(input) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) throw new Error('live invocation must be an object');
  const keys = Object.keys(input);
  if (keys.some((key) => !['mode', 'selector', 'timeout_ms'].includes(key))) {
    throw new Error('caller-controlled URL, profile, headers, proxy, or CDP input rejected');
  }
  if (input.mode !== 'live') throw new Error('live invocation mode required');
  const selector = parseSelector(input.selector);
  const timeout = input.timeout_ms === undefined ? 120000 : Number(input.timeout_ms);
  if (!Number.isSafeInteger(timeout) || timeout < 1000 || timeout > 120000) throw new Error('timeout outside live budget');
  return Object.freeze({ selector, navigation: navigationDescriptor(selector), timeout_ms: timeout });
}
