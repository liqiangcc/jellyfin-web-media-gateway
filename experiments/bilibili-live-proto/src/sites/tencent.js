// Tencent Video (v.qq.com) adapter — encrypted-response acquisition.
//
// The page POSTs vd6.l.qq.com/vinfo_proxy and receives an encrypted
// payload which the site JS decrypts into window.__VINFO_DATA__.
// __VINFO_DATA__.proxyhttp.vinfo is a JSON string with:
//   fl.fi[]          — available formats (id/name/cname: 480P..4K)
//   vl.vi[].ul.ui[]  — per-format CDN mirror URLs, each a complete
//                      m3u8 playlist URL (signed path, no vkey needed)
//
// Verified on ecs-node: apdcdn.tc.qq.com mirror responds 200 with real
// TS bytes; smtcdns.com mirrors reject TLS from datacenter IPs — the
// adapter probes each mirror and keeps the first reachable one.
//
// No rgv587-style page punishment observed on either VM or ECS.

import { evalInPage } from '../browser.js';

const UA =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0 Safari/537.36';

/** v.qq.com/x/cover/<cid>/<vid>.html | /x/page/<vid>.html | /x/cover/<cid>.html */
export function recognize(source) {
  const m =
    /v\.qq\.com\/x\/cover\/([a-z0-9]+)(?:\/([a-z0-9]+))?\.html/i.exec(source) ||
    /v\.qq\.com\/x\/page\/([a-z0-9]+)\.html/i.exec(source) ||
    /^tencent:(vid:[a-z0-9]+)$/i.exec(source);
  if (!m) return { matched: false };
  // /x/cover/<cid>.html has only one group; /x/page/<vid>.html puts vid in m[1]
  const cid = m.length > 2 ? m[1] : null;
  const vid = m.length > 2 ? m[2] || m[1] : m[1]?.replace(/^vid:/, '');
  if (!vid) return { matched: false };
  return {
    matched: true,
    site_id: 'tencent',
    locator: {
      site_id: 'tencent',
      locator_version: 1,
      opaque_payload: { vid, cid },
    },
  };
}

const PAGE = (vid, cid) =>
  cid
    ? `https://v.qq.com/x/cover/${cid}/${vid}.html`
    : `https://v.qq.com/x/page/${vid}.html`;

/** Probe a CDN mirror with HEAD; return first that answers. */
async function pickMirror(urls) {
  for (const u of urls) {
    try {
      const r = await fetch(u, {
        method: 'HEAD',
        headers: { 'User-Agent': UA, Referer: 'https://v.qq.com/' },
        signal: AbortSignal.timeout(8000),
      });
      if (r.status < 500) return u; // 200/403 both mean the host is reachable
    } catch {
      continue;
    }
  }
  return urls[urls.length - 1]; // fallback: last resort
}

export async function resolve(locator, { prefer = {} } = {}) {
  const { vid, cid } = locator.opaque_payload;
  const vinfo = await evalInPage(
    PAGE(vid, cid),
    'typeof window.__VINFO_DATA__==="object"&&!!window.__VINFO_DATA__.proxyhttp',
    `window.__VINFO_DATA__.proxyhttp.vinfo`,
    30000,
  );
  if (!vinfo || typeof vinfo !== 'string')
    throw Object.assign(new Error('BROWSER_BLOCKED'), { code: 'BROWSER_BLOCKED' });
  const d = JSON.parse(vinfo);
  const code = Number(d.code);
  if (Number.isFinite(code) && code !== 0)
    throw Object.assign(new Error(`UPSTREAM_ERROR: ${d.msg || d.code}`), {
      code: 'UPSTREAM_ERROR',
    });

  const vi = d.vl?.vi?.[0];
  if (!vi)
    throw Object.assign(new Error('SOURCE_UNSUPPORTED'), {
      code: 'SOURCE_UNSUPPORTED',
    });
  const title = vi.ti || vi.video_title || vid;
  const mirrors = (vi.ul?.ui || []).map((u) => u.url).filter(Boolean);
  if (!mirrors.length)
    throw Object.assign(new Error('SOURCE_UNSUPPORTED'), {
      code: 'SOURCE_UNSUPPORTED',
    });

  const base = await pickMirror(mirrors);
  // fl.fi[] gives format names; vl.vi[] entries here are the resolved
  // default-format ladder. Single stream for MVP — quality switching
  // needs re-requesting vinfo with a different defn param.
  const streams = [
    {
      id: 'default',
      kind: 'video',
      protocol: 'hls',
      url: base,
      width: vi.vw || 0,
      height: vi.vh || 0,
      upstream_headers: { 'User-Agent': UA, Referer: 'https://v.qq.com/' },
    },
  ];
  return {
    title,
    duration_ms: Math.round(Number(vi.td || 0) * 1000),
    is_preview: vi.isprevid === 1,
    proxied_hls: true,
    quality: vi.vh || 0,
    stream_idx: 0,
    streams,
    quality_options: streams.map((s) => ({ qn: s.height, label: `${s.height}p` })),
  };
}
