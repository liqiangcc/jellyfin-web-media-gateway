use bilibili::{BilibiliAdapter, FROZEN_BVID};
use gateway_core::{EgressPolicy, EgressScope};
use site_adapter_api::SiteAdapter;
use std::process::ExitCode;
use url::Url;

const SAMPLE_URL: &str = "https://www.bilibili.com/video/BV14V411W7r5/";
const MAX_PAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

#[tokio::main]
async fn main() -> ExitCode {
    let adapter = BilibiliAdapter;
    let locator = adapter
        .recognize(SAMPLE_URL)
        .expect("recognize frozen public URL")
        .locator
        .expect("frozen public URL must match");
    let (status, html) = match fetch_public_page().await {
        Ok(value) => value,
        Err(FetchError::HttpStatus(status)) => {
            println!(
                "result=BLOCKED (public page returned HTTP {status}; no challenge bypass attempted)"
            );
            return ExitCode::from(2);
        }
        Err(error) => {
            println!("result=BLOCKED (central public-web fetch failed: {error})");
            return ExitCode::from(2);
        }
    };
    let media = match adapter.resolve_public_html(&locator, &html) {
        Ok(media) => media,
        Err(error) => {
            println!("result=FAIL (plugin resolution returned {error:?})");
            return ExitCode::from(1);
        }
    };
    let navigation = match adapter.navigation_from_html(&locator, &html) {
        Ok(navigation) => navigation,
        Err(error) => {
            println!("result=FAIL (plugin navigation returned {error:?})");
            return ExitCode::from(1);
        }
    };
    let protocol = media
        .streams
        .first()
        .map(|stream| format!("{:?}", stream.protocol))
        .unwrap_or_else(|| "none".into());
    println!("site=bilibili plugin=bilibili-public locator_version=1");
    println!("bvid={FROZEN_BVID} http_status={status} protocol={protocol}");
    println!("expires_at_unix={:?}", media.expires_at_unix);
    println!(
        "navigation current_index={:?} has_previous={} has_next={}",
        navigation.current_index,
        navigation.previous.is_some(),
        navigation.next.is_some()
    );
    ExitCode::SUCCESS
}

#[derive(Debug)]
enum FetchError {
    Egress(gateway_core::EgressPolicyError),
    Request,
    BodyTooLarge,
    HttpStatus(u16),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Egress(error) => write!(f, "egress policy rejected request ({error:?})"),
            Self::Request => f.write_str("upstream request failed"),
            Self::BodyTooLarge => f.write_str("public document exceeded size limit"),
            Self::HttpStatus(status) => write!(f, "HTTP {status}"),
        }
    }
}

async fn fetch_public_page() -> Result<(u16, String), FetchError> {
    let policy = EgressPolicy::default();
    let mut url = Url::parse(SAMPLE_URL).expect("frozen sample URL");
    for hop in 0..=MAX_REDIRECTS {
        let target = policy
            .validate_and_resolve(&url, &EgressScope::PublicWeb)
            .await
            .map_err(FetchError::Egress)?;
        let client = target.pinned_client().map_err(|_| FetchError::Request)?;
        let response = client
            .get(url.clone())
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|_| FetchError::Request)?;
        if response.status().is_redirection() {
            if hop == MAX_REDIRECTS {
                return Err(FetchError::Request);
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or(FetchError::Request)?;
            url = url.join(location).map_err(|_| FetchError::Request)?;
            continue;
        }
        let status = response.status().as_u16();
        if status != 200 {
            return Err(FetchError::HttpStatus(status));
        }
        if response
            .content_length()
            .is_some_and(|length| length as usize > MAX_PAGE_BYTES)
        {
            return Err(FetchError::BodyTooLarge);
        }
        let body = response.bytes().await.map_err(|_| FetchError::Request)?;
        if body.len() > MAX_PAGE_BYTES {
            return Err(FetchError::BodyTooLarge);
        }
        return String::from_utf8(body.to_vec())
            .map(|body| (status, body))
            .map_err(|_| FetchError::Request);
    }
    Err(FetchError::Request)
}
