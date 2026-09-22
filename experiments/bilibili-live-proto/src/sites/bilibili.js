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
  const { bvid, page, ep_id } = locator.opaque_payload;

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
  let durlRes;
  try {
    durlRes = await api(
      `/x/player/playurl?bvid=${bvid}&cid=${cid}&fnval=0&platform=html5`,
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
  if (prefer !== 'dash' && durlRes.durl?.length) {
    return {
      title: entry.part,
      duration: entry.duration,
      source_site: SITE_ID,
      protection: 'clear',
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

  // fnval=16: DASH — separate video/audio candidates.
  const dashRes = await api(
    `/x/player/playurl?bvid=${bvid}&cid=${cid}&fnval=16&fnver=0`,
  );
  const video = dashRes.dash?.video?.[0];
  const audio = dashRes.dash?.audio?.[0];
  if (!video || !audio) {
    throw new Error('NO_STREAMS');
  }
  return {
    title: entry.part,
    duration: entry.duration,
    source_site: SITE_ID,
    protection: 'clear',
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
