//! Reusable, deterministic checks for compile-time `SiteAdapter` plugins.
//!
//! The harness deliberately treats `SourceLocator::opaque_payload` as opaque:
//! it only checks ownership and version metadata, then hands the locator back
//! to the adapter.  Site-specific parsing belongs in the plugin under test.

use crate::{AdapterError, MediaProtection, RecognizeResult, ResolvedMedia, SiteAdapter};
use std::fmt;

pub use crate::security::is_secret_header;

#[derive(Clone, Copy, Debug)]
pub struct RecognizeFixture {
    pub input: &'static str,
    pub expected_match: bool,
    pub expected_site_id: &'static str,
    pub expected_locator_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConformanceFailure {
    RecognitionChanged { input: String },
    RecognitionOwnership { input: String },
    RecognitionMatch { input: String },
    MissingLocator { input: String },
    UnexpectedLocator { input: String },
    LocatorMetadata { input: String },
    ResolveAcceptedForeignPlugin,
    ResolveAcceptedForeignSite,
    ResolveAcceptedUnsupportedVersion,
    InvalidResolvedMedia(String),
    SecretInDiagnostic(String),
}

impl fmt::Display for ConformanceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ConformanceFailure {}

/// Run deterministic recognition, locator-boundary and output checks for one
/// plugin.  This function is suitable for plugin crates to call from their
/// own tests; it does not make network requests or decode opaque payloads.
pub fn assert_adapter_conforms(
    adapter: &dyn SiteAdapter,
    fixtures: &[RecognizeFixture],
) -> Result<(), ConformanceFailure> {
    if fixtures.is_empty() {
        return Err(ConformanceFailure::RecognitionMatch {
            input: "<empty fixture set>".into(),
        });
    }

    for fixture in fixtures {
        let first =
            adapter
                .recognize(fixture.input)
                .map_err(|_| ConformanceFailure::RecognitionMatch {
                    input: fixture.input.into(),
                })?;
        let second =
            adapter
                .recognize(fixture.input)
                .map_err(|_| ConformanceFailure::RecognitionMatch {
                    input: fixture.input.into(),
                })?;

        if !same_recognition(&first, &second) {
            return Err(ConformanceFailure::RecognitionChanged {
                input: fixture.input.into(),
            });
        }
        if first.site_id != fixture.expected_site_id
            || first.plugin_id != adapter.plugin_id()
            || first.site_id != adapter.site_id()
        {
            return Err(ConformanceFailure::RecognitionOwnership {
                input: fixture.input.into(),
            });
        }
        if first.matched != fixture.expected_match {
            return Err(ConformanceFailure::RecognitionMatch {
                input: fixture.input.into(),
            });
        }

        if !first.matched {
            if first.locator.is_some() {
                return Err(ConformanceFailure::UnexpectedLocator {
                    input: fixture.input.into(),
                });
            }
            continue;
        }

        let locator = first
            .locator
            .as_ref()
            .ok_or_else(|| ConformanceFailure::MissingLocator {
                input: fixture.input.into(),
            })?;
        if locator.site_id != fixture.expected_site_id
            || locator.plugin_id != adapter.plugin_id()
            || locator.locator_version != fixture.expected_locator_version
        {
            return Err(ConformanceFailure::LocatorMetadata {
                input: fixture.input.into(),
            });
        }

        let media = adapter
            .resolve(locator)
            .map_err(|_| ConformanceFailure::InvalidResolvedMedia(fixture.input.into()))?;
        validate_resolved_media(&media).map_err(|error| {
            ConformanceFailure::InvalidResolvedMedia(format!("{}: {error}", fixture.input))
        })?;

        let mut foreign_plugin = locator.clone();
        foreign_plugin.plugin_id = "foreign-plugin".into();
        if adapter.resolve(&foreign_plugin).is_ok() {
            return Err(ConformanceFailure::ResolveAcceptedForeignPlugin);
        }

        let mut foreign_site = locator.clone();
        foreign_site.site_id = "foreign-site".into();
        if adapter.resolve(&foreign_site).is_ok() {
            return Err(ConformanceFailure::ResolveAcceptedForeignSite);
        }

        let mut unsupported_version = locator.clone();
        unsupported_version.locator_version = locator.locator_version.wrapping_add(1);
        if adapter.resolve(&unsupported_version).is_ok() {
            return Err(ConformanceFailure::ResolveAcceptedUnsupportedVersion);
        }
    }

    Ok(())
}

fn same_recognition(left: &RecognizeResult, right: &RecognizeResult) -> bool {
    left.matched == right.matched
        && left.site_id == right.site_id
        && left.plugin_id == right.plugin_id
        && left.priority == right.priority
        && left.locator == right.locator
}

/// Validate the display-facing portion of a `ResolvedMedia` result.
pub fn validate_resolved_media(media: &ResolvedMedia) -> Result<(), String> {
    if media.title.trim().is_empty() || media.source_site.trim().is_empty() {
        return Err("title and source_site must be non-empty".into());
    }
    if media.protection == MediaProtection::Clear && media.streams.is_empty() {
        return Err("clear media must contain at least one stream".into());
    }

    for stream in &media.streams {
        if stream.id.trim().is_empty() {
            return Err("stream id must be non-empty".into());
        }
        if !matches!(stream.url.scheme(), "http" | "https") {
            return Err("stream URL must use http or https".into());
        }
        for (name, value) in &stream.public_headers {
            if is_secret_header(name, value) {
                return Err(format!("secret-bearing public header: {name}"));
            }
        }
    }
    for subtitle in &media.subtitles {
        if subtitle.id.trim().is_empty() || subtitle.id.len() > 128 {
            return Err("subtitle id must be non-empty and bounded".into());
        }
        if !matches!(subtitle.url.scheme(), "http" | "https") {
            return Err("subtitle URL must use http or https".into());
        }
        if !subtitle.content_type.eq_ignore_ascii_case("text/vtt") {
            return Err("subtitle content type must be text/vtt".into());
        }
        if subtitle
            .language
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 64)
            || subtitle
                .label
                .as_deref()
                .is_some_and(|value| value.is_empty() || value.len() > 128)
        {
            return Err("subtitle language/label is invalid".into());
        }
        for (name, value) in &subtitle.public_headers {
            if is_secret_header(name, value) {
                return Err(format!("secret-bearing subtitle header: {name}"));
            }
        }
    }
    Ok(())
}

/// Check that the public error enum remains bounded and cannot echo fixture
/// secrets.  Callers should pass sentinel values used by their tests.
pub fn assert_error_diagnostics_bounded(sentinels: &[&str]) -> Result<(), ConformanceFailure> {
    let errors = [
        AdapterError::InvalidInput,
        AdapterError::InvalidAdapterOutput,
        AdapterError::InvalidLocatorOwnership,
        AdapterError::InvalidResolvedMedia,
        AdapterError::UnsupportedLocator,
        AdapterError::NoMatch,
        AdapterError::AmbiguousMatch,
        AdapterError::DuplicatePlugin,
        AdapterError::PluginNotFound,
    ];
    for error in errors {
        let rendered = error.to_string();
        for sentinel in sentinels {
            if rendered.contains(sentinel) {
                return Err(ConformanceFailure::SecretInDiagnostic((*sentinel).into()));
            }
        }
        if rendered.len() > 64 {
            return Err(ConformanceFailure::SecretInDiagnostic(rendered));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ResolvedStream, ResolvedSubtitle, SiteAdapterRegistry, SourceLocator, StreamProtocol,
    };
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use url::Url;

    struct Fake {
        plugin: &'static str,
        site: &'static str,
        priority: u16,
        matches: bool,
    }

    impl SiteAdapter for Fake {
        fn site_id(&self) -> &'static str {
            self.site
        }

        fn plugin_id(&self) -> &'static str {
            self.plugin
        }

        fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
            Ok(RecognizeResult {
                matched: self.matches,
                site_id: self.site.into(),
                plugin_id: self.plugin.into(),
                priority: self.priority,
                locator: self.matches.then(|| SourceLocator {
                    site_id: self.site.into(),
                    plugin_id: self.plugin.into(),
                    locator_version: 1,
                    opaque_payload: input.into(),
                }),
            })
        }

        fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
            if locator.site_id != self.site
                || locator.plugin_id != self.plugin
                || locator.locator_version != 1
            {
                return Err(AdapterError::UnsupportedLocator);
            }
            Ok(ResolvedMedia {
                title: "fixture".into(),
                source_site: self.site.into(),
                streams: vec![ResolvedStream {
                    id: "primary".into(),
                    protocol: StreamProtocol::HttpFile,
                    url: Url::parse("https://example.test/video.mp4").unwrap(),
                    public_headers: BTreeMap::new(),
                    upstream_access_ref: None,
                }],
                subtitles: Vec::new(),
                protection: MediaProtection::Clear,
            })
        }
    }

    fn fixture() -> RecognizeFixture {
        RecognizeFixture {
            input: "https://example.test/video.mp4",
            expected_match: true,
            expected_site_id: "fixture-site",
            expected_locator_version: 1,
        }
    }

    #[test]
    fn harness_checks_determinism_ownership_version_and_media_shape() {
        assert_adapter_conforms(
            &Fake {
                plugin: "fixture-plugin",
                site: "fixture-site",
                priority: 10,
                matches: true,
            },
            &[fixture()],
        )
        .unwrap();
    }

    #[test]
    fn public_headers_reject_cookie_authorization_and_bearer_material() {
        for (name, value) in [
            ("Cookie", "session=fixture-secret"),
            ("Authorization", "Bearer fixture-secret"),
            ("Set-Cookie", "session=fixture-secret"),
            ("X-API-Key", "fixture-secret"),
            ("x-auth-token", "fixture-secret"),
            ("proxy-authenticate", "fixture-secret"),
            ("X-Trace", "Basic fixture-secret"),
            ("X-Trace", "Bearer fixture-secret"),
        ] {
            assert!(is_secret_header(name, value));
        }
        assert!(!is_secret_header("Accept", "video/mp4"));
    }

    #[test]
    fn subtitle_contract_rejects_local_unsupported_and_secret_inputs() {
        let base = ResolvedMedia {
            title: "fixture".into(),
            source_site: "fixture-site".into(),
            streams: vec![ResolvedStream {
                id: "primary".into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://example.test/video.mp4").unwrap(),
                public_headers: BTreeMap::new(),
                upstream_access_ref: None,
            }],
            subtitles: Vec::new(),
            protection: MediaProtection::Clear,
        };
        for subtitle in [
            ResolvedSubtitle {
                id: "local".into(),
                url: Url::parse("file:///tmp/captions.vtt").unwrap(),
                content_type: "text/vtt".into(),
                language: None,
                label: None,
                public_headers: BTreeMap::new(),
                upstream_access_ref: None,
            },
            ResolvedSubtitle {
                id: "unsupported".into(),
                url: Url::parse("https://example.test/captions.srt").unwrap(),
                content_type: "text/srt".into(),
                language: None,
                label: None,
                public_headers: BTreeMap::new(),
                upstream_access_ref: None,
            },
            ResolvedSubtitle {
                id: "secret".into(),
                url: Url::parse("https://example.test/captions.vtt").unwrap(),
                content_type: "text/vtt".into(),
                language: None,
                label: None,
                public_headers: BTreeMap::from([(
                    "Authorization".into(),
                    "Bearer fixture-secret".into(),
                )]),
                upstream_access_ref: None,
            },
        ] {
            let mut candidate = base.clone();
            candidate.subtitles.push(subtitle);
            assert!(validate_resolved_media(&candidate).is_err());
        }
    }

    #[test]
    fn registry_conflicts_and_locator_ownership_are_deterministic() {
        let low = Arc::new(Fake {
            plugin: "low",
            site: "fixture-site",
            priority: 1,
            matches: true,
        });
        let high = Arc::new(Fake {
            plugin: "high",
            site: "fixture-site",
            priority: 10,
            matches: true,
        });
        let mut registry = SiteAdapterRegistry::default();
        registry.register(low.clone()).unwrap();
        registry.register(high.clone()).unwrap();
        assert_eq!(
            registry.recognize(fixture().input).unwrap().plugin_id,
            "high"
        );
        assert_eq!(registry.register(high), Err(AdapterError::DuplicatePlugin));

        let mut ambiguous = SiteAdapterRegistry::default();
        ambiguous
            .register(Arc::new(Fake {
                plugin: "a",
                site: "fixture-site",
                priority: 10,
                matches: true,
            }))
            .unwrap();
        ambiguous
            .register(Arc::new(Fake {
                plugin: "b",
                site: "fixture-site",
                priority: 10,
                matches: true,
            }))
            .unwrap();
        assert_eq!(
            ambiguous.recognize(fixture().input),
            Err(AdapterError::AmbiguousMatch)
        );

        let mut no_match = SiteAdapterRegistry::default();
        no_match
            .register(Arc::new(Fake {
                plugin: "no-match",
                site: "fixture-site",
                priority: 10,
                matches: false,
            }))
            .unwrap();
        assert_eq!(
            no_match.recognize(fixture().input),
            Err(AdapterError::NoMatch)
        );

        let unsupported = SourceLocator {
            site_id: "fixture-site".into(),
            plugin_id: "high".into(),
            locator_version: 2,
            opaque_payload: "opaque".into(),
        };
        assert_eq!(
            registry.resolve(&unsupported),
            Err(AdapterError::UnsupportedLocator)
        );
        assert_eq!(
            registry.resolve(&SourceLocator {
                site_id: "fixture-site".into(),
                plugin_id: "missing".into(),
                locator_version: 1,
                opaque_payload: "opaque".into(),
            }),
            Err(AdapterError::PluginNotFound)
        );
        assert_eq!(
            registry.resolve(&SourceLocator {
                site_id: "foreign-site".into(),
                plugin_id: "high".into(),
                locator_version: 1,
                opaque_payload: "opaque".into(),
            }),
            Err(AdapterError::InvalidLocatorOwnership)
        );
    }

    #[test]
    fn diagnostics_do_not_echo_secret_sentinels() {
        assert_error_diagnostics_bounded(&[
            "fixture-cookie-secret",
            "fixture-authorization-secret",
            "Bearer fixture-secret",
        ])
        .unwrap();
    }
}
