use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::Command;
use std::time::{Duration, Instant};

use tokio::net::lookup_host;
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EgressScope {
    PublicWeb,
    /// The identifier is only useful when the Core has an administrator-created
    /// entry with the same name. It is not a caller-selected URL or a bypass flag.
    ConfiguredLocalService(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EgressPolicyError {
    InvalidScheme,
    MissingHost,
    InvalidPort,
    DnsLookupFailed,
    TargetRejected,
    LocalServiceNotConfigured,
    LocalServiceOriginMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTarget {
    host: String,
    addresses: Vec<SocketAddr>,
}

impl ValidatedTarget {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }

    /// Keep the hostname in the URL for HTTP Host/TLS verification while
    /// forcing the connector to use exactly the addresses checked by policy.
    pub fn pinned_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        self.pinned_client_with_timeout(None)
    }

    pub fn pinned_client_with_timeout(
        &self,
        timeout: Option<Duration>,
    ) -> Result<reqwest::Client, reqwest::Error> {
        let mut builder = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve_to_addrs(&self.host, &self.addresses);
        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }
        builder.build()
    }
}

#[derive(Clone, Debug)]
struct LocalService {
    scheme: String,
    host: String,
    port: Option<u16>,
}

#[derive(Clone, Debug, Default)]
pub struct EgressPolicy {
    local_services: HashMap<String, LocalService>,
}

impl EgressPolicy {
    pub fn configure_local_service(
        &mut self,
        name: impl Into<String>,
        origin: &Url,
    ) -> Result<(), EgressPolicyError> {
        if !matches!(origin.scheme(), "http" | "https") {
            return Err(EgressPolicyError::InvalidScheme);
        }
        let host = origin
            .host_str()
            .ok_or(EgressPolicyError::MissingHost)?
            .to_ascii_lowercase();
        let port = origin
            .port_or_known_default()
            .ok_or(EgressPolicyError::InvalidPort)?;
        let name = name.into();
        if name.is_empty() {
            return Err(EgressPolicyError::LocalServiceNotConfigured);
        }
        self.local_services.insert(
            name,
            LocalService {
                scheme: origin.scheme().to_string(),
                host,
                port: Some(port),
            },
        );
        Ok(())
    }

    pub async fn validate(&self, url: &Url, scope: &EgressScope) -> Result<(), EgressPolicyError> {
        self.validate_and_resolve(url, scope).await.map(|_| ())
    }

    pub async fn validate_and_resolve(
        &self,
        url: &Url,
        scope: &EgressScope,
    ) -> Result<ValidatedTarget, EgressPolicyError> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(EgressPolicyError::InvalidScheme);
        }
        let host = url.host_str().ok_or(EgressPolicyError::MissingHost)?;
        let port = url
            .port_or_known_default()
            .ok_or(EgressPolicyError::InvalidPort)?;
        match scope {
            EgressScope::PublicWeb => self.validate_public(host, port).await,
            EgressScope::ConfiguredLocalService(name) => {
                let Some(service) = self.local_services.get(name) else {
                    return Err(EgressPolicyError::LocalServiceNotConfigured);
                };
                if !url.scheme().eq_ignore_ascii_case(&service.scheme)
                    || !host.eq_ignore_ascii_case(&service.host)
                    || Some(port) != service.port
                {
                    return Err(EgressPolicyError::LocalServiceOriginMismatch);
                }
                resolve_target(host, port, false).await
            }
        }
    }

    async fn validate_public(
        &self,
        host: &str,
        port: u16,
    ) -> Result<ValidatedTarget, EgressPolicyError> {
        if is_forbidden_hostname(host) {
            return Err(EgressPolicyError::TargetRejected);
        }
        resolve_target(host, port, true).await
    }
}

async fn resolve_target(
    host: &str,
    port: u16,
    require_public: bool,
) -> Result<ValidatedTarget, EgressPolicyError> {
    let addresses: Vec<SocketAddr> = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        lookup_host((host, port))
            .await
            .map_err(|_| EgressPolicyError::DnsLookupFailed)?
            .collect()
    };
    if addresses.is_empty()
        || (require_public
            && addresses
                .iter()
                .any(|address| !is_public_web_ip(address.ip())))
    {
        return Err(EgressPolicyError::TargetRejected);
    }
    Ok(ValidatedTarget {
        host: host.to_string(),
        addresses,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpAuthorityError {
    InvalidScheme,
    MissingHost,
    InvalidPort,
    InvalidOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HttpAuthority {
    scheme: String,
    host: String,
    port: u16,
}

#[derive(Clone, Debug, Default)]
pub struct HttpAuthorityPolicy {
    authorities: Vec<HttpAuthority>,
}

impl HttpAuthorityPolicy {
    pub fn configure(&mut self, origin: &Url) -> Result<(), HttpAuthorityError> {
        if !matches!(origin.scheme(), "http" | "https") {
            return Err(HttpAuthorityError::InvalidScheme);
        }
        if !origin.username().is_empty()
            || origin.password().is_some()
            || !matches!(origin.path(), "" | "/")
            || origin.query().is_some()
            || origin.fragment().is_some()
        {
            return Err(HttpAuthorityError::InvalidOrigin);
        }
        let host = origin
            .host_str()
            .ok_or(HttpAuthorityError::MissingHost)
            .map(canonical_host)?;
        let port = origin
            .port_or_known_default()
            .ok_or(HttpAuthorityError::InvalidPort)?;
        self.authorities = vec![HttpAuthority {
            scheme: origin.scheme().to_ascii_lowercase(),
            host,
            port,
        }];
        Ok(())
    }

    pub fn allows_host(&self, host: &str) -> bool {
        let Some(parsed) = parse_host_authority(host) else {
            return false;
        };
        self.authorities.iter().any(|authority| {
            parsed.host == authority.host
                && parsed.port.map_or(
                    default_port(&authority.scheme) == Some(authority.port),
                    |port| port == authority.port,
                )
        })
    }

    pub fn allows_origin_for_host(&self, origin: &str, host: &str) -> bool {
        let Some(host) = parse_host_authority(host) else {
            return false;
        };
        let Ok(parsed) = Url::parse(origin) else {
            return false;
        };
        if !matches!(parsed.scheme(), "http" | "https")
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || !matches!(parsed.path(), "" | "/")
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return false;
        }
        let Some(origin_host) = parsed.host_str().map(canonical_host) else {
            return false;
        };
        let Some(origin_port) = parsed.port_or_known_default() else {
            return false;
        };
        self.authorities.iter().any(|authority| {
            authority.scheme == parsed.scheme()
                && authority.host == host.host
                && authority.host == origin_host
                && authority.port == origin_port
                && self.allows_host(host.raw)
        })
    }
}

pub fn is_valid_http_host(value: &str) -> bool {
    parse_host_authority(value).is_some()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedHostAuthority<'a> {
    raw: &'a str,
    host: String,
    port: Option<u16>,
}

fn parse_host_authority(value: &str) -> Option<ParsedHostAuthority<'_>> {
    if value.trim() != value || value.is_empty() || value.contains(['/', '?', '#', '@']) {
        return None;
    }
    let parsed = Url::parse(&format!("http://{value}")).ok()?;
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        return None;
    }
    Some(ParsedHostAuthority {
        raw: value,
        host: canonical_host(parsed.host_str()?),
        port: parsed.port(),
    })
}

fn canonical_host(host: &str) -> String {
    host.trim_end_matches('.').to_ascii_lowercase()
}

fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "http" => Some(80),
        "https" => Some(443),
        _ => None,
    }
}

fn is_forbidden_hostname(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "metadata.google.internal"
        || host == "metadata"
        || host == "ip6-localhost"
}

pub fn is_public_web_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => is_public_v6(ip),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || octets[0] == 0
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 192 && octets[1] == 88 && octets[2] == 99)
        || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
        || octets[0] >= 240)
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    let mapped_v4 = if segments[..5].iter().all(|segment| *segment == 0) && segments[5] == 0xffff {
        Some(Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            segments[6] as u8,
            (segments[7] >> 8) as u8,
            segments[7] as u8,
        ))
    } else {
        None
    };
    !(mapped_v4.is_some_and(|ip| !is_public_v4(ip))
        || ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (segments[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (segments[0] & 0xffc0) == 0xfe80 // link local fe80::/10
        || (segments[0] == 0x2001 && segments[1] == 0x0db8) // documentation
        || (segments[0] == 0x2001 && segments[1] == 0x0002) // benchmarking
        || (segments[0] == 0x2001 && (segments[1] & 0xfff0) == 0x0010)) // ORCHID
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SiteAccessError {
    Expired,
    SiteMismatch,
    AccountMismatch,
    HostNotAllowed,
    InvalidUrl,
    SessionNotBound,
    SessionExpired,
    StaleSession,
}

/// A capability contains scope metadata only. Raw Vault material is never part
/// of this value and is injected by the Core-owned HTTP implementation.
#[derive(Clone, Debug)]
pub struct SiteAccessCapability {
    site_id: String,
    account_ref: Option<String>,
    allowed_hosts: HashSet<String>,
    expires_at: Instant,
    capability_id: String,
    session_id: Option<String>,
}

impl SiteAccessCapability {
    pub fn issue(
        site_id: impl Into<String>,
        account_ref: Option<String>,
        allowed_hosts: impl IntoIterator<Item = String>,
        capability_id: impl Into<String>,
        ttl: Duration,
    ) -> Self {
        Self {
            site_id: site_id.into(),
            account_ref,
            allowed_hosts: allowed_hosts
                .into_iter()
                .map(|host| host.trim_end_matches('.').to_ascii_lowercase())
                .collect(),
            expires_at: Instant::now() + ttl,
            capability_id: capability_id.into(),
            session_id: None,
        }
    }

    pub fn issue_for_session(
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
        allowed_hosts: impl IntoIterator<Item = String>,
        capability_id: impl Into<String>,
        session_id: impl Into<String>,
        ttl: Duration,
    ) -> Self {
        let mut capability = Self::issue(
            site_id,
            Some(account_ref.into()),
            allowed_hosts,
            capability_id,
            ttl,
        );
        capability.session_id = Some(session_id.into());
        capability
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn account_ref(&self) -> Option<&str> {
        self.account_ref.as_deref()
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    pub fn allowed_hosts(&self) -> impl Iterator<Item = &str> {
        self.allowed_hosts.iter().map(String::as_str)
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn authorize(&self, site_id: &str, url: &Url) -> Result<(), SiteAccessError> {
        self.authorize_for_account(site_id, None, url)
    }

    pub fn authorize_for_account(
        &self,
        site_id: &str,
        account_ref: Option<&str>,
        url: &Url,
    ) -> Result<(), SiteAccessError> {
        if self.expires_at <= Instant::now() {
            return Err(SiteAccessError::Expired);
        }
        if self.site_id != site_id {
            return Err(SiteAccessError::SiteMismatch);
        }
        if let Some(expected_account) = self.account_ref.as_deref() {
            if let Some(account_ref) = account_ref
                && account_ref != expected_account
            {
                return Err(SiteAccessError::AccountMismatch);
            }
        }
        if !matches!(url.scheme(), "http" | "https")
            || url.username() != ""
            || url.password().is_some()
            || url.host_str().is_none()
        {
            return Err(SiteAccessError::InvalidUrl);
        }
        let host = url.host_str().ok_or(SiteAccessError::InvalidUrl)?;
        if !self
            .allowed_hosts
            .contains(&host.trim_end_matches('.').to_ascii_lowercase())
        {
            return Err(SiteAccessError::HostNotAllowed);
        }
        Ok(())
    }
}

pub fn is_secret_header(name: &str, value: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let value = value.trim().to_ascii_lowercase();
    matches!(
        name.as_str(),
        "authorization"
            | "proxy-authorization"
            | "cookie"
            | "set-cookie"
            | "x-api-key"
            | "x-auth-token"
            | "proxy-authenticate"
    ) || value.starts_with("bearer ")
        || value.starts_with("basic ")
}

pub fn redact_text(input: &str, secrets: &[&str]) -> String {
    secrets.iter().fold(input.to_string(), |output, secret| {
        if secret.is_empty() {
            output
        } else {
            output.replace(secret, "[REDACTED]")
        }
    })
}

pub fn redact_url(url: &Url) -> String {
    let mut safe = url.clone();
    safe.set_query(None);
    safe.set_fragment(None);
    safe.to_string()
}

/// External tools are represented as argv, never as a shell command string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredCommand {
    program: OsString,
    args: Vec<OsString>,
}

impl StructuredCommand {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn argv(&self) -> impl Iterator<Item = &OsStr> {
        std::iter::once(self.program.as_os_str()).chain(self.args.iter().map(OsString::as_os_str))
    }

    pub fn into_command(self) -> Command {
        let mut command = Command::new(self.program);
        command.args(self.args);
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use tokio::net::TcpListener;

    #[test]
    fn forbidden_ip_matrix_is_rejected() {
        for value in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.169.254",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "224.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::ffff:127.0.0.1",
            "fc00::1",
            "fe80::1",
            "2001:db8::1",
            "ff02::1",
        ] {
            assert!(!is_public_web_ip(value.parse().unwrap()), "{value}");
        }
        for value in ["8.8.8.8", "1.1.1.1", "2001:4860:4860::8888"] {
            assert!(is_public_web_ip(value.parse().unwrap()), "{value}");
        }
    }

    #[tokio::test]
    async fn configured_local_service_is_named_and_origin_bound() {
        let mut policy = EgressPolicy::default();
        policy
            .configure_local_service("jellyfin", &Url::parse("http://127.0.0.1:8096/").unwrap())
            .unwrap();
        let scope = EgressScope::ConfiguredLocalService("jellyfin".into());
        assert!(
            policy
                .validate(&Url::parse("http://127.0.0.1:8096/Items").unwrap(), &scope)
                .await
                .is_ok()
        );
        assert_eq!(
            policy
                .validate(
                    &Url::parse("http://169.254.169.254/latest").unwrap(),
                    &scope
                )
                .await,
            Err(EgressPolicyError::LocalServiceOriginMismatch)
        );
        assert_eq!(
            policy
                .validate(
                    &Url::parse("http://127.0.0.1:8096/Items").unwrap(),
                    &EgressScope::ConfiguredLocalService("user-selected".into())
                )
                .await,
            Err(EgressPolicyError::LocalServiceNotConfigured)
        );
    }

    #[tokio::test]
    async fn pinned_client_uses_validated_address_without_a_second_dns_lookup() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = axum::Router::new().route("/", get(|| async { "pinned" }));
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let target = ValidatedTarget {
            host: "dns-race.invalid".into(),
            addresses: vec![address],
        };
        let response = target
            .pinned_client()
            .unwrap()
            .get(format!("http://dns-race.invalid:{}/", address.port()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.text().await.unwrap(), "pinned");
    }

    #[test]
    fn site_capability_is_site_host_and_expiry_scoped() {
        let cap = SiteAccessCapability::issue(
            "video-site",
            Some("account-a".into()),
            ["media.example.test".into()],
            "cap-a",
            Duration::from_secs(60),
        );
        assert!(
            cap.authorize(
                "video-site",
                &Url::parse("https://media.example.test/a").unwrap()
            )
            .is_ok()
        );
        assert_eq!(
            cap.authorize(
                "other-site",
                &Url::parse("https://media.example.test/a").unwrap()
            ),
            Err(SiteAccessError::SiteMismatch)
        );
        assert_eq!(
            cap.authorize(
                "video-site",
                &Url::parse("https://other.example.test/a").unwrap()
            ),
            Err(SiteAccessError::HostNotAllowed)
        );
        let expired = SiteAccessCapability::issue(
            "video-site",
            None,
            ["media.example.test".into()],
            "cap-expired",
            Duration::ZERO,
        );
        assert_eq!(
            expired.authorize(
                "video-site",
                &Url::parse("https://media.example.test/a").unwrap()
            ),
            Err(SiteAccessError::Expired)
        );
    }

    #[test]
    fn redaction_removes_secret_and_signed_query_material() {
        let secret = "fixture-secret-7f9b";
        let diagnostic = format!("upstream Authorization: Bearer {secret}");
        assert!(!redact_text(&diagnostic, &[secret]).contains(secret));
        let url = Url::parse("https://cdn.example.test/a.m3u8?token=fixture-secret-7f9b").unwrap();
        assert_eq!(redact_url(&url), "https://cdn.example.test/a.m3u8");
    }

    #[test]
    fn structured_command_preserves_argument_boundaries() {
        let command = StructuredCommand::new("ffmpeg")
            .arg("-i")
            .arg("https://example.test/a?title=hello;echo-pwned")
            .arg("output file.mp4");
        let args: Vec<_> = command
            .argv()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "ffmpeg",
                "-i",
                "https://example.test/a?title=hello;echo-pwned",
                "output file.mp4"
            ]
        );
    }
}
