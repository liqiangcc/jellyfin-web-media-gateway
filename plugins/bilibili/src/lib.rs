//! Production Bilibili SiteAdapter boundary.
//!
//! This crate owns Bilibili URL, BVID and part semantics.  Acquisition is
//! intentionally mediated by the generic BrowserObservation plus a
//! server-owned handoff; this adapter never performs network I/O or receives
//! browser credentials.

use serde::{Deserialize, Serialize};
use site_adapter_api::{
    AdapterError, AuthenticatedSessionHandoff, BrowserAcquisitionTarget,
    BROWSER_AUTH_OBSERVATION_VERSION,
    BrowserAuthObservation, BrowserAuthState, BrowserExpiryHint, BrowserMediaKind,
    BrowserObservation, BrowserStatusClass, MediaProtection, MediaShapeV1, MediaTrack,
    MediaTrackKind, NavigationContext, RecognizeResult, ResolveContext, ResolvedMedia,
    ResolvedStream, ServerOwnedObservation, SiteAdapter, SiteAdapterRegistry, SourceLocator,
    StreamProtocol, validate_browser_auth_observation, validate_browser_observation,
    validate_server_owned_observation,
};
use url::Url;

pub const SITE_ID: &str = "bilibili";
pub const PLUGIN_ID: &str = "bilibili";
pub const PLUGIN_VERSION: &str = "1.0.0";
pub const LOCATOR_VERSION: u32 = 1;
pub const RECOGNITION_PRIORITY: u16 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BilibiliAccountState {
    LoginRequired,
    Checking,
    Valid,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BilibiliAuthInterpretation {
    pub account_state: BilibiliAccountState,
    pub session_ready: bool,
}

const LOCATOR_SCHEMA: &str = "bilibili.source.v1";
const MAX_INPUT_BYTES: usize = 2048;
const MAX_LOCATOR_BYTES: usize = 1024;
const MAX_BVID_BYTES: usize = 12;
const MAX_TITLE_BYTES: usize = 512;

#[derive(Clone, Debug, Default)]
pub struct BilibiliAdapter;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LocatorPayload {
    schema: String,
    bvid: String,
    part: u16,
}

impl BilibiliAdapter {
    pub const fn new() -> Self {
        Self
    }

    pub fn plugin_version(&self) -> &'static str {
        PLUGIN_VERSION
    }

    /// Register the production adapter in a caller-owned registry.  The
    /// registry remains the only routing authority and duplicate IDs fail
    /// closed there.
    pub fn register(registry: &mut SiteAdapterRegistry) -> Result<(), AdapterError> {
        registry.register(std::sync::Arc::new(Self::new()))
    }

    pub fn page_url(&self, locator: &SourceLocator) -> Result<Url, AdapterError> {
        let payload = decode_locator(locator)?;
        let mut url = Url::parse(&format!("https://www.bilibili.com/video/{}/", payload.bvid))
            .map_err(|_| AdapterError::InvalidInput)?;
        if payload.part > 1 {
            url.query_pairs_mut()
                .append_pair("p", &payload.part.to_string());
        }
        Ok(url)
    }

    /// The direct method deliberately cannot acquire media.  Callers must
    /// supply both the generic observation and server-owned handoff so a
    /// plugin cannot invent an upstream URL or bypass the access boundary.
    pub fn resolve_observation(
        &self,
        locator: &SourceLocator,
        observation: &BrowserObservation,
        server_observation: &ServerOwnedObservation,
    ) -> Result<ResolvedMedia, AdapterError> {
        self.resolve_with_context(
            locator,
            ResolveContext {
                browser_observation: Some(observation),
                server_observation: Some(server_observation),
                authenticated_session: None,
            },
        )
    }

    /// Interpret the generic auth lifecycle in the site-owned account
    /// vocabulary. Core and the Browser Worker deliberately do not make this
    /// decision.
    pub fn interpret_auth_observation(
        &self,
        observation: &BrowserAuthObservation,
    ) -> Result<BilibiliAuthInterpretation, AdapterError> {
        validate_browser_auth_observation(observation)?;
        let interpretation = match observation.state {
            BrowserAuthState::Required | BrowserAuthState::Expired => BilibiliAuthInterpretation {
                account_state: BilibiliAccountState::LoginRequired,
                session_ready: false,
            },
            BrowserAuthState::InputNeeded => BilibiliAuthInterpretation {
                account_state: BilibiliAccountState::Checking,
                session_ready: false,
            },
            BrowserAuthState::CandidateReady => BilibiliAuthInterpretation {
                account_state: BilibiliAccountState::Valid,
                session_ready: true,
            },
            BrowserAuthState::Cancelled
            | BrowserAuthState::Crashed
            | BrowserAuthState::Disconnected
            | BrowserAuthState::TimedOut => BilibiliAuthInterpretation {
                account_state: BilibiliAccountState::Error,
                session_ready: false,
            },
        };
        Ok(interpretation)
    }

    /// Consume a server-owned candidate-session handoff. The handoff proves
    /// only that Core atomically accepted an opaque session reference; all
    /// Bilibili URL/media semantics remain in this adapter.
    pub fn resolve_authenticated(
        &self,
        locator: &SourceLocator,
        handoff: &AuthenticatedSessionHandoff,
        observation: &BrowserObservation,
        server_observation: &ServerOwnedObservation,
    ) -> Result<ResolvedMedia, AdapterError> {
        if handoff.schema_version() != BROWSER_AUTH_OBSERVATION_VERSION
            || handoff.site_id() != SITE_ID
            || handoff.session_ref().is_empty()
        {
            return Err(AdapterError::InvalidObservation);
        }
        let auth_observation = handoff.observation();
        let interpretation = self.interpret_auth_observation(&auth_observation)?;
        if !interpretation.session_ready {
            return Err(AdapterError::AccessRequired);
        }
        self.resolve_with_context(
            locator,
            ResolveContext {
                browser_observation: Some(observation),
                server_observation: Some(server_observation),
                authenticated_session: Some(handoff),
            },
        )
    }
}

impl SiteAdapter for BilibiliAdapter {
    fn site_id(&self) -> &'static str {
        SITE_ID
    }

    fn plugin_id(&self) -> &'static str {
        PLUGIN_ID
    }

    fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
        let unmatched = || RecognizeResult {
            matched: false,
            site_id: SITE_ID.into(),
            plugin_id: PLUGIN_ID.into(),
            priority: RECOGNITION_PRIORITY,
            locator: None,
        };
        if input.is_empty() || input.len() > MAX_INPUT_BYTES || input.chars().any(char::is_control)
        {
            return Ok(unmatched());
        }
        let url = match Url::parse(input) {
            Ok(url) => url,
            Err(_) => return Ok(unmatched()),
        };
        if url.scheme() != "https"
            || !matches!(
                url.host_str(),
                Some("www.bilibili.com") | Some("bilibili.com")
            )
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Ok(unmatched());
        }
        let mut segments: Vec<_> = url
            .path_segments()
            .map(|segments| segments.collect())
            .unwrap_or_default();
        if segments.last() == Some(&"") {
            segments.pop();
        }
        if segments.len() != 2 || segments[0] != "video" || !is_bvid(segments[1]) {
            return Ok(unmatched());
        }
        let mut part = 1_u16;
        let mut seen_part = false;
        for (key, value) in url.query_pairs() {
            if key != "p" || seen_part {
                return Ok(unmatched());
            }
            seen_part = true;
            part = match value.parse::<u16>() {
                Ok(value) if (1..=9999).contains(&value) => value,
                _ => return Ok(unmatched()),
            };
        }
        Ok(RecognizeResult {
            matched: true,
            site_id: SITE_ID.into(),
            plugin_id: PLUGIN_ID.into(),
            priority: RECOGNITION_PRIORITY,
            locator: Some(encode_locator(segments[1], part)),
        })
    }

    fn resolve(&self, _locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
        Err(AdapterError::ObservationRequired)
    }

    fn resolve_with_context(
        &self,
        locator: &SourceLocator,
        context: ResolveContext<'_>,
    ) -> Result<ResolvedMedia, AdapterError> {
        let _ = decode_locator(locator)?;
        let observation = context
            .browser_observation
            .ok_or(AdapterError::ObservationRequired)?;
        let server_observation = context
            .server_observation
            .ok_or(AdapterError::ObservationRequired)?;
        if let Some(authenticated_session) = context.authenticated_session {
            if authenticated_session.site_id() != SITE_ID
                || authenticated_session.observation().state != BrowserAuthState::CandidateReady
            {
                return Err(AdapterError::AccessRequired);
            }
        }
        validate_browser_observation(observation)?;
        validate_server_owned_observation(server_observation)?;
        if server_observation.observation_id != observation.observation_id {
            return Err(AdapterError::ContentNotFound);
        }
        let page_locator = self
            .recognize(&observation.page_url)?
            .locator
            .ok_or(AdapterError::ContentNotFound)?;
        if page_locator != *locator {
            return Err(AdapterError::ContentNotFound);
        }

        let mut muxed_candidate = None;
        let mut paired_candidates = std::collections::BTreeMap::<
            String,
            Vec<(
                &site_adapter_api::BrowserMediaCandidate,
                &site_adapter_api::ServerOwnedMedia,
            )>,
        >::new();
        for observed in &observation.candidates {
            if !matches!(
                observed.protocol,
                StreamProtocol::HttpFile | StreamProtocol::Hls | StreamProtocol::Dash
            ) || observed.status != BrowserStatusClass::Success
                || !observed.egress_allowed
                || observed.expiry == BrowserExpiryHint::Expired
            {
                continue;
            }
            let handoff = server_observation.media.iter().find(|media| {
                media.observation_id == observation.observation_id
                    && media.candidate_id == observed.id
                    && media.access_ref == observed.access_ref
                    && media.protocol == observed.protocol
            });
            if let Some(handoff) = handoff {
                match observed.kind {
                    BrowserMediaKind::Muxed => {
                        if muxed_candidate.is_none() {
                            muxed_candidate = Some((observed, handoff));
                        }
                    }
                    BrowserMediaKind::Video | BrowserMediaKind::Audio => {
                        if let Some(group_id) = observed.group_id.as_ref() {
                            paired_candidates
                                .entry(group_id.clone())
                                .or_default()
                                .push((observed, handoff));
                        }
                    }
                }
            }
        }

        let selected = if let Some(candidate) = muxed_candidate {
            vec![candidate]
        } else {
            let complete_groups: Vec<_> = paired_candidates
                .into_iter()
                .filter_map(|(group_id, candidates)| {
                    let has_video = candidates
                        .iter()
                        .any(|(candidate, _)| candidate.kind == BrowserMediaKind::Video);
                    let has_audio = candidates
                        .iter()
                        .any(|(candidate, _)| candidate.kind == BrowserMediaKind::Audio);
                    (candidates.len() == 2 && has_video && has_audio)
                        .then_some((group_id, candidates))
                })
                .collect();
            if complete_groups.len() == 1 {
                complete_groups
                    .into_iter()
                    .next()
                    .expect("complete group")
                    .1
            } else {
                Vec::new()
            }
        };
        if selected.is_empty() {
            return Err({
                if observation
                    .candidates
                    .iter()
                    .any(|candidate| candidate.expiry == BrowserExpiryHint::Expired)
                {
                    AdapterError::ObservationExpired
                } else if observation.candidates.iter().any(|candidate| {
                    matches!(
                        candidate.kind,
                        BrowserMediaKind::Video | BrowserMediaKind::Audio
                    )
                }) {
                    AdapterError::UnsupportedMedia
                } else if observation
                    .candidates
                    .iter()
                    .any(|candidate| !candidate.egress_allowed)
                {
                    AdapterError::EgressRejected
                } else {
                    AdapterError::UnsupportedMedia
                }
            });
        }
        if selected
            .iter()
            .any(|(_, handoff)| handoff.url.host_str().is_none())
        {
            return Err(AdapterError::InvalidObservation);
        }
        let streams = selected
            .iter()
            .map(|(observed, handoff)| ResolvedStream {
                id: observed.id.clone(),
                protocol: handoff.protocol,
                url: handoff.url.clone(),
                public_headers: handoff.public_headers.clone(),
                // The URL/ref are held in the server-owned handoff. Do not
                // copy the opaque capability reference into display media.
                upstream_access_ref: None,
            })
            .collect::<Vec<_>>();
        let shape = MediaShapeV1 {
            version: site_adapter_api::MEDIA_SHAPE_VERSION,
            tracks: selected
                .iter()
                .map(|(observed, handoff)| MediaTrack {
                    id: observed.id.clone(),
                    kind: MediaTrackKind::from(observed.kind),
                    group_id: observed.group_id.clone(),
                    protocol: handoff.protocol,
                    codec: observed.codec.clone(),
                    container: observed.container.clone(),
                    mime_type: observed.mime_type.clone(),
                    width: observed.width,
                    height: observed.height,
                    bitrate: observed.bitrate,
                    language: observed.language.clone(),
                    access_ref: Some(handoff.access_ref.clone()),
                    expires_at: observed.expires_at,
                })
                .collect(),
        };
        Ok(ResolvedMedia {
            title: bounded_title(&observation.page_title),
            source_site: SITE_ID.into(),
            streams,
            subtitles: Vec::new(),
            protection: MediaProtection::Clear,
            shape,
        })
    }

    fn browser_acquisition_target(
        &self,
        locator: &SourceLocator,
    ) -> Result<Option<BrowserAcquisitionTarget>, AdapterError> {
        // Page/BVID/part interpretation stays in this plugin. Core receives
        // only the bounded transport instruction after ownership validation.
        Ok(Some(BrowserAcquisitionTarget::new(self.page_url(locator)?)?))
    }

    fn navigation(&self, locator: &SourceLocator) -> Result<NavigationContext, AdapterError> {
        let _ = decode_locator(locator)?;
        Err(AdapterError::UnsupportedNavigation)
    }
}

fn bounded_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_TITLE_BYTES)
        .collect()
}

fn is_bvid(value: &str) -> bool {
    value.len() == MAX_BVID_BYTES
        && value.starts_with("BV")
        && value[2..]
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

fn encode_locator(bvid: &str, part: u16) -> SourceLocator {
    let payload = serde_json::to_vec(&LocatorPayload {
        schema: LOCATOR_SCHEMA.into(),
        bvid: bvid.into(),
        part,
    })
    .expect("fixed locator payload serializes");
    SourceLocator {
        site_id: SITE_ID.into(),
        plugin_id: PLUGIN_ID.into(),
        locator_version: LOCATOR_VERSION,
        opaque_payload: hex_encode(&payload),
    }
}

fn decode_locator(locator: &SourceLocator) -> Result<LocatorPayload, AdapterError> {
    if locator.site_id != SITE_ID
        || locator.plugin_id != PLUGIN_ID
        || locator.locator_version != LOCATOR_VERSION
        || locator.opaque_payload.is_empty()
        || locator.opaque_payload.len() > MAX_LOCATOR_BYTES
    {
        return Err(AdapterError::UnsupportedLocator);
    }
    let bytes = hex_decode(&locator.opaque_payload).ok_or(AdapterError::UnsupportedLocator)?;
    let text = String::from_utf8(bytes).map_err(|_| AdapterError::UnsupportedLocator)?;
    let lower = text.to_ascii_lowercase();
    if [
        "cookie",
        "authorization",
        "bearer",
        "sessdata",
        "token",
        "password",
        "secret",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return Err(AdapterError::SecretMaterial);
    }
    let payload: LocatorPayload =
        serde_json::from_str(&text).map_err(|_| AdapterError::UnsupportedLocator)?;
    if payload.schema != LOCATOR_SCHEMA
        || !is_bvid(&payload.bvid)
        || !(1..=9999).contains(&payload.part)
    {
        return Err(AdapterError::UnsupportedLocator);
    }
    Ok(payload)
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

#[cfg(test)]
mod tests {
    use super::*;
    use site_adapter_api::{
        BROWSER_OBSERVATION_VERSION, BrowserRangeSupport, ServerOwnedMedia,
        conformance::assert_error_diagnostics_bounded,
    };
    use std::collections::BTreeMap;

    const BVID: &str = "BV1xx411c7mD";

    fn registry() -> SiteAdapterRegistry {
        let mut registry = SiteAdapterRegistry::default();
        BilibiliAdapter::register(&mut registry).unwrap();
        registry
    }

    fn locator(part: u16) -> SourceLocator {
        registry()
            .recognize(&format!("https://www.bilibili.com/video/{BVID}/?p={part}"))
            .unwrap()
    }

    fn observation(part: u16, candidate: &str) -> BrowserObservation {
        BrowserObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "obs-1".into(),
            page_url: format!("https://www.bilibili.com/video/{BVID}/?p={part}"),
            page_title: "Synthetic Bilibili fixture".into(),
            event_count: 4,
            resource_count: 1,
            candidates: vec![site_adapter_api::BrowserMediaCandidate {
                id: candidate.into(),
                kind: BrowserMediaKind::Muxed,
                group_id: None,
                codec: None,
                container: Some("mp4".into()),
                mime_type: Some("video/mp4".into()),
                width: None,
                height: None,
                bitrate: None,
                language: None,
                protocol: StreamProtocol::HttpFile,
                status: BrowserStatusClass::Success,
                range: BrowserRangeSupport::Supported,
                egress_allowed: true,
                access_ref: "media-ref-1".into(),
                expiry: BrowserExpiryHint::NoneObserved,
                expires_at: None,
            }],
        }
    }

    fn server() -> ServerOwnedObservation {
        ServerOwnedObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "obs-1".into(),
            media: vec![ServerOwnedMedia {
                observation_id: "obs-1".into(),
                candidate_id: "primary".into(),
                access_ref: "media-ref-1".into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://cdn.example.invalid/media.mp4?expires=4102444800")
                    .unwrap(),
                public_headers: BTreeMap::new(),
            }],
        }
    }

    fn paired_observation() -> BrowserObservation {
        BrowserObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "obs-paired".into(),
            page_url: format!("https://www.bilibili.com/video/{BVID}/?p=1"),
            page_title: "Synthetic paired media".into(),
            event_count: 6,
            resource_count: 2,
            candidates: vec![
                site_adapter_api::BrowserMediaCandidate {
                    id: "video-track".into(),
                    kind: BrowserMediaKind::Video,
                    group_id: Some("av-main".into()),
                    codec: Some("avc1.640028".into()),
                    container: Some("fmp4".into()),
                    mime_type: Some("video/mp4".into()),
                    width: Some(1920),
                    height: Some(1080),
                    bitrate: Some(2_000_000),
                    language: None,
                    protocol: StreamProtocol::Dash,
                    status: BrowserStatusClass::Success,
                    range: BrowserRangeSupport::Supported,
                    egress_allowed: true,
                    access_ref: "paired-video-ref".into(),
                    expiry: BrowserExpiryHint::ShortLived,
                    expires_at: Some(site_adapter_api::MAX_MEDIA_EXPIRY_UNIX_SECONDS),
                },
                site_adapter_api::BrowserMediaCandidate {
                    id: "audio-track".into(),
                    kind: BrowserMediaKind::Audio,
                    group_id: Some("av-main".into()),
                    codec: Some("mp4a.40.2".into()),
                    container: Some("m4a".into()),
                    mime_type: Some("audio/mp4".into()),
                    width: None,
                    height: None,
                    bitrate: Some(128_000),
                    language: Some("und".into()),
                    protocol: StreamProtocol::Dash,
                    status: BrowserStatusClass::Success,
                    range: BrowserRangeSupport::Supported,
                    egress_allowed: true,
                    access_ref: "paired-audio-ref".into(),
                    expiry: BrowserExpiryHint::ShortLived,
                    expires_at: Some(site_adapter_api::MAX_MEDIA_EXPIRY_UNIX_SECONDS),
                },
            ],
        }
    }

    fn paired_server() -> ServerOwnedObservation {
        ServerOwnedObservation {
            schema_version: BROWSER_OBSERVATION_VERSION,
            observation_id: "obs-paired".into(),
            media: vec![
                ServerOwnedMedia {
                    observation_id: "obs-paired".into(),
                    candidate_id: "video-track".into(),
                    access_ref: "paired-video-ref".into(),
                    protocol: StreamProtocol::Dash,
                    url: Url::parse("https://media.example.invalid/video.init").unwrap(),
                    public_headers: BTreeMap::new(),
                },
                ServerOwnedMedia {
                    observation_id: "obs-paired".into(),
                    candidate_id: "audio-track".into(),
                    access_ref: "paired-audio-ref".into(),
                    protocol: StreamProtocol::Dash,
                    url: Url::parse("https://media.example.invalid/audio.init").unwrap(),
                    public_headers: BTreeMap::new(),
                },
            ],
        }
    }

    #[test]
    fn registry_recognizes_only_bounded_canonical_inputs() {
        let registry = registry();
        let locator = registry
            .recognize(&format!("https://www.bilibili.com/video/{BVID}/?p=2"))
            .unwrap();
        assert_eq!(locator.site_id, SITE_ID);
        assert_eq!(locator.plugin_id, PLUGIN_ID);
        assert_eq!(locator.locator_version, LOCATOR_VERSION);
        assert!(!locator.opaque_payload.contains(BVID));
        for input in [
            "http://www.bilibili.com/video/BV1xx411c7mD/",
            "https://www.bilibili.com/",
            "https://www.bilibili.com/video/BV1xx411c7mD/?p=0",
            "https://www.bilibili.com/video/BV1xx411c7mD/?p=2&token=secret",
        ] {
            assert_eq!(registry.recognize(input), Err(AdapterError::NoMatch));
        }
    }

    #[test]
    fn conformance_covers_determinism_ownership_and_version() {
        let adapter = BilibiliAdapter;
        let input = "https://www.bilibili.com/video/BV1xx411c7mD/?p=1";
        let first = adapter.recognize(input).unwrap();
        let second = adapter.recognize(input).unwrap();
        assert_eq!(first.matched, second.matched);
        assert_eq!(first.site_id, second.site_id);
        assert_eq!(first.plugin_id, second.plugin_id);
        assert_eq!(first.priority, second.priority);
        assert_eq!(first.locator, second.locator);
        assert!(first.matched);
        let locator = first.locator.unwrap();
        assert_eq!(locator.site_id, SITE_ID);
        assert_eq!(locator.plugin_id, PLUGIN_ID);
        assert_eq!(locator.locator_version, LOCATOR_VERSION);
        adapter
            .resolve_observation(&locator, &observation(1, "primary"), &server())
            .unwrap();
    }

    #[test]
    fn deterministic_observation_handoff_resolves_without_secret_output() {
        let adapter = BilibiliAdapter;
        let locator = locator(1);
        let media = adapter
            .resolve_observation(&locator, &observation(1, "primary"), &server())
            .unwrap();
        assert_eq!(media.source_site, SITE_ID);
        assert_eq!(media.protection, MediaProtection::Clear);
        assert!(media.streams[0].public_headers.is_empty());
        assert!(media.streams[0].upstream_access_ref.is_none());
        assert!(!format!("{media:?}").contains("media-ref-1"));
    }

    #[test]
    fn paired_video_audio_observation_projects_generic_shape() {
        let adapter = BilibiliAdapter;
        let media = adapter
            .resolve_observation(&locator(1), &paired_observation(), &paired_server())
            .unwrap();
        assert_eq!(media.streams.len(), 2);
        assert_eq!(media.shape.version, site_adapter_api::MEDIA_SHAPE_VERSION);
        assert_eq!(media.shape.tracks[0].kind, MediaTrackKind::Video);
        assert_eq!(media.shape.tracks[1].kind, MediaTrackKind::Audio);
        assert_eq!(media.shape.tracks[0].group_id.as_deref(), Some("av-main"));
        assert_eq!(media.shape.tracks[1].group_id.as_deref(), Some("av-main"));
        assert_eq!(media.shape.tracks[0].codec.as_deref(), Some("avc1.640028"));
        assert_eq!(media.shape.tracks[1].codec.as_deref(), Some("mp4a.40.2"));
        assert!(!format!("{media:?}").contains("paired-video-ref"));
    }

    #[test]
    fn incomplete_paired_group_is_rejected_without_fallback() {
        let adapter = BilibiliAdapter;
        let mut observation = paired_observation();
        observation.candidates.pop();
        observation.resource_count = 1;
        let mut server = paired_server();
        server.media.pop();
        assert_eq!(
            adapter.resolve_observation(&locator(1), &observation, &server),
            Err(AdapterError::UnsupportedMedia)
        );
    }

    #[test]
    fn generic_auth_observation_and_server_handoff_are_consumed_by_plugin() {
        let adapter = BilibiliAdapter;
        let locator = locator(1);
        let auth_observation = BrowserAuthObservation {
            schema_version: BROWSER_AUTH_OBSERVATION_VERSION,
            sequence: 3,
            state: BrowserAuthState::CandidateReady,
            diagnostic: site_adapter_api::BrowserAuthDiagnostic::CandidateAccepted,
        };
        assert_eq!(
            adapter.interpret_auth_observation(&auth_observation),
            Ok(BilibiliAuthInterpretation {
                account_state: BilibiliAccountState::Valid,
                session_ready: true,
            })
        );
        let handoff = AuthenticatedSessionHandoff::new_server_owned(
            SITE_ID,
            "fixture-account",
            "opaque-session-ref",
            auth_observation,
        )
        .unwrap();
        adapter
            .resolve_authenticated(&locator, &handoff, &observation(1, "primary"), &server())
            .unwrap();

        let mut pending = auth_observation;
        pending.state = BrowserAuthState::InputNeeded;
        let pending_handoff = AuthenticatedSessionHandoff::new_server_owned(
            SITE_ID,
            "fixture-account",
            "opaque-session-ref",
            pending,
        )
        .unwrap();
        assert_eq!(
            adapter.resolve_authenticated(
                &locator,
                &pending_handoff,
                &observation(1, "primary"),
                &server(),
            ),
            Err(AdapterError::AccessRequired)
        );
    }

    #[test]
    fn malformed_stale_and_secret_handoffs_fail_closed() {
        let adapter = BilibiliAdapter;
        let locator = locator(1);
        assert_eq!(
            adapter.resolve(&locator),
            Err(AdapterError::ObservationRequired)
        );
        let mut stale = observation(1, "primary");
        stale.page_url = "https://www.bilibili.com/video/BV1xx411c7mD/?p=2".into();
        assert_eq!(
            adapter.resolve_observation(&locator, &stale, &server()),
            Err(AdapterError::ContentNotFound)
        );
        let mut secret = server();
        secret.media[0]
            .public_headers
            .insert("Authorization".into(), "Bearer fixture-secret".into());
        assert_eq!(
            adapter.resolve_observation(&locator, &observation(1, "primary"), &secret),
            Err(AdapterError::InvalidObservation)
        );
        let mut wrong_version = locator.clone();
        wrong_version.locator_version = 2;
        assert_eq!(
            adapter.resolve_observation(&wrong_version, &observation(1, "primary"), &server()),
            Err(AdapterError::UnsupportedLocator)
        );
    }

    #[test]
    fn acquisition_target_uses_plugin_owned_locator_part_semantics() {
        let adapter = BilibiliAdapter;
        let first = adapter
            .browser_acquisition_target(&locator(1))
            .unwrap()
            .unwrap();
        let third = adapter
            .browser_acquisition_target(&locator(3))
            .unwrap()
            .unwrap();
        assert_eq!(first.url().as_str(), "https://www.bilibili.com/video/BV1xx411c7mD/");
        assert_eq!(
            third.url().as_str(),
            "https://www.bilibili.com/video/BV1xx411c7mD/?p=3"
        );
        assert!(!format!("{third:?}").contains("BV1xx411c7mD"));
    }

    #[test]
    fn error_diagnostics_do_not_echo_secret_sentinels() {
        assert_error_diagnostics_bounded(&["fixture-secret", "media-ref-1", "cdn.example.invalid"])
            .unwrap();
    }
}
