use site_adapter_api::{
    AdapterError, MediaProtection, RecognizeResult, ResolvedMedia, ResolvedStream, SiteAdapter,
    SourceLocator, StreamProtocol,
};
use std::collections::BTreeMap;
use url::Url;

#[derive(Default)]
pub struct GenericDirectAdapter;

impl GenericDirectAdapter {
    fn protocol(url: &Url) -> Option<StreamProtocol> {
        let path = url.path().to_ascii_lowercase();
        if path.ends_with(".mp4") || path.ends_with(".m4v") {
            Some(StreamProtocol::HttpFile)
        } else if path.ends_with(".m3u8") {
            Some(StreamProtocol::Hls)
        } else {
            None
        }
    }
}

impl SiteAdapter for GenericDirectAdapter {
    fn plugin_id(&self) -> &'static str {
        "generic-direct"
    }

    fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
        let url = match Url::parse(input) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => url,
            _ => {
                return Ok(RecognizeResult {
                    matched: false,
                    site_id: "generic".into(),
                    plugin_id: self.plugin_id().into(),
                    priority: 10,
                    locator: None,
                });
            }
        };
        let matched = Self::protocol(&url).is_some();
        Ok(RecognizeResult {
            matched,
            site_id: "generic".into(),
            plugin_id: self.plugin_id().into(),
            priority: 10,
            locator: matched.then(|| SourceLocator {
                site_id: "generic".into(),
                plugin_id: self.plugin_id().into(),
                locator_version: 1,
                opaque_payload: input.to_string(),
            }),
        })
    }

    fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
        if locator.plugin_id != self.plugin_id() || locator.locator_version != 1 {
            return Err(AdapterError::UnsupportedLocator);
        }
        let url = Url::parse(&locator.opaque_payload).map_err(|_| AdapterError::InvalidInput)?;
        let protocol = Self::protocol(&url).ok_or(AdapterError::UnsupportedLocator)?;
        Ok(ResolvedMedia {
            title: "generic-direct media".into(),
            source_site: "generic".into(),
            streams: vec![ResolvedStream {
                id: "primary".into(),
                protocol,
                url,
                public_headers: BTreeMap::new(),
                upstream_access_ref: None,
            }],
            protection: MediaProtection::Clear,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_mp4_and_hls_resolve_without_secret_headers() {
        let adapter = GenericDirectAdapter;
        for input in [
            "https://example.com/media/video.mp4",
            "https://example.com/live/master.m3u8?quality=1",
        ] {
            let locator = adapter.recognize(input).unwrap().locator.unwrap();
            let media = adapter.resolve(&locator).unwrap();
            assert_eq!(media.protection, MediaProtection::Clear);
            assert!(media.streams[0].public_headers.is_empty());
            assert!(media.streams[0].upstream_access_ref.is_none());
        }
    }

    #[test]
    fn non_media_url_is_not_claimed() {
        assert!(
            !adapter()
                .recognize("https://example.com/page")
                .unwrap()
                .matched
        );
    }

    fn adapter() -> GenericDirectAdapter {
        GenericDirectAdapter
    }
}
