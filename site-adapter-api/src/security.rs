//! Shared low-level Secret-header classification.
//!
//! This rule is intentionally conservative.  Both the SiteAdapter public
//! output boundary and Gateway/R008 response filtering consume it so a plugin
//! cannot pass conformance with weaker Secret semantics than Core.

/// Return true when header material must stay behind the scoped access
/// boundary instead of being exposed to a display or generic caller.
pub fn is_secret_header(name: &str, value: &str) -> bool {
    let normalized_name = name.to_ascii_lowercase().replace('_', "-");
    let normalized_value = value.trim().to_ascii_lowercase();
    matches!(
        normalized_name.as_str(),
        "authorization"
            | "proxy-authorization"
            | "cookie"
            | "set-cookie"
            | "x-api-key"
            | "x-auth-token"
            | "proxy-authenticate"
            | "www-authenticate"
            | "api-key"
            | "access-token"
            | "refresh-token"
            | "id-token"
    ) || normalized_value.starts_with("bearer ")
        || normalized_value.starts_with("basic ")
}

#[cfg(test)]
mod tests {
    use super::is_secret_header;

    #[test]
    fn accepted_r008_header_names_and_credential_schemes_are_secret() {
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
            assert!(is_secret_header(name, value), "{name}: {value}");
        }
    }

    #[test]
    fn ordinary_public_headers_remain_allowed() {
        for (name, value) in [("Accept", "video/mp4"), ("X-Trace", "request-123")] {
            assert!(!is_secret_header(name, value), "{name}: {value}");
        }
    }
}
