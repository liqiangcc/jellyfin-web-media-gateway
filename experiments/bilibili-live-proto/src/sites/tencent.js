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

/**
 * Danmaku — dm.video.qq.com/barrage/segment/<vid>/t/v1/<start>/<end>
 * is a plain GET (no auth/signing), paginated in 30s windows.
 * Lazy: mats are per-minute buckets → two 30s segment fetches each.
 * Field map: time_offset=ms, content=text, content_style JSON has
 * color/position when non-empty.
 */
export const lazy_danmaku = true;

/**
 * Search — trpc.videosearch.mobile_search.MultiTerminalSearch/MbSearch
 * is a plain POST (no signing). Results live in
 * data.normalList.itemList[].videoInfo.videoDoc {id,title,imgUrl,timeLong}.
 */
export async function search(keyword, { limit = 20 } = {}) {
  const r = await fetch(
    'https://pbaccess.video.qq.com/trpc.videosearch.mobile_search.MultiTerminalSearch/MbSearch?vplatform=2',
    {
      method: 'POST',
      headers: {
        'User-Agent': UA,
        'content-type': 'application/json',
        origin: 'https://v.qq.com',
        referer: 'https://v.qq.com/x/search/',
      },
      body: JSON.stringify({
        version: '26022601',
        clientType: 1,
        query: keyword,
        pagenum: 0,
        pagesize: Math.max(limit, 10),
        queryFrom: 0,
        isPrefetch: true,
        isneedQc: true,
      }),
    },
  );
  const d = await r.json();
  const items = d?.data?.normalList?.itemList || [];
  const hits = [];
  for (const it of items) {
    const vid = it?.doc?.id;
    const vi = it?.videoInfo;
    if (!vid || !vi) continue;
    hits.push({
      vid,
      title: String(vi.title || '').replace(/<[^>]+>/g, ''),
      img: vi.imgUrl || '',
      duration: vi.videoDoc?.timeLong || '',
      mark: vi.views || vi.typeName || '',
    });
  }
  return hits.slice(0, limit);
}

export async function danmaku(locator, { mats = [0, 1, 2] } = {}) {
  const vid = locator.opaque_payload.vid;
  const out = [];
  for (const mat of mats) {
    for (const half of [0, 1]) {
      const start = mat * 60000 + half * 30000;
      const end = start + 30000;
      try {
        const r = await fetch(
          `https://dm.video.qq.com/barrage/segment/${vid}/t/v1/${start}/${end}`,
          { headers: { 'User-Agent': UA, Referer: 'https://v.qq.com/' } },
        );
        const d = await r.json();
        for (const it of d.barrage_list || []) {
          let color = 16777215;
          let mode = 1;
          if (it.content_style) {
            try {
              const st = JSON.parse(it.content_style);
              if (st.color) color = parseInt(st.color, 16);
              if (st.position === 2) mode = 5; // top
              else if (st.position === 3) mode = 4; // bottom
            } catch {}
          }
          const t = Number(it.time_offset) / 1000;
          const text = String(it.content || '').trim();
          if (Number.isFinite(t) && text) out.push({ t, mode, color, text });
        }
        await new Promise((r2) => setTimeout(r2, 100 + Math.random() * 150));
      } catch {
        /* skip window */
      }
    }
  }
  return out;
}
