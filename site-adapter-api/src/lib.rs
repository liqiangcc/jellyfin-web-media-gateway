use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::sync::Arc;
use url::Url;

pub mod conformance;
pub mod security;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocator {
    pub site_id: String,
    pub plugin_id: String,
    pub locator_version: u32,
    pub opaque_payload: String,
}

/// Version of the generic, redacted handoff emitted by a Site Browser Worker.
/// Concrete sites interpret these facts in their own plugin; the worker never
/// needs to know what a candidate means for a particular site.
pub const BROWSER_OBSERVATION_VERSION: u32 = 1;

/// Version of the generic authenticated-browser lifecycle observation.  The
/// Browser Worker only emits these bounded facts; a Site Plugin decides what
/// they mean for its account/session semantics.
pub const BROWSER_AUTH_OBSERVATION_VERSION: u32 = 1;

const MAX_OBSERVATION_ID_BYTES: usize = 128;
const MAX_PAGE_URL_BYTES: usize = 2048;
const MAX_OBSERVATION_CANDIDATES: usize = 16;
const MAX_CANDIDATE_ID_BYTES: usize = 128;
const MAX_ACCESS_REF_BYTES: usize = 256;
pub const MAX_MEDIA_TRACKS: usize = 16;
const MAX_MEDIA_GROUP_BYTES: usize = 128;
const MAX_MEDIA_CODEC_BYTES: usize = 128;
const MAX_MEDIA_CONTAINER_BYTES: usize = 64;
const MAX_MEDIA_MIME_BYTES: usize = 128;
const MAX_MEDIA_LANGUAGE_BYTES: usize = 64;
pub const MAX_MEDIA_EXPIRY_UNIX_SECONDS: u64 = 4_102_444_800;

/// Version of the generic media shape carried alongside legacy stream fields.
/// The shape is site-neutral and describes only media tracks and their
/// server-owned pairing metadata; source locators remain plugin-owned.
pub const MEDIA_SHAPE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserMediaKind {
    Muxed,
    Video,
    Audio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserStatusClass {
    Success,
    Redirect,
    ClientError,
    ServerError,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserRangeSupport {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserExpiryHint {
    NoneObserved,
    ShortLived,
    Expired,
    Unknown,
}

/// Generic facts safe to cross the Browser Worker boundary.  It intentionally
/// contains no response URL, Cookie, Authorization value, DOM selector, or
/// site-specific identity.  `page_url` is the generic page navigation fact;
/// a Site Plugin may interpret it after binding it to its own locator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserMediaCandidate {
    pub id: String,
    pub kind: BrowserMediaKind,
    /// Opaque pairing identity. The Browser Worker never interprets it.
    pub group_id: Option<String>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bitrate: Option<u64>,
    pub language: Option<String>,
    pub protocol: StreamProtocol,
    pub status: BrowserStatusClass,
    pub range: BrowserRangeSupport,
    pub egress_allowed: bool,
    /// Opaque reference to server-owned media state, never a URL or secret.
    pub access_ref: String,
    pub expiry: BrowserExpiryHint,
    /// Optional server-observed absolute expiry; never inferred from a URL.
    pub expires_at: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserObservation {
    pub schema_version: u32,
    pub observation_id: String,
    pub page_url: String,
    pub page_title: String,
    pub event_count: u16,
    pub resource_count: u16,
    pub candidates: Vec<BrowserMediaCandidate>,
}

/// Server-owned handoff for a redacted observation.  This is deliberately not
/// serializable and its Debug output omits the URL, headers, and access ref.
/// The short-lived URL exists only in this bounded server-side object until
/// the normal Gateway capability boundary consumes the resulting media.
#[derive(Clone, Eq, PartialEq)]
pub struct ServerOwnedMedia {
    pub observation_id: String,
    pub candidate_id: String,
    pub access_ref: String,
    pub protocol: StreamProtocol,
    pub url: Url,
    pub public_headers: BTreeMap<String, String>,
}

impl fmt::Debug for ServerOwnedMedia {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerOwnedMedia")
            .field("observation_id", &self.observation_id)
            .field("candidate_id", &self.candidate_id)
            .field("access_ref", &"<redacted>")
            .field("protocol", &self.protocol)
            .field("url", &"<redacted>")
            .field("public_headers", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerOwnedObservation {
    pub schema_version: u32,
    pub observation_id: String,
    pub media: Vec<ServerOwnedMedia>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ResolveContext<'a> {
    pub browser_observation: Option<&'a BrowserObservation>,
    pub server_observation: Option<&'a ServerOwnedObservation>,
    /// An optional server-owned authenticated session handoff.  Core only
    /// transports this opaque proof; the owning Site Plugin decides whether
    /// the source requires it and how to interpret its generic auth facts.
    pub authenticated_session: Option<&'a AuthenticatedSessionHandoff>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserAuthState {
    Required,
    InputNeeded,
    CandidateReady,
    Cancelled,
    Expired,
    Crashed,
    Disconnected,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserAuthDiagnostic {
    None,
    AwaitingInput,
    CandidateAccepted,
    CandidateRejected,
    Cancelled,
    Expired,
    Crashed,
    Disconnected,
    TimedOut,
    CleanupComplete,
}

/// Bounded, redacted authentication lifecycle facts.  There is deliberately
/// no arbitrary message, URL, input, profile path, session material or site
/// selector in this value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserAuthObservation {
    pub schema_version: u32,
    pub sequence: u64,
    pub state: BrowserAuthState,
    pub diagnostic: BrowserAuthDiagnostic,
}

pub fn validate_browser_auth_observation(
    observation: &BrowserAuthObservation,
) -> Result<(), AdapterError> {
    if observation.schema_version != BROWSER_AUTH_OBSERVATION_VERSION || observation.sequence == 0 {
        return Err(AdapterError::InvalidObservation);
    }
    Ok(())
}

/// Server-owned candidate session handoff.  The session reference is opaque;
/// the plugin never receives cookies, profile bytes, or a Vault path.
#[derive(Clone, Eq, PartialEq)]
pub struct AuthenticatedSessionHandoff {
    schema_version: u32,
    site_id: String,
    account_ref: String,
    session_ref: String,
    observation: BrowserAuthObservation,
}

impl fmt::Debug for AuthenticatedSessionHandoff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticatedSessionHandoff")
            .field("schema_version", &self.schema_version)
            .field("site_id", &self.site_id)
            .field("account_ref", &"[opaque]")
            .field("session_ref", &"[opaque]")
            .field("observation", &self.observation)
            .finish()
    }
}

impl AuthenticatedSessionHandoff {
    /// This constructor is for trusted server-side handoff code.  The value
    /// carries only opaque identifiers and generic auth facts, never Secret.
    pub fn new_server_owned(
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
        session_ref: impl Into<String>,
        observation: BrowserAuthObservation,
    ) -> Result<Self, AdapterError> {
        validate_browser_auth_observation(&observation)?;
        let handoff = Self {
            schema_version: BROWSER_AUTH_OBSERVATION_VERSION,
            site_id: site_id.into(),
            account_ref: account_ref.into(),
            session_ref: session_ref.into(),
            observation,
        };
        if !bounded_auth_ref(&handoff.site_id, 128)
            || !bounded_auth_ref(&handoff.account_ref, 256)
            || !bounded_auth_ref(&handoff.session_ref, 256)
            || contains_auth_secret_marker(&handoff.account_ref)
            || contains_auth_secret_marker(&handoff.session_ref)
        {
            return Err(AdapterError::InvalidObservation);
        }
        Ok(handoff)
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn account_ref(&self) -> &str {
        &self.account_ref
    }

    pub fn session_ref(&self) -> &str {
        &self.session_ref
    }

    pub fn observation(&self) -> BrowserAuthObservation {
        self.observation
    }
}

fn bounded_auth_ref(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
}

fn contains_auth_secret_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["cookie", "authorization", "bearer", "password", "token"]
        .iter()
        .any(|marker| lower.contains(marker))
}

pub fn validate_browser_observation(observation: &BrowserObservation) -> Result<(), AdapterError> {
    if observation.schema_version != BROWSER_OBSERVATION_VERSION
        || !bounded_text(&observation.observation_id, MAX_OBSERVATION_ID_BYTES)
        || observation.page_url.is_empty()
        || observation.page_url.len() > MAX_PAGE_URL_BYTES
        || observation.page_url.chars().any(char::is_control)
        || !bounded_text(&observation.page_title, 512)
        || contains_secret_marker(&observation.page_title)
        || observation.event_count > 200
        || observation.resource_count > 64
        || observation.candidates.len() > MAX_OBSERVATION_CANDIDATES
        || observation.resource_count as usize != observation.candidates.len()
    {
        return Err(AdapterError::InvalidObservation);
    }
    if observation.page_url.starts_with("blob:")
        || observation.page_url.starts_with("file:")
        || contains_secret_marker(&observation.page_url)
        || Url::parse(&observation.page_url)
            .ok()
            .is_none_or(|url| !matches!(url.scheme(), "http" | "https"))
    {
        return Err(AdapterError::InvalidObservation);
    }
    for candidate in &observation.candidates {
        if !bounded_text(&candidate.id, MAX_CANDIDATE_ID_BYTES)
            || contains_secret_marker(&candidate.id)
            || !bounded_opaque_ref(&candidate.access_ref)
            || !bounded_optional_metadata(candidate.group_id.as_deref(), MAX_MEDIA_GROUP_BYTES)
            || !bounded_optional_metadata(candidate.codec.as_deref(), MAX_MEDIA_CODEC_BYTES)
            || !bounded_optional_metadata(candidate.container.as_deref(), MAX_MEDIA_CONTAINER_BYTES)
            || !bounded_optional_metadata(candidate.mime_type.as_deref(), MAX_MEDIA_MIME_BYTES)
            || !bounded_optional_metadata(candidate.language.as_deref(), MAX_MEDIA_LANGUAGE_BYTES)
            || candidate
                .width
                .is_some_and(|value| value == 0 || value > 16_384)
            || candidate
                .height
                .is_some_and(|value| value == 0 || value > 16_384)
            || candidate
                .bitrate
                .is_some_and(|value| value == 0 || value > 1_000_000_000)
            || candidate
                .expires_at
                .is_some_and(|value| value == 0 || value > MAX_MEDIA_EXPIRY_UNIX_SECONDS)
        {
            return Err(AdapterError::InvalidObservation);
        }
    }
    Ok(())
}

pub fn validate_server_owned_observation(
    observation: &ServerOwnedObservation,
) -> Result<(), AdapterError> {
    if observation.schema_version != BROWSER_OBSERVATION_VERSION
        || !bounded_text(&observation.observation_id, MAX_OBSERVATION_ID_BYTES)
        || observation.media.len() > MAX_OBSERVATION_CANDIDATES
    {
        return Err(AdapterError::InvalidObservation);
    }
    for media in &observation.media {
        if media.observation_id != observation.observation_id
            || !bounded_text(&media.candidate_id, MAX_CANDIDATE_ID_BYTES)
            || !bounded_opaque_ref(&media.access_ref)
            || !matches!(media.url.scheme(), "http" | "https")
            || media.url.host_str().is_none()
            || !media.url.username().is_empty()
            || media.url.password().is_some()
            || media.public_headers.len() > 16
        {
            return Err(AdapterError::InvalidObservation);
        }
        for (name, value) in &media.public_headers {
            if name.is_empty()
                || name.len() > 128
                || !name.chars().all(is_http_token_character)
                || value.len() > 4096
                || value.chars().any(char::is_control)
                || security::is_secret_header(name, value)
            {
                return Err(AdapterError::InvalidObservation);
            }
        }
    }
    Ok(())
}

fn bounded_text(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value.chars().all(|character| !character.is_control())
}

fn bounded_opaque_ref(value: &str) -> bool {
    bounded_text(value, MAX_ACCESS_REF_BYTES)
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
        && !contains_secret_marker(value)
}

fn bounded_optional_metadata(value: Option<&str>, max: usize) -> bool {
    value.is_none_or(|value| {
        !value.is_empty()
            && value.len() <= max
            && value.chars().all(|character| !character.is_control())
            && !contains_secret_marker(value)
    })
}

fn is_http_token_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(character)
}

fn contains_secret_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "cookie",
        "authorization",
        "bearer",
        "sessdata",
        "token",
        "signed-url",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

const MAX_LOCATOR_FIELD_BYTES: usize = 128;
const MAX_OPAQUE_PAYLOAD_BYTES: usize = 16 * 1024;
const MAX_COLLECTION_ID_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationContext {
    pub previous: Option<SourceLocator>,
    pub next: Option<SourceLocator>,
    pub collection_id: Option<String>,
    pub current_index: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationDirection {
    Previous,
    Next,
}

impl NavigationDirection {
    pub fn select(self, context: &NavigationContext) -> Option<&SourceLocator> {
        match self {
            Self::Previous => context.previous.as_ref(),
            Self::Next => context.next.as_ref(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamProtocol {
    HttpFile,
    Hls,
    Dash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaTrackKind {
    Muxed,
    Video,
    Audio,
}

impl MediaTrackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Muxed => "muxed",
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }
}

impl From<BrowserMediaKind> for MediaTrackKind {
    fn from(value: BrowserMediaKind) -> Self {
        match value {
            BrowserMediaKind::Muxed => Self::Muxed,
            BrowserMediaKind::Video => Self::Video,
            BrowserMediaKind::Audio => Self::Audio,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct MediaTrack {
    pub id: String,
    pub kind: MediaTrackKind,
    pub group_id: Option<String>,
    pub protocol: StreamProtocol,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bitrate: Option<u64>,
    pub language: Option<String>,
    /// Server-owned opaque access reference; never a URL or credential.
    pub access_ref: Option<String>,
    /// Unix seconds, bounded by the contract. `None` means no observed expiry.
    pub expires_at: Option<u64>,
}

impl fmt::Debug for MediaTrack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MediaTrack")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("group_id", &self.group_id)
            .field("protocol", &self.protocol)
            .field("codec", &self.codec)
            .field("container", &self.container)
            .field("mime_type", &self.mime_type)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bitrate", &self.bitrate)
            .field("language", &self.language)
            .field(
                "access_ref",
                &self.access_ref.as_ref().map(|_| "<redacted>"),
            )
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaShapeV1 {
    pub version: u32,
    pub tracks: Vec<MediaTrack>,
}

impl MediaShapeV1 {
    pub fn legacy_from_streams(streams: &[ResolvedStream]) -> Self {
        Self {
            version: MEDIA_SHAPE_VERSION,
            tracks: streams
                .iter()
                .map(|stream| MediaTrack {
                    id: stream.id.clone(),
                    kind: MediaTrackKind::Muxed,
                    group_id: None,
                    protocol: stream.protocol,
                    codec: None,
                    container: None,
                    mime_type: None,
                    width: None,
                    height: None,
                    bitrate: None,
                    language: None,
                    access_ref: stream.upstream_access_ref.clone(),
                    expires_at: None,
                })
                .collect(),
        }
    }

    pub fn track(&self, id: &str) -> Option<&MediaTrack> {
        self.tracks.iter().find(|track| track.id == id)
    }

    pub fn is_expired(&self, now_unix_seconds: u64) -> bool {
        self.tracks.iter().any(|track| {
            track
                .expires_at
                .is_some_and(|expires_at| expires_at <= now_unix_seconds)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedStream {
    pub id: String,
    pub protocol: StreamProtocol,
    pub url: Url,
    pub public_headers: BTreeMap<String, String>,
    pub upstream_access_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSubtitle {
    pub id: String,
    pub url: Url,
    pub content_type: String,
    pub language: Option<String>,
    pub label: Option<String>,
    pub public_headers: BTreeMap<String, String>,
    pub upstream_access_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedMedia {
    pub title: String,
    pub source_site: String,
    pub streams: Vec<ResolvedStream>,
    pub subtitles: Vec<ResolvedSubtitle>,
    pub protection: MediaProtection,
    /// Explicit compatibility path: legacy stream-only adapters use
    /// `ResolvedMedia::legacy`, while new adapters provide MediaShapeV1.
    pub shape: MediaShapeV1,
}

impl ResolvedMedia {
    pub fn legacy(
        title: impl Into<String>,
        source_site: impl Into<String>,
        streams: Vec<ResolvedStream>,
        subtitles: Vec<ResolvedSubtitle>,
        protection: MediaProtection,
    ) -> Self {
        let shape = MediaShapeV1::legacy_from_streams(&streams);
        Self {
            title: title.into(),
            source_site: source_site.into(),
            streams,
            subtitles,
            protection,
            shape,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaProtection {
    Clear,
    DrmUnsupported,
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct RecognizeResult {
    pub matched: bool,
    pub site_id: String,
    pub plugin_id: String,
    pub priority: u16,
    pub locator: Option<SourceLocator>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidInput,
    InvalidAdapterOutput,
    InvalidLocatorOwnership,
    InvalidResolvedMedia,
    UnsupportedLocator,
    NoMatch,
    AmbiguousMatch,
    DuplicatePlugin,
    PluginNotFound,
    UnsupportedNavigation,
    InvalidNavigation,
    InvalidObservation,
    AccessRequired,
    ObservationRequired,
    ObservationExpired,
    ContentNotFound,
    UpstreamDenied,
    UnsupportedMedia,
    EgressRejected,
    SecretMaterial,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AdapterError {}

pub trait SiteAdapter: Send + Sync {
    fn site_id(&self) -> &'static str;
    fn plugin_id(&self) -> &'static str;
    fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError>;
    fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError>;

    /// Resolve using a generic redacted observation plus a server-owned
    /// handoff. Existing adapters remain compatible and ignore the optional
    /// context; adapters that need browser acquisition must opt in.
    fn resolve_with_context(
        &self,
        locator: &SourceLocator,
        _context: ResolveContext<'_>,
    ) -> Result<ResolvedMedia, AdapterError> {
        self.resolve(locator)
    }

    /// Return opaque neighbouring locators.  Adapters that do not expose a
    /// collection/navigation model fail closed without changing their
    /// existing recognize/resolve implementation.
    fn navigation(&self, _locator: &SourceLocator) -> Result<NavigationContext, AdapterError> {
        Err(AdapterError::UnsupportedNavigation)
    }
}

#[derive(Default)]
pub struct SiteAdapterRegistry {
    adapters: Vec<Arc<dyn SiteAdapter>>,
    ids: HashSet<String>,
}

impl SiteAdapterRegistry {
    pub fn register(&mut self, adapter: Arc<dyn SiteAdapter>) -> Result<(), AdapterError> {
        if adapter.site_id().is_empty() || adapter.plugin_id().is_empty() {
            return Err(AdapterError::InvalidAdapterOutput);
        }
        if !self.ids.insert(adapter.plugin_id().to_string()) {
            return Err(AdapterError::DuplicatePlugin);
        }
        self.adapters.push(adapter);
        Ok(())
    }

    pub fn recognize(&self, input: &str) -> Result<SourceLocator, AdapterError> {
        let mut candidates = Vec::new();
        for adapter in &self.adapters {
            let result = adapter.recognize(input)?;
            validate_recognition(adapter.as_ref(), &result)?;
            if result.matched {
                candidates.push(result);
            }
        }
        let max = candidates
            .iter()
            .map(|c| c.priority)
            .max()
            .ok_or(AdapterError::NoMatch)?;
        let winners: Vec<_> = candidates
            .into_iter()
            .filter(|c| c.priority == max)
            .collect();
        if winners.len() != 1 {
            return Err(AdapterError::AmbiguousMatch);
        }
        winners[0].locator.clone().ok_or(AdapterError::NoMatch)
    }

    pub fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
        let adapter = self
            .adapters
            .iter()
            .find(|a| a.plugin_id() == locator.plugin_id)
            .ok_or(AdapterError::PluginNotFound)?;
        if locator.site_id != adapter.site_id() {
            return Err(AdapterError::InvalidLocatorOwnership);
        }
        let media = adapter.resolve(locator)?;
        conformance::validate_resolved_media(&media)
            .map_err(|_| AdapterError::InvalidResolvedMedia)?;
        Ok(media)
    }

    pub fn resolve_with_context(
        &self,
        locator: &SourceLocator,
        context: ResolveContext<'_>,
    ) -> Result<ResolvedMedia, AdapterError> {
        let adapter = self
            .adapters
            .iter()
            .find(|a| a.plugin_id() == locator.plugin_id)
            .ok_or(AdapterError::PluginNotFound)?;
        if locator.site_id != adapter.site_id() {
            return Err(AdapterError::InvalidLocatorOwnership);
        }
        if let Some(observation) = context.browser_observation {
            validate_browser_observation(observation)?;
        }
        if let Some(observation) = context.server_observation {
            validate_server_owned_observation(observation)?;
        }
        let media = adapter.resolve_with_context(locator, context)?;
        conformance::validate_resolved_media(&media)
            .map_err(|_| AdapterError::InvalidResolvedMedia)?;
        Ok(media)
    }

    /// Route navigation by the owning plugin identity, never by registration
    /// order or caller-selected destination plugin.
    pub fn navigation(&self, locator: &SourceLocator) -> Result<NavigationContext, AdapterError> {
        let adapter = self
            .adapters
            .iter()
            .find(|adapter| adapter.plugin_id() == locator.plugin_id)
            .ok_or(AdapterError::PluginNotFound)?;
        validate_locator_ownership(adapter.as_ref(), locator)?;
        let context = adapter.navigation(locator)?;
        validate_navigation(adapter.as_ref(), &context)?;
        Ok(context)
    }
}

fn validate_locator_ownership(
    adapter: &dyn SiteAdapter,
    locator: &SourceLocator,
) -> Result<(), AdapterError> {
    if locator.site_id != adapter.site_id() || locator.plugin_id != adapter.plugin_id() {
        return Err(AdapterError::InvalidLocatorOwnership);
    }
    if locator.locator_version == 0
        || locator.site_id.is_empty()
        || locator.plugin_id.is_empty()
        || locator.site_id.len() > MAX_LOCATOR_FIELD_BYTES
        || locator.plugin_id.len() > MAX_LOCATOR_FIELD_BYTES
        || locator.opaque_payload.is_empty()
        || locator.opaque_payload.len() > MAX_OPAQUE_PAYLOAD_BYTES
        || locator.opaque_payload.chars().any(char::is_control)
    {
        return Err(AdapterError::InvalidNavigation);
    }
    Ok(())
}

fn validate_navigation(
    adapter: &dyn SiteAdapter,
    context: &NavigationContext,
) -> Result<(), AdapterError> {
    if context.collection_id.as_deref().is_some_and(|value| {
        value.is_empty()
            || value.len() > MAX_COLLECTION_ID_BYTES
            || value.chars().any(char::is_control)
    }) {
        return Err(AdapterError::InvalidNavigation);
    }
    for locator in [context.previous.as_ref(), context.next.as_ref()]
        .into_iter()
        .flatten()
    {
        validate_locator_ownership(adapter, locator)?;
    }
    Ok(())
}

fn validate_recognition(
    adapter: &dyn SiteAdapter,
    result: &RecognizeResult,
) -> Result<(), AdapterError> {
    if result.site_id != adapter.site_id() || result.plugin_id != adapter.plugin_id() {
        return Err(AdapterError::InvalidAdapterOutput);
    }
    match (result.matched, result.locator.as_ref()) {
        (false, None) => Ok(()),
        (true, Some(locator))
            if locator.site_id == adapter.site_id() && locator.plugin_id == adapter.plugin_id() =>
        {
            Ok(())
        }
        _ => Err(AdapterError::InvalidAdapterOutput),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(&'static str, u16);
    impl SiteAdapter for Fake {
        fn site_id(&self) -> &'static str {
            "fake"
        }

        fn plugin_id(&self) -> &'static str {
            self.0
        }
        fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
            Ok(RecognizeResult {
                matched: input.starts_with("https://"),
                site_id: "fake".into(),
                plugin_id: self.0.into(),
                priority: self.1,
                locator: Some(SourceLocator {
                    site_id: "fake".into(),
                    plugin_id: self.0.into(),
                    locator_version: 1,
                    opaque_payload: input.into(),
                }),
            })
        }
        fn resolve(&self, _locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
            Err(AdapterError::UnsupportedLocator)
        }
    }

    struct NavigationFake {
        plugin: &'static str,
        site: &'static str,
        result: Result<NavigationContext, AdapterError>,
    }

    impl SiteAdapter for NavigationFake {
        fn site_id(&self) -> &'static str {
            self.site
        }

        fn plugin_id(&self) -> &'static str {
            self.plugin
        }

        fn recognize(&self, _input: &str) -> Result<RecognizeResult, AdapterError> {
            Ok(RecognizeResult {
                matched: false,
                site_id: self.site.into(),
                plugin_id: self.plugin.into(),
                priority: 1,
                locator: None,
            })
        }

        fn resolve(&self, _locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
            Err(AdapterError::UnsupportedLocator)
        }

        fn navigation(&self, _locator: &SourceLocator) -> Result<NavigationContext, AdapterError> {
            self.result.clone()
        }
    }

    #[test]
    fn registry_uses_explicit_priority_not_registration_order() {
        let mut registry = SiteAdapterRegistry::default();
        registry.register(Arc::new(Fake("low", 1))).unwrap();
        registry.register(Arc::new(Fake("high", 10))).unwrap();
        let locator = registry.recognize("https://example.com/video.mp4").unwrap();
        assert_eq!(locator.plugin_id, "high");
    }

    #[test]
    fn authenticated_handoff_is_opaque_versioned_and_bounded() {
        let observation = BrowserAuthObservation {
            schema_version: BROWSER_AUTH_OBSERVATION_VERSION,
            sequence: 1,
            state: BrowserAuthState::CandidateReady,
            diagnostic: BrowserAuthDiagnostic::CandidateAccepted,
        };
        let handoff = AuthenticatedSessionHandoff::new_server_owned(
            "site",
            "fixture-account-opaque",
            "fixture-session-opaque",
            observation,
        )
        .unwrap();
        assert_eq!(handoff.schema_version(), BROWSER_AUTH_OBSERVATION_VERSION);
        assert_eq!(handoff.observation(), observation);
        let debug = format!("{handoff:?}");
        assert!(!debug.contains("fixture-account-opaque"));
        assert!(!debug.contains("fixture-session-opaque"));
        assert_eq!(
            validate_browser_auth_observation(&BrowserAuthObservation {
                sequence: 0,
                ..observation
            }),
            Err(AdapterError::InvalidObservation)
        );
        assert_eq!(
            AuthenticatedSessionHandoff::new_server_owned(
                "site",
                "account-token",
                "session-ref",
                observation,
            ),
            Err(AdapterError::InvalidObservation)
        );
    }

    fn locator(plugin: &str, site: &str, payload: &str) -> SourceLocator {
        SourceLocator {
            site_id: site.into(),
            plugin_id: plugin.into(),
            locator_version: 1,
            opaque_payload: payload.into(),
        }
    }

    #[test]
    fn navigation_defaults_to_unsupported_for_existing_adapters() {
        let mut registry = SiteAdapterRegistry::default();
        registry.register(Arc::new(Fake("legacy", 1))).unwrap();

        assert_eq!(
            registry.navigation(&locator("legacy", "fake", "current")),
            Err(AdapterError::UnsupportedNavigation)
        );
    }

    #[test]
    fn navigation_validates_owner_and_edge_values() {
        let current = locator("navigator", "site", "current");
        let next = locator("navigator", "site", "next");
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(NavigationFake {
                plugin: "navigator",
                site: "site",
                result: Ok(NavigationContext {
                    previous: None,
                    next: Some(next.clone()),
                    collection_id: Some("collection".into()),
                    current_index: Some(0),
                }),
            }))
            .unwrap();

        let context = registry.navigation(&current).unwrap();
        assert_eq!(context.previous, None);
        assert_eq!(context.next, Some(next));
        assert_eq!(context.current_index, Some(0));
        assert_eq!(
            registry.navigation(&locator("foreign", "site", "current")),
            Err(AdapterError::PluginNotFound)
        );
    }

    #[test]
    fn navigation_rejects_foreign_and_malformed_returned_locators() {
        let current = locator("navigator", "site", "current");
        for (returned, expected) in [
            (
                locator("foreign", "site", "next"),
                AdapterError::InvalidLocatorOwnership,
            ),
            (
                SourceLocator {
                    locator_version: 0,
                    ..locator("navigator", "site", "next")
                },
                AdapterError::InvalidNavigation,
            ),
        ] {
            let mut registry = SiteAdapterRegistry::default();
            registry
                .register(Arc::new(NavigationFake {
                    plugin: "navigator",
                    site: "site",
                    result: Ok(NavigationContext {
                        previous: None,
                        next: Some(returned),
                        collection_id: None,
                        current_index: None,
                    }),
                }))
                .unwrap();
            assert_eq!(registry.navigation(&current), Err(expected));
        }
    }
}
