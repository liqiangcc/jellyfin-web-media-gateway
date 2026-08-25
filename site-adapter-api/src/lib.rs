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

    #[test]
    fn registry_uses_explicit_priority_not_registration_order() {
        let mut registry = SiteAdapterRegistry::default();
        registry.register(Arc::new(Fake("low", 1))).unwrap();
        registry.register(Arc::new(Fake("high", 10))).unwrap();
        let locator = registry.recognize("https://example.com/video.mp4").unwrap();
        assert_eq!(locator.plugin_id, "high");
    }
}
