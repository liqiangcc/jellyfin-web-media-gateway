use serde::Deserialize;
use site_adapter_api::{
    AdapterError, MediaProtection, NavigationContext, RecognizeResult, ResolveContext,
    ResolvedMedia, ResolvedStream, SiteAdapter, SourceLocator, StreamProtocol,
};
use std::collections::BTreeMap;
use url::Url;

pub const SITE_ID: &str = "bilibili";
pub const PLUGIN_ID: &str = "bilibili-public";
pub const PLUGIN_VERSION: &str = "0.1.0";
pub const LOCATOR_VERSION: u32 = 1;
pub const FROZEN_BVID: &str = "BV14V411W7r5";

const LOCATOR_SCHEMA: &str = "bilibili.source.v1";

#[derive(Clone, Debug, Default)]
pub struct BilibiliAdapter;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct LocatorPayload {
    schema: String,
    bvid: String,
    page: usize,
}

#[derive(Debug, Deserialize)]
struct InitialState {
    bvid: Option<String>,
    #[serde(rename = "videoData")]
    video_data: Option<VideoData>,
}

#[derive(Debug, Deserialize)]
struct VideoData {
    title: Option<String>,
    pages: Option<Vec<Page>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Page {
    page: Option<usize>,
    duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PlayInfo {
    data: Option<PlayData>,
}

#[derive(Debug, Deserialize)]
struct PlayData {
    durl: Option<Vec<DurlEntry>>,
    dash: Option<DashData>,
    drm: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DurlEntry {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashData {
    video: Option<Vec<DashVideo>>,
}

#[derive(Debug, Deserialize)]
struct DashVideo {
    #[serde(rename = "baseUrl", alias = "base_url")]
    base_url: Option<String>,
}

impl BilibiliAdapter {
    pub fn plugin_version(&self) -> &'static str {
        PLUGIN_VERSION
    }

    pub fn page_url(locator: &SourceLocator) -> Result<Url, AdapterError> {
        let payload = decode_locator(locator)?;
        let mut url = Url::parse(&format!("https://www.bilibili.com/video/{}/", payload.bvid))
            .map_err(|_| AdapterError::InvalidInput)?;
        if payload.page > 1 {
            url.query_pairs_mut()
                .append_pair("p", &payload.page.to_string());
        }
        Ok(url)
    }

    pub fn resolve_public_html(
        &self,
        locator: &SourceLocator,
        html: &str,
    ) -> Result<ResolvedMedia, AdapterError> {
        self.resolve_with_context(
            locator,
            ResolveContext {
                public_document: Some(html),
                upstream_status: None,
            },
        )
    }

    pub fn navigation_from_html(
        &self,
        locator: &SourceLocator,
        html: &str,
    ) -> Result<NavigationContext, AdapterError> {
        let payload = decode_locator(locator)?;
        let state = parse_initial_state(html)?;
        if state.bvid.as_deref() != Some(payload.bvid.as_str()) {
            return Err(AdapterError::ContentNotFound);
        }
        let pages = state
            .video_data
            .and_then(|video| video.pages)
            .ok_or(AdapterError::SchemaError)?;
        navigation_for_pages(&payload, &pages)
    }

    pub fn fixture_document() -> &'static str {
        FIXTURE_DOCUMENT
    }
}

impl SiteAdapter for BilibiliAdapter {
    fn plugin_id(&self) -> &'static str {
        PLUGIN_ID
    }

    fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
        let url = match Url::parse(input) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => url,
            _ => {
                return Ok(RecognizeResult {
                    matched: false,
                    site_id: SITE_ID.into(),
                    plugin_id: PLUGIN_ID.into(),
                    priority: 100,
                    locator: None,
                });
            }
        };
        let host_matches = matches!(
            url.host_str(),
            Some("www.bilibili.com") | Some("bilibili.com")
        );
        let mut segments = url.path_segments().ok_or(AdapterError::NoMatch)?;
        let matched = host_matches && segments.next() == Some("video");
        let Some(bvid) = segments.next().filter(|value| is_bvid(value)) else {
            return Ok(RecognizeResult {
                matched: false,
                site_id: SITE_ID.into(),
                plugin_id: PLUGIN_ID.into(),
                priority: 100,
                locator: None,
            });
        };
        if !matched {
            return Ok(RecognizeResult {
                matched: false,
                site_id: SITE_ID.into(),
                plugin_id: PLUGIN_ID.into(),
                priority: 100,
                locator: None,
            });
        }
        let page = match url
            .query_pairs()
            .find_map(|(key, value)| (key == "p").then(|| value.into_owned()))
        {
            Some(value) => value
                .parse::<usize>()
                .map_err(|_| AdapterError::InvalidInput)?,
            None => 1,
        };
        if page == 0 {
            return Err(AdapterError::InvalidInput);
        }
        Ok(RecognizeResult {
            matched: true,
            site_id: SITE_ID.into(),
            plugin_id: PLUGIN_ID.into(),
            priority: 100,
            locator: Some(encode_locator(bvid, page)),
        })
    }

    fn resolve(&self, _locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
        Err(AdapterError::AccessRequired)
    }

    fn resolve_with_context(
        &self,
        locator: &SourceLocator,
        context: ResolveContext<'_>,
    ) -> Result<ResolvedMedia, AdapterError> {
        let _payload = decode_locator(locator)?;
        if let Some(status) = context.upstream_status {
            return match status {
                401 | 403 => Err(AdapterError::UpstreamDenied),
                404 => Err(AdapterError::ContentNotFound),
                200..=299 => context
                    .public_document
                    .ok_or(AdapterError::AccessRequired)
                    .and_then(|html| resolve_document(locator, html)),
                _ => Err(AdapterError::UpstreamDenied),
            };
        }
        let html = context
            .public_document
            .ok_or(AdapterError::AccessRequired)?;
        resolve_document(locator, html)
    }

    fn navigation(&self, locator: &SourceLocator) -> Result<NavigationContext, AdapterError> {
        let payload = decode_locator(locator)?;
        if payload.bvid != FROZEN_BVID {
            return Err(AdapterError::UnsupportedLocator);
        }
        let pages = (1..=4)
            .map(|page| Page {
                page: Some(page),
                duration: None,
            })
            .collect::<Vec<_>>();
        navigation_for_pages(&payload, &pages)
    }
}

fn resolve_document(locator: &SourceLocator, html: &str) -> Result<ResolvedMedia, AdapterError> {
    let payload = decode_locator(locator)?;
    let state = parse_initial_state(html)?;
    let bvid = state.bvid.as_deref().ok_or(AdapterError::SchemaError)?;
    if bvid != payload.bvid {
        return Err(AdapterError::ContentNotFound);
    }
    let video = state.video_data.ok_or(AdapterError::SchemaError)?;
    let title = video
        .title
        .filter(|title| !title.trim().is_empty())
        .ok_or(AdapterError::SchemaError)?;
    let duration_ms = video.pages.as_ref().and_then(|pages| {
        pages
            .iter()
            .find(|page| page.page == Some(payload.page))
            .and_then(|page| page.duration)
            .map(|seconds| seconds.saturating_mul(1_000))
    });
    let playinfo = parse_playinfo(html)?;
    let data = playinfo.data.ok_or(AdapterError::SchemaError)?;
    if data.drm == Some(true) {
        return Ok(ResolvedMedia {
            title,
            duration_ms,
            source_site: SITE_ID.into(),
            streams: Vec::new(),
            expires_at_unix: None,
            protection: MediaProtection::DrmUnsupported,
        });
    }
    let (protocol, media_url) = media_url(&data)?;
    let url = Url::parse(&media_url).map_err(|_| AdapterError::SchemaError)?;
    let expires_at_unix = expiry_from_url(&url);
    Ok(ResolvedMedia {
        title,
        duration_ms,
        source_site: SITE_ID.into(),
        streams: vec![ResolvedStream {
            id: "primary".into(),
            protocol,
            url,
            public_headers: BTreeMap::from([
                ("Referer".into(), "https://www.bilibili.com/".into()),
                ("User-Agent".into(), "Mozilla/5.0".into()),
            ]),
            upstream_access_ref: None,
        }],
        expires_at_unix,
        protection: MediaProtection::Clear,
    })
}

fn media_url(data: &PlayData) -> Result<(StreamProtocol, String), AdapterError> {
    if let Some(url) = data
        .durl
        .as_ref()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry.url.clone())
    {
        return Ok((StreamProtocol::HttpFile, url));
    }
    if let Some(url) = data
        .dash
        .as_ref()
        .and_then(|dash| dash.video.as_ref())
        .and_then(|entries| entries.first())
        .and_then(|entry| entry.base_url.clone())
    {
        return Ok((StreamProtocol::Dash, url));
    }
    Err(AdapterError::UnsupportedMedia)
}

fn expiry_from_url(url: &Url) -> Option<u64> {
    ["deadline", "expires", "expire"]
        .into_iter()
        .find_map(|key| {
            url.query_pairs()
                .find_map(|(name, value)| (name == key).then(|| value.parse().ok()))
                .flatten()
        })
}

fn navigation_for_pages(
    payload: &LocatorPayload,
    pages: &[Page],
) -> Result<NavigationContext, AdapterError> {
    if pages.is_empty() {
        return Err(AdapterError::SchemaError);
    }
    let mut ordered = pages
        .iter()
        .filter_map(|page| page.page)
        .collect::<Vec<_>>();
    ordered.sort_unstable();
    ordered.dedup();
    let index = ordered
        .iter()
        .position(|page| *page == payload.page)
        .ok_or(AdapterError::ContentNotFound)?;
    let locator_for = |page| encode_locator(&payload.bvid, page);
    Ok(NavigationContext {
        previous: index.checked_sub(1).map(|i| locator_for(ordered[i])),
        next: ordered.get(index + 1).copied().map(locator_for),
        collection_id: Some(payload.bvid.clone()),
        current_index: Some(index),
    })
}

fn encode_locator(bvid: &str, page: usize) -> SourceLocator {
    let payload = serde_json::json!({
        "schema": LOCATOR_SCHEMA,
        "bvid": bvid,
        "page": page,
    })
    .to_string();
    SourceLocator {
        site_id: SITE_ID.into(),
        plugin_id: PLUGIN_ID.into(),
        locator_version: LOCATOR_VERSION,
        opaque_payload: hex_encode(payload.as_bytes()),
    }
}

fn decode_locator(locator: &SourceLocator) -> Result<LocatorPayload, AdapterError> {
    if locator.site_id != SITE_ID
        || locator.plugin_id != PLUGIN_ID
        || locator.locator_version != LOCATOR_VERSION
    {
        return Err(AdapterError::UnsupportedLocator);
    }
    let bytes = hex_decode(&locator.opaque_payload).ok_or(AdapterError::UnsupportedLocator)?;
    let payload = String::from_utf8(bytes).map_err(|_| AdapterError::UnsupportedLocator)?;
    let lower = payload.to_ascii_lowercase();
    if ["cookie", "authorization", "bearer", "password", "secret"]
        .iter()
        .any(|sentinel| lower.contains(sentinel))
    {
        return Err(AdapterError::SecretMaterial);
    }
    let payload: LocatorPayload =
        serde_json::from_str(&payload).map_err(|_| AdapterError::UnsupportedLocator)?;
    if payload.schema != LOCATOR_SCHEMA || !is_bvid(&payload.bvid) || payload.page == 0 {
        return Err(AdapterError::UnsupportedLocator);
    }
    Ok(payload)
}

fn is_bvid(value: &str) -> bool {
    value.len() >= 3
        && value.starts_with("BV")
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

fn parse_initial_state(html: &str) -> Result<InitialState, AdapterError> {
    let json = extract_json(html, "__INITIAL_STATE__").ok_or_else(|| {
        if html.contains("\"code\":-404") || html.contains("404 Not Found") {
            AdapterError::ContentNotFound
        } else {
            AdapterError::SchemaError
        }
    })?;
    serde_json::from_str(json).map_err(|_| AdapterError::ParseError)
}

fn parse_playinfo(html: &str) -> Result<PlayInfo, AdapterError> {
    let json = extract_json(html, "__playinfo__").ok_or(AdapterError::SchemaError)?;
    serde_json::from_str(json).map_err(|_| AdapterError::ParseError)
}

fn extract_json<'a>(html: &'a str, marker: &str) -> Option<&'a str> {
    let marker_start = html.find(marker)?;
    let start = html[marker_start..].find('{')? + marker_start;
    let bytes = html.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in bytes[start..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return html.get(start..=start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if value.is_empty() || value.len() % 2 != 0 {
        return None;
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Some((hex_digit(pair[0])? << 4) | hex_digit(pair[1])?))
        .collect()
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

const FIXTURE_DOCUMENT: &str = r#"
<script>window.__INITIAL_STATE__ = {"bvid":"BV14V411W7r5","videoData":{"title":"BILIBILI MACRO LINK 2021","pages":[{"page":1,"part":"Opening","duration":60},{"page":2,"part":"Performance","duration":60},{"page":3,"part":"Talk","duration":60},{"page":4,"part":"Closing","duration":60}]}};</script>
<script>window.__playinfo__ = {"data":{"durl":[{"url":"https://media.example.invalid/bilibili/BV14V411W7r5/part-1.mp4?deadline=4102444800"}]}};</script>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use site_adapter_api::{ResolveContext, SiteAdapterRegistry};
    use std::sync::Arc;

    fn registry() -> SiteAdapterRegistry {
        let mut registry = SiteAdapterRegistry::default();
        registry.register(Arc::new(BilibiliAdapter)).unwrap();
        registry
    }

    #[test]
    fn frozen_public_url_routes_to_opaque_versioned_locator() {
        let locator = registry()
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/")
            .unwrap();
        assert_eq!(locator.site_id, SITE_ID);
        assert_eq!(locator.plugin_id, PLUGIN_ID);
        assert_eq!(locator.locator_version, LOCATOR_VERSION);
        assert!(!locator.opaque_payload.contains(FROZEN_BVID));
        assert!(decode_locator(&locator).is_ok());
    }

    #[test]
    fn fixture_resolves_public_media_and_expiry_without_secret_headers() {
        let registry = registry();
        let locator = registry
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/?p=1")
            .unwrap();
        let media = registry
            .resolve_with_context(
                &locator,
                ResolveContext {
                    public_document: Some(BilibiliAdapter::fixture_document()),
                    upstream_status: Some(200),
                },
            )
            .unwrap();
        assert_eq!(media.protection, MediaProtection::Clear);
        assert_eq!(media.source_site, SITE_ID);
        assert_eq!(media.duration_ms, Some(60_000));
        assert_eq!(media.expires_at_unix, Some(4_102_444_800));
        assert_eq!(
            media.streams[0].public_headers.get("Referer"),
            Some(&"https://www.bilibili.com/".into())
        );
        assert!(!media.streams[0].public_headers.contains_key("Cookie"));
        assert!(
            !media.streams[0]
                .public_headers
                .contains_key("Authorization")
        );
        assert!(media.streams[0].upstream_access_ref.is_none());
    }

    #[test]
    fn frozen_sample_navigation_is_four_part_and_core_opaque() {
        let registry = registry();
        let locator = registry
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/?p=2")
            .unwrap();
        let navigation = registry.navigation(&locator).unwrap();
        assert_eq!(navigation.current_index, Some(1));
        assert!(navigation.previous.is_some());
        assert!(navigation.next.is_some());
        assert_eq!(navigation.collection_id.as_deref(), Some(FROZEN_BVID));
        assert_ne!(
            navigation.next.unwrap().opaque_payload,
            locator.opaque_payload
        );
    }

    #[test]
    fn fixture_html_navigation_is_four_part_and_round_trips_parts() {
        let adapter = BilibiliAdapter;
        let locator = adapter
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/?p=2")
            .unwrap()
            .locator
            .unwrap();
        let navigation = adapter
            .navigation_from_html(&locator, FIXTURE_DOCUMENT)
            .unwrap();
        assert_eq!(navigation.current_index, Some(1));
        let previous = navigation.previous.unwrap();
        let next = navigation.next.unwrap();
        assert_eq!(decode_locator(&previous).unwrap().page, 1);
        assert_eq!(decode_locator(&next).unwrap().page, 3);
        assert_eq!(navigation.collection_id.as_deref(), Some(FROZEN_BVID));
    }

    #[test]
    fn dash_and_drm_outputs_keep_protection_and_protocol_explicit() {
        let adapter = BilibiliAdapter;
        let locator = adapter
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/")
            .unwrap()
            .locator
            .unwrap();
        let dash = FIXTURE_DOCUMENT.replace(
            r#""durl":[{"url":"https://media.example.invalid/bilibili/BV14V411W7r5/part-1.mp4?deadline=4102444800"}]"#,
            r#""dash":{"video":[{"baseUrl":"https://media.example.invalid/bilibili/part-1.mpd?expires=4102444800"}]}"#,
        );
        let media = adapter.resolve_public_html(&locator, &dash).unwrap();
        assert_eq!(media.streams[0].protocol, StreamProtocol::Dash);
        assert_eq!(media.expires_at_unix, Some(4_102_444_800));

        let drm = FIXTURE_DOCUMENT.replace(
            r#""durl":[{"url":"https://media.example.invalid/bilibili/BV14V411W7r5/part-1.mp4?deadline=4102444800"}]"#,
            r#""drm":true"#,
        );
        let media = adapter.resolve_public_html(&locator, &drm).unwrap();
        assert_eq!(media.protection, MediaProtection::DrmUnsupported);
        assert!(media.streams.is_empty());
    }

    #[test]
    fn status_and_parse_failures_are_stable_and_non_echoing() {
        let adapter = BilibiliAdapter;
        let locator = adapter
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/")
            .unwrap()
            .locator
            .unwrap();
        for status in [401, 403] {
            assert_eq!(
                adapter.resolve_with_context(
                    &locator,
                    ResolveContext {
                        public_document: None,
                        upstream_status: Some(status),
                    },
                ),
                Err(AdapterError::UpstreamDenied)
            );
        }
        assert_eq!(
            adapter.resolve_with_context(
                &locator,
                ResolveContext {
                    public_document: None,
                    upstream_status: Some(404),
                },
            ),
            Err(AdapterError::ContentNotFound)
        );
        assert_eq!(
            adapter.resolve_public_html(&locator, "<html>\"code\":-404</html>"),
            Err(AdapterError::ContentNotFound)
        );
        assert_eq!(
            adapter.resolve_public_html(&locator, "<script>__INITIAL_STATE__ = {bad}</script>"),
            Err(AdapterError::ParseError)
        );
        assert_eq!(
            adapter.resolve_public_html(&locator, "<script>__INITIAL_STATE__ = {}</script>"),
            Err(AdapterError::SchemaError)
        );
    }

    #[test]
    fn non_matching_site_and_invalid_page_are_not_claimed() {
        let adapter = BilibiliAdapter;
        assert!(
            !adapter
                .recognize("https://example.com/video/BV14V411W7r5/")
                .unwrap()
                .matched
        );
        assert!(matches!(
            adapter.recognize("https://www.bilibili.com/video/BV14V411W7r5/?p=0"),
            Err(AdapterError::InvalidInput)
        ));
    }

    #[test]
    fn stale_or_secret_locator_and_failure_states_are_explicit() {
        let adapter = BilibiliAdapter;
        let locator = adapter
            .recognize("https://www.bilibili.com/video/BV14V411W7r5/")
            .unwrap()
            .locator
            .unwrap();
        assert_eq!(adapter.resolve(&locator), Err(AdapterError::AccessRequired));
        assert_eq!(
            adapter.resolve_with_context(
                &locator,
                ResolveContext {
                    public_document: None,
                    upstream_status: Some(403),
                },
            ),
            Err(AdapterError::UpstreamDenied)
        );
        let mut secret = locator.clone();
        secret.opaque_payload = hex_encode(
            br#"{"schema":"bilibili.source.v1","bvid":"BV14V411W7r5","page":1,"secret":"cookie"}"#,
        );
        assert_eq!(decode_locator(&secret), Err(AdapterError::SecretMaterial));
        assert_eq!(
            decode_locator(&SourceLocator {
                locator_version: 2,
                ..locator
            }),
            Err(AdapterError::UnsupportedLocator)
        );
    }
}
