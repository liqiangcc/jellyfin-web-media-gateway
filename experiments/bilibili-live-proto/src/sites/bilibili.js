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

/** URL → SourceLocator. Recognizes bv/BV ids and an optional ?p= part. */
export function recognize(input) {
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

async function api(path) {
  const res = await fetch(`https://api.bilibili.com${path}`, {
    headers: { 'User-Agent': UA, Referer: REFERER },
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
  const { bvid, page } = locator.opaque_payload;
  const pages = await api(`/x/player/pagelist?bvid=${bvid}`);
  const entry = pages[Math.min(page, pages.length) - 1];
  if (!entry) throw new Error('PAGE_NOT_FOUND');
  const cid = entry.cid;

  // fnval=0: legacy durl — returns a muxed MP4 for anonymous/public content.
  const durlRes = await api(
    `/x/player/playurl?bvid=${bvid}&cid=${cid}&fnval=0&platform=html5`,
  );
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
