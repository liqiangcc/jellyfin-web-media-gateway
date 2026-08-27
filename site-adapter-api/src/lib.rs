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
