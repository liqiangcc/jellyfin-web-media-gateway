// Prototype Bilibili SiteAdapter — anonymous public path only.
// Mirrors the SiteAdapter contract surface (recognize / resolve /
// navigation) so evidence maps directly onto the real implementation.

const UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36';
const REFERER = 'https://www.bilibili.com';

const PLUGIN_ID = 'bilibili-proto';
const SITE_ID = 'bilibili';
const LOCATOR_VERSION = 1;

export const manifest = { plugin_id: PLUGIN_ID, site_id: SITE_ID };

/** URL → SourceLocator. Recognizes BV ids (+?p=) and bangumi ep ids. */
export function recognize(input) {
  // b23.tv short links — what the Bilibili app shares produce. Resolve
  // via HEAD redirect, then recognize the canonical URL.
  if (/b23\.tv\//i.test(input)) {
    return {
      matched: true,
      site_id: SITE_ID,
      plugin_id: PLUGIN_ID,
      short: true,
      locator: null,
    };
  }
  const ep = /bangumi\/play\/ep(\d+)/i.exec(input);
  if (ep) {
    return {
      matched: true,
      site_id: SITE_ID,
      plugin_id: PLUGIN_ID,
      locator: {
        site_id: SITE_ID,
        plugin_id: PLUGIN_ID,
        locator_version: LOCATOR_VERSION,
        opaque_payload: { ep_id: Number(ep[1]) },
      },
    };
  }
  // Live room: live.bilibili.com/<room_id>
  const live = /live\.bilibili\.com\/(?:h5\/)?(\d+)/i.exec(input);
  if (live) {
    return {
      matched: true,
      site_id: SITE_ID,
      plugin_id: PLUGIN_ID,
      locator: {
        site_id: SITE_ID,
        plugin_id: PLUGIN_ID,
        locator_version: LOCATOR_VERSION,
        opaque_payload: { room_id: Number(live[1]) },
      },
    };
  }
  const match = /(?:^|\/)(BV[0-9A-Za-z]{10})(?:[/?#]|$)/i.exec(input);
  if (!match) return { matched: false };
  let page = 1;
  try {
    const url = new URL(input);
    page = Math.max(1, parseInt(url.searchParams.get('p') || '1', 10) || 1);
  } catch {
    /* bare BV id is fine */
  }
  return {
    matched: true,
    site_id: SITE_ID,
    plugin_id: PLUGIN_ID,
    locator: {
      site_id: SITE_ID,
      plugin_id: PLUGIN_ID,
      locator_version: LOCATOR_VERSION,
      opaque_payload: { bvid: match[1], page },
    },
  };
}

/** Follow a b23.tv short link and return the canonical URL it points to. */
export async function resolveShortLink(url) {
  const r = await fetch(url, {
    method: 'HEAD',
    redirect: 'follow',
    headers: { 'User-Agent': UA },
  });
  return r.url;
}

// --- auth state (prototype-local; real impl owns this in Vault) ---
let authCookie = null;

export function setAuthCookie(c) {
  authCookie = c;
}
export function getAuthCookie() {
  return authCookie;
}
export function loggedIn() {
  return !!authCookie;
}

/**
 * Passport QR login. generate() → {qrcode_key, url}; poll(key) →
 * {code, cookies} where code 0 = confirmed, 86038 = expired,
 * 86090 = scanned-not-confirmed, 86101 = waiting.
 */
export async function qrLoginStart() {
  const res = await fetch(
    'https://passport.bilibili.com/x/passport-login/web/qrcode/generate',
    { headers: { 'User-Agent': UA } },
  ).then((r) => r.json());
  if (res.code !== 0) throw new Error(`qr generate -> ${res.code}`);
  return res.data; // {url, qrcode_key}
}

export async function qrLoginPoll(qrcode_key) {
  const res = await fetch(
    `https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key=${qrcode_key}`,
    { headers: { 'User-Agent': UA } },
  );
  const body = await res.json();
  const out = { code: body.data?.code ?? body.code, message: body.message };
  // Confirmed login: cookies arrive via Set-Cookie AND the redirect url
  // carries them as query params (SESSDATA etc.).
  if (out.code === 0) {
    const fromHeaders = (res.headers.getSetCookie?.() || [])
      .map((c) => c.split(';')[0])
      .filter(Boolean)
      .join('; ');
    const fromUrl = new URL(body.data.url).searchParams;
    const parts = [];
    for (const k of ['SESSDATA', 'bili_jct', 'DedeUserID']) {
      const v = fromUrl.get(k);
      if (v) parts.push(`${k}=${v}`);
    }
    out.cookies = fromHeaders || parts.join('; ');
  }
  return out;
}

async function api(path) {
  const res = await fetch(`https://api.bilibili.com${path}`, {
    headers: {
      'User-Agent': UA,
      Referer: REFERER,
      ...(authCookie ? { Cookie: authCookie } : {}),
    },
  });
  const body = await res.json();
  if (body.code !== 0) {
    throw new Error(`bilibili api ${path} -> code ${body.code} ${body.message}`);
  }
  return body.data;
}

/** SourceLocator → ResolvedMedia. Prefers muxed durl; falls back to DASH A/V. */
export async function resolve(locator, { prefer } = {}) {
  if (locator.locator_version !== LOCATOR_VERSION) {
    throw new Error('SOURCE_LOCATOR_UNSUPPORTED');
  }
  const { bvid, page, ep_id, room_id } = locator.opaque_payload;

  // Live path: getRoomPlayInfo → prefer http_hls/ts/avc (plays natively
  // on Safari, hls.js elsewhere). Live streams are infinite, no duration.
  if (room_id) {
    const headers = {
      'User-Agent': UA,
      Referer: 'https://live.bilibili.com',
      ...(authCookie ? { Cookie: authCookie } : {}),
    };
    const d = await fetch(
      `https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo?room_id=${room_id}&protocol=0,1&format=0,1,2&codec=0,1&platform=h5`,
      { headers },
    ).then((r) => r.json());
    if (d.data?.live_status !== 1) throw new Error('LIVE_OFFLINE');
    const pi = d.data.playurl_info || {};
    // Collect HLS candidates; prefer fmp4/ts + avc (Safari-friendly),
    // fall back to hevc variants. Probe each with a short timeout —
    // dead variants (some rooms serve stale CDN URLs) are skipped.
    const cands = [];
    for (const s of pi.playurl?.stream || []) {
      if (s.protocol_name !== 'http_hls') continue;
      for (const f of s.format || []) {
        for (const c of f.codec || []) {
          const u = c.url_info?.[0];
          if (!u) continue;
          const score =
            (f.format_name === 'fmp4' ? 0 : 10) +
            (c.codec_name === 'avc' ? 0 : 1);
          cands.push({
            score,
            url: u.host + c.base_url + (u.extra || ''),
            fmt: `${f.format_name}/${c.codec_name}`,
          });
        }
      }
    }
    cands.sort((a, b) => a.score - b.score);
    let picked = null;
    for (const c of cands) {
      try {
        const r = await fetch(c.url, {
          headers,
          signal: AbortSignal.timeout(5000),
        });
        r.body?.cancel();
        if (r.ok) {
          picked = c;
          break;
        }
      } catch {
        /* dead candidate */
      }
    }
    if (!picked) throw new Error('NO_STREAMS');
    return {
      title: `live room ${room_id} (${picked.fmt})`,
      duration: null, // live — no duration
      live: true,
      source_site: SITE_ID,
      protection: 'clear',
      streams: [
        {
          id: 'live-hls',
          kind: 'muxed',
          protocol: 'hls_live',
          url: picked.url,
          upstream_headers: { 'User-Agent': UA, Referer: 'https://live.bilibili.com' },
        },
      ],
    };
  }

  // Bangumi path: pgc endpoint, ep_id instead of bvid+cid.
  if (ep_id) {
    const headers = {
      'User-Agent': UA,
      Referer: REFERER,
      ...(authCookie ? { Cookie: authCookie } : {}),
    };
    const p = await fetch(
      `https://api.bilibili.com/pgc/player/web/playurl?ep_id=${ep_id}&fnval=0`,
      { headers },
    ).then((r) => r.json());
    const r0 = p.result || {};
    if (!r0.durl?.length) throw new Error('NO_STREAMS');
    return {
      title: `ep${ep_id}${r0.is_preview ? ' (preview)' : ''}`,
      duration: (r0.timelength || r0.durl[0].length) / 1000,
      source_site: SITE_ID,
      protection: 'clear',
      is_preview: !!r0.is_preview,
      streams: r0.durl.map((d, i) => ({
        id: `durl-${i}`,
        kind: 'muxed',
        protocol: 'http_file',
        url: d.url,
        upstream_headers: { 'User-Agent': UA, Referer: REFERER },
        size: d.size,
      })),
    };
  }
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const entry = pages[Math.min(page, pages.length) - 1];
  if (!entry) throw new Error('PAGE_NOT_FOUND');
  const cid = entry.cid;

  // fnval=0: legacy durl — returns a muxed MP4 for anonymous/public content.
  // If the BV is actually a bangumi (playurl -404), hop through view's
  // redirect_url (needs login) and resolve via the pgc path.
  const qn = prefer?.qn ? `&qn=${prefer.qn}` : '';
  let durlRes;
  try {
    durlRes = await api(
      `/x/player/playurl?bvid=${bvid}&cid=${cid}&fnval=0&platform=html5${qn}`,
    );
  } catch (e) {
    if (authCookie && /-404/.test(e.message)) {
      const v = await api(`/x/web-interface/view?bvid=${bvid}`);
      const ep = /bangumi\/play\/ep(\d+)/.exec(v.redirect_url || '');
      if (ep) {
        return resolve(
          {
            ...locator,
            opaque_payload: { ep_id: Number(ep[1]) },
          },
          { prefer },
        );
      }
    }
    throw e;
  }
  // Quality ladder for the control UI: accept_quality ids + descriptions.
  // Anonymous caps at 480p regardless of the advertised ladder — the UI
  // must filter by the streams actually returned.
  const quality_options = (durlRes.accept_quality || []).map((q, i) => ({
    qn: q,
    label: durlRes.accept_description?.[i] || `${q}p`,
  }));

  if (prefer?.mode !== 'dash' && durlRes.durl?.length) {
    return {
      title: entry.part,
      duration: entry.duration,
      source_site: SITE_ID,
      protection: 'clear',
      quality: durlRes.quality,
      quality_options,
      streams: durlRes.durl.map((d, i) => ({
        id: `durl-${i}`,
        kind: 'muxed',
        protocol: 'http_file',
        url: d.url,
        upstream_headers: { 'User-Agent': UA, Referer: REFERER },
        size: d.size,
      })),
    };
  }

  // fnval=16: DASH — separate video/audio candidates. qn selects the
  // video ladder entry; audio takes the best available.
  const dashRes = await api(
    `/x/player/playurl?bvid=${bvid}&cid=${cid}&fnval=16&fnver=0${qn}`,
  );
  const vids = dashRes.dash?.video || [];
  const video = prefer?.qn
    ? vids.find((x) => x.id === Number(prefer.qn)) || vids[0]
    : vids[0];
  const audio = dashRes.dash?.audio?.[0];
  if (!video || !audio) {
    throw new Error('NO_STREAMS');
  }
  const dashQ = vids.map((x) => ({ qn: x.id, label: `${x.height}p` }));
  return {
    title: entry.part,
    duration: entry.duration,
    source_site: SITE_ID,
    protection: 'clear',
    quality: video.id,
    quality_options: dashQ.length ? dashQ : quality_options,
    streams: [
      {
        id: 'dash-video',
        kind: 'video',
        protocol: 'dash',
        url: video.baseUrl,
        upstream_headers: { 'User-Agent': UA, Referer: REFERER },
        width: video.width,
        height: video.height,
        bitrate: video.bandwidth,
      },
      {
        id: 'dash-audio',
        kind: 'audio',
        protocol: 'dash',
        url: audio.baseUrl,
        upstream_headers: { 'User-Agent': UA, Referer: REFERER },
        bitrate: audio.bandwidth,
      },
    ],
  };
}

/**
 * Discovery surfaces — all anonymous-reachable:
 *  popular() → 热门视频; related(bvid) → 相关推荐; rcmd() → feed
 *  (personalized when logged in). Returns [{bvid,title,duration,cover}].
 */
export async function discovery(kind = 'popular', bvid = null) {
  const headers = {
    'User-Agent': UA,
    Referer: REFERER,
    ...(authCookie ? { Cookie: authCookie } : {}),
  };
  const map = (list) =>
    (list || []).map((v) => ({
      bvid: v.bvid,
      title: v.title,
      duration: v.duration,
      cover: v.pic || v.cover,
      owner: v.owner?.name,
      stat: v.stat?.view,
    }));
  if (kind === 'related' && bvid) {
    const d = await fetch(
      `https://api.bilibili.com/x/web-interface/archive/related?bvid=${bvid}`,
      { headers },
    ).then((r) => r.json());
    return map(d.data);
  }
  if (kind === 'rcmd') {
    const d = await fetch(
      'https://api.bilibili.com/x/web-interface/index/top/feed/rcmd',
      { headers },
    ).then((r) => r.json());
    return map(d.data?.item);
  }
  if (kind === 'ranking') {
    const d = await fetch(
      'https://api.bilibili.com/x/web-interface/ranking/v2?rid=0&type=all',
      { headers },
    ).then((r) => r.json());
    return map(d.data?.list);
  }
  const d = await fetch(
    'https://api.bilibili.com/x/web-interface/popular?ps=40',
    { headers },
  ).then((r) => r.json());
  return map(d.data?.list);
}

/**
 * Subtitle tracks for a locator's page via x/player/v2.
 * Returns [{lan, lan_doc, url}] — url is protocol-relative hdslb JSON.
 */
export async function subtitles(locator) {
  const { bvid, page = 1 } = locator.opaque_payload;
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const entry = pages[Math.min(page, pages.length) - 1];
  if (!entry) return [];
  const d = await api(`/x/player/v2?bvid=${bvid}&cid=${entry.cid}`);
  return (d.subtitle?.subtitles || []).map((s) => ({
    lan: s.lan,
    label: s.lan_doc,
    url: s.subtitle_url?.startsWith('//')
      ? 'https:' + s.subtitle_url
      : s.subtitle_url,
  }));
}

/**
 * Post a scroll danmaku at the current position. Requires login —
 * bili_jct in the cookie doubles as the CSRF token.
 */
export async function sendDanmaku(locator, text, progressSec = 0) {
  if (!authCookie) throw new Error('NOT_LOGGED_IN');
  const { bvid, page = 1 } = locator.opaque_payload;
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const entry = pages[Math.min(page, pages.length) - 1];
  if (!entry) throw new Error('PAGE_NOT_FOUND');
  let aid = entry.aid;
  if (!aid) {
    const v = await api(`/x/web-interface/view?bvid=${bvid}`);
    aid = v.aid;
  }
  const csrf = /bili_jct=([^;]+)/.exec(authCookie)?.[1];
  if (!csrf) throw new Error('NO_CSRF');
  const body = new URLSearchParams({
    type: '1',
    oid: String(entry.cid),
    aid: String(aid || ''),
    bvid,
    msg: text,
    progress: String(Math.floor(progressSec * 1000)),
    color: '16777215',
    fontsize: '25',
    mode: '1',
    rnd: String(Math.floor(Date.now() / 1000)),
    csrf,
  });
  const r = await fetch('https://api.bilibili.com/x/v2/dm/post', {
    method: 'POST',
    headers: {
      'User-Agent': UA,
      Referer: `https://www.bilibili.com/video/${bvid}`,
      Cookie: authCookie,
      'content-type': 'application/x-www-form-urlencoded',
    },
    body,
  }).then((x) => x.json());
  if (r.code !== 0) throw new Error(`DM_POST ${r.code} ${r.message || ''}`);
  return { ok: true };
}

/**
 * Fetch one subtitle track's cue list: [{from,to,content}].
 */
export async function subtitleCues(url) {
  const headers = {
    'User-Agent': UA,
    Referer: REFERER,
    ...(authCookie ? { Cookie: authCookie } : {}),
  };
  const d = await fetch(url, { headers }).then((r) => r.json());
  return (d.body || []).map((c) => ({
    from: c.from,
    to: c.to,
    text: c.content,
  }));
}

/**
 * List the logged-in user's favorite folders, or the videos inside one.
 * Requires login. folderId=null → folder list; else → [{bvid,title,duration,cover}].
 */
export async function favorites(folderId = null) {
  if (!authCookie) throw new Error('NOT_LOGGED_IN');
  const headers = { 'User-Agent': UA, Referer: REFERER, Cookie: authCookie };
  if (folderId === null) {
    const nav = await fetch(
      'https://api.bilibili.com/x/web-interface/nav',
      { headers },
    ).then((r) => r.json());
    const mid = nav.data?.mid;
    const d = await fetch(
      `https://api.bilibili.com/x/v3/fav/folder/created/list-all?up_mid=${mid}`,
      { headers },
    ).then((r) => r.json());
    return (d.data?.list || []).map((f) => ({
      id: f.id,
      title: f.title,
      count: f.media_count,
    }));
  }
  const d = await fetch(
    `https://api.bilibili.com/x/v3/fav/resource/list?media_id=${folderId}&ps=40&platform=web`,
    { headers },
  ).then((r) => r.json());
  return (d.data?.medias || []).map((m) => ({
    bvid: m.bvid,
    title: m.title,
    duration: `${Math.floor(m.duration / 60)}:${String(m.duration % 60).padStart(2, '0')}`,
    cover: m.cover,
    pages: m.page,
  }));
}

/**
 * Fetch the full danmaku pool for a locator's page as a normalized list.
 * `x/v1/dm/list.so` returns XML; anonymous-reachable on this host.
 * Each entry: { t: seconds, mode, color, text } sorted by t.
 */
export async function danmaku(locator) {
  const { bvid, page } = locator.opaque_payload;
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const entry = pages[Math.min(page, pages.length) - 1];
  if (!entry) throw new Error('PAGE_NOT_FOUND');
  const res = await fetch(
    `https://api.bilibili.com/x/v1/dm/list.so?oid=${entry.cid}`,
    { headers: { 'User-Agent': UA, Referer: REFERER } },
  );
  const xml = await res.text();
  const out = [];
  const re = /<d p="([^"]+)">([^<]*)<\/d>/g;
  let m;
  while ((m = re.exec(xml))) {
    const p = m[1].split(',');
    const t = Number(p[0]);
    const text = m[2].trim();
    if (Number.isFinite(t) && text) {
      out.push({ t, mode: Number(p[1]), color: Number(p[3]), text });
    }
  }
  out.sort((a, b) => a.t - b.t);
  return out;
}

/** previous/next/queue locators for multipart content. */
export async function navigation(locator) {
  const { bvid, page } = locator.opaque_payload;
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const mk = (p) =>
    p
      ? {
          site_id: SITE_ID,
          plugin_id: PLUGIN_ID,
          locator_version: LOCATOR_VERSION,
          opaque_payload: { bvid, page: p.page },
        }
      : null;
  const idx = pages.findIndex((p) => p.page === page);
  return {
    previous: mk(pages[idx - 1]),
    next: mk(pages[idx + 1]),
    queue: pages.map(mk),
    current_index: idx,
  };
}
