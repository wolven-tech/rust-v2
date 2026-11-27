//! The provider-agnostic half of the OAuth sign-in flow: the sealed pending-origin
//! cookie and the return-target check. Nothing here knows which provider runs.
//!
//! The browser chooses where to land after sign-in, so that choice cannot ride in
//! a query parameter the callback trusts. The start route seals the target into a
//! short-lived cookie bound to the OAuth `state`; the callback opens it, and
//! [`admit`] decides whether the target is safe to redirect to.
//!
//! The redirect itself lives in `oauth_routes`. Its VERIFY is [`admit`].

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// `__Host-` makes browsers accept this only with `Secure`, `Path=/` and no
/// `Domain`, so a sibling subdomain cannot set or shadow it.
pub const PENDING_COOKIE: &str = "__Host-rv2_oauth_pending";
/// Shorter than the 600 s better-auth gives its state record, so the cookie
/// expires before the state it is bound to.
pub const PENDING_TTL_SECS: i64 = 540;
const PAYLOAD_VERSION: u8 = 1;
const MAX_PATH_LEN: usize = 2048;

/// What the sealed cookie carries between the start and the callback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    version: u8,
    pub provider: String,
    pub state: String,
    pub origin: String,
    pub path: String,
    pub expires_at: i64,
}

impl Pending {
    #[must_use]
    pub fn new(provider: &str, state: &str, target: &ReturnTarget, now: i64) -> Self {
        Self {
            version: PAYLOAD_VERSION,
            provider: provider.to_string(),
            state: state.to_string(),
            origin: target.origin.clone(),
            path: target.path.clone(),
            expires_at: now + PENDING_TTL_SECS,
        }
    }
}

/// A destination the server has checked. Only [`admit`] builds one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnTarget {
    pub origin: String,
    pub path: String,
}

impl ReturnTarget {
    /// The app's home on the primary origin: where every refused target lands.
    #[must_use]
    pub fn home(allowlist: &[String]) -> Self {
        Self {
            origin: allowlist.first().cloned().unwrap_or_default(),
            path: "/".to_string(),
        }
    }

    #[must_use]
    pub fn url(&self) -> String {
        format!("{}{}", self.origin, self.path)
    }
}

/// Why a cookie or a target was not admitted. Logged by name; never echoed to the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No cookie, or no pending value in the request.
    Missing,
    /// Not `payload.signature`, or the payload does not parse.
    Malformed,
    /// The HMAC does not match: tampered, or sealed with another secret.
    BadSignature,
    Expired,
    /// The sealed `state` is not the one the callback presented.
    StateMismatch,
    /// The cookie names a provider other than the route's.
    ProviderMismatch,
    /// More than one pending cookie reached the callback. The browser sends every
    /// cookie that matches, so a second one can only come from someone else's flow.
    DuplicateCookie,
    /// This process has already consumed the sealed `state`.
    AlreadyUsed,
    /// The origin is not on the allowlist.
    UnknownOrigin,
    /// The path is absolute, protocol-relative, or contains a control character,
    /// a space or a backslash, or a percent-escape anywhere in the path. (Query
    /// values are checked by `UnsafeQuery`.)
    NotSameOriginPath,
    /// A query value is not printable ASCII raw, is not UTF-8 once decoded, has a
    /// control character once decoded, or is double-encoded.
    UnsafeQuery,
    /// The shared single-use claim could not be written. Refused, not retried.
    ClaimUnavailable,
    /// The audit record for an account change could not be written.
    AuditUnavailable,
}

fn mac(secret: &[u8]) -> Hmac<Sha256> {
    // HMAC accepts keys of any length; the minimum length is enforced at boot.
    Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts any key length")
}

/// `payload.signature`, both base64url without padding.
pub fn seal(secret: &[u8], pending: &Pending) -> Result<String, serde_json::Error> {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(pending)?);
    let mut signer = mac(secret);
    signer.update(payload.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(signer.finalize().into_bytes());
    Ok(format!("{payload}.{signature}"))
}

/// Verify the signature before reading anything in the payload.
pub fn open(secret: &[u8], sealed: &str, now: i64) -> Result<Pending, Refusal> {
    let (payload, signature) = sealed.split_once('.').ok_or(Refusal::Malformed)?;
    if signature.contains('.') {
        return Err(Refusal::Malformed);
    }
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| Refusal::Malformed)?;
    let mut verifier = mac(secret);
    verifier.update(payload.as_bytes());
    verifier
        .verify_slice(&signature)
        .map_err(|_| Refusal::BadSignature)?;

    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| Refusal::Malformed)?;
    let pending: Pending = serde_json::from_slice(&bytes).map_err(|_| Refusal::Malformed)?;
    if pending.version != PAYLOAD_VERSION {
        return Err(Refusal::Malformed);
    }
    if now >= pending.expires_at {
        return Err(Refusal::Expired);
    }
    Ok(pending)
}

/// The form every account email is stored and looked up in. Password sign-up and
/// sign-in apply it at the gate; the Google identity applies it in `better.rs`.
#[must_use]
pub fn normalise_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

/// VERIFY for the redirect sink. Returns the target only if `origin` is on the
/// allowlist and `path` is a same-origin relative path.
///
/// `app` is `None` at the start route to mean the primary origin. At the callback
/// it is always the origin the sealed cookie carried, which is checked again here
/// because the allowlist may have changed since the cookie was sealed.
pub fn admit(
    allowlist: &[String],
    app: Option<&str>,
    path: Option<&str>,
) -> Result<ReturnTarget, Refusal> {
    let origin = match app {
        None => allowlist.first().ok_or(Refusal::UnknownOrigin)?.as_str(),
        Some(candidate) => allowlist
            .iter()
            .map(String::as_str)
            .find(|allowed| *allowed == candidate)
            .ok_or(Refusal::UnknownOrigin)?,
    };
    let path = match path {
        None => "/",
        Some(raw) => admit_path(raw)?,
    };
    Ok(ReturnTarget {
        origin: origin.to_string(),
        path: path.to_string(),
    })
}

/// The path takes no percent-escapes, so no encoded `/`, `\` or `.` can change its
/// segments. The query may carry escapes: each value is decoded once for the
/// checks, and the raw form is what is returned, so the Location stays ASCII.
fn admit_path(raw: &str) -> Result<&str, Refusal> {
    if raw.len() > MAX_PATH_LEN || raw.contains('#') {
        return Err(Refusal::NotSameOriginPath);
    }
    let (path, query) = match raw.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (raw, None),
    };
    check_path_shape(path)?;
    if path.contains('%') {
        return Err(Refusal::NotSameOriginPath);
    }
    if let Some(query) = query {
        check_query(query)?;
    }

    // Resolve against a fixed origin the way a browser would. The host must not
    // move, which catches any authority the shape check missed.
    let base = url::Url::parse("https://app.invalid/").map_err(|_| Refusal::NotSameOriginPath)?;
    let resolved = base.join(raw).map_err(|_| Refusal::NotSameOriginPath)?;
    if resolved.host_str() != base.host_str() {
        return Err(Refusal::NotSameOriginPath);
    }
    Ok(raw)
}

fn check_path_shape(path: &str) -> Result<(), Refusal> {
    if !path.starts_with('/') {
        return Err(Refusal::NotSameOriginPath);
    }
    // `//host` and `/\host` are authority-relative: a browser leaves this origin.
    if matches!(path.as_bytes().get(1), Some(b'/' | b'\\')) {
        return Err(Refusal::NotSameOriginPath);
    }
    // Printable ASCII, no backslash: rules out CR, LF, NUL and tab.
    if path
        .bytes()
        .any(|b| !(0x21..=0x7e).contains(&b) || b == b'\\')
    {
        return Err(Refusal::NotSameOriginPath);
    }
    Ok(())
}

/// The raw query is printable ASCII (so a space is `%20` or `+`). Its decoded form
/// must be UTF-8 with no control characters and no escape of its own, which
/// refuses double encoding such as `%252F`.
fn check_query(query: &str) -> Result<(), Refusal> {
    if query.bytes().any(|b| !(0x21..=0x7e).contains(&b)) {
        return Err(Refusal::UnsafeQuery);
    }
    let decoded = decode_once(query).ok_or(Refusal::UnsafeQuery)?;
    if decoded.chars().any(char::is_control) || has_escape(&decoded) {
        return Err(Refusal::UnsafeQuery);
    }
    Ok(())
}

/// Each `%XX` decoded once, where `XX` is exactly two hex digits. `None` if the
/// result is not UTF-8.
fn decode_once(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let high = bytes.get(i + 1).and_then(|&b| hex_value(b));
            let low = bytes.get(i + 2).and_then(|&b| hex_value(b));
            if let (Some(high), Some(low)) = (high, low) {
                out.push(high * 16 + low);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn has_escape(text: &str) -> bool {
    text.as_bytes()
        .windows(3)
        .any(|w| w[0] == b'%' && w[1].is_ascii_hexdigit() && w[2].is_ascii_hexdigit())
}

/// `Set-Cookie` for the sealed value. `Secure` and `SameSite=Lax` are both
/// required: the callback is a top-level GET from the provider, and Lax is what
/// lets the cookie ride on it.
#[must_use]
pub fn set_cookie(sealed: &str) -> String {
    format!(
        "{PENDING_COOKIE}={sealed}; Path=/; Max-Age={PENDING_TTL_SECS}; HttpOnly; Secure; SameSite=Lax"
    )
}

#[must_use]
pub fn clear_cookie() -> String {
    format!("{PENDING_COOKIE}=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Lax")
}

/// The one sealed value in a `Cookie` header. Every cookie with the name counts,
/// empty ones included, so a planted duplicate is refused rather than skipped.
pub fn pending_cookie(cookie_header: &str) -> Result<&str, Refusal> {
    let values: Vec<&str> = cookie_header
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == PENDING_COOKIE).then_some(value)
        })
        .collect();
    match values.as_slice() {
        [] => Err(Refusal::Missing),
        [only] if !only.is_empty() => Ok(*only),
        [_] => Err(Refusal::Missing),
        _ => Err(Refusal::DuplicateCookie),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-pending-secret-that-is-at-least-32-bytes";
    const NOW: i64 = 1_800_000_000;
    const ALLOWED: &[&str] = &["http://localhost:4402", "http://localhost:4401"];

    fn allowlist() -> Vec<String> {
        ALLOWED.iter().map(|s| s.to_string()).collect()
    }

    fn sealed_for(state: &str, now: i64) -> String {
        let target = ReturnTarget {
            origin: "http://localhost:4402".into(),
            path: "/".into(),
        };
        seal(SECRET, &Pending::new("google", state, &target, now)).expect("seals")
    }

    #[test]
    fn a_sealed_pending_opens_within_its_ttl() {
        let sealed = sealed_for("s1", NOW);
        let opened = open(SECRET, &sealed, NOW + 1).expect("opens");
        assert_eq!(opened.state, "s1");
        assert_eq!(opened.provider, "google");
    }

    #[test]
    fn an_expired_pending_is_refused() {
        let sealed = sealed_for("s1", NOW);
        assert_eq!(
            open(SECRET, &sealed, NOW + PENDING_TTL_SECS),
            Err(Refusal::Expired)
        );
    }

    #[test]
    fn a_tampered_payload_fails_the_signature() {
        let sealed = sealed_for("s1", NOW);
        let (payload, signature) = sealed.split_once('.').expect("has a dot");
        let forged = Pending {
            version: PAYLOAD_VERSION,
            provider: "google".into(),
            state: "s1".into(),
            origin: "https://evil.example".into(),
            path: "/".into(),
            expires_at: NOW + 60,
        };
        let forged_payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&forged).expect("json"));
        assert_ne!(payload, forged_payload);
        assert_eq!(
            open(SECRET, &format!("{forged_payload}.{signature}"), NOW),
            Err(Refusal::BadSignature)
        );
    }

    #[test]
    fn a_cookie_sealed_with_another_secret_is_refused() {
        let other = b"a-different-secret-of-sufficient-length!!";
        let target = ReturnTarget {
            origin: "http://localhost:4402".into(),
            path: "/".into(),
        };
        let sealed = seal(other, &Pending::new("google", "s1", &target, NOW)).expect("seals");
        assert_eq!(open(SECRET, &sealed, NOW), Err(Refusal::BadSignature));
    }

    #[test]
    fn a_value_without_a_signature_is_malformed() {
        assert_eq!(open(SECRET, "abc", NOW), Err(Refusal::Malformed));
        assert_eq!(open(SECRET, "a.b.c", NOW), Err(Refusal::Malformed));
    }

    #[test]
    fn tainted_return_paths_are_refused() {
        let refused = [
            "https://evil.example/x",
            "http://evil.example",
            "//evil.example",
            "/\\evil.example",
            "/\\\\evil.example",
            "/%2F%2Fevil.example",
            "/%2f%2fevil.example",
            "/%252F%252Fevil.example",
            "/%5Cevil.example",
            "/%09/evil.example",
            "/%0d%0aSet-Cookie:x",
            "/%20/x",
            "/x y",
            "/\tevil.example",
            "/\u{ff0f}evil.example",
            "javascript:alert(1)",
            "posts",
            "",
        ];
        for path in refused {
            assert_eq!(
                admit(&allowlist(), Some("http://localhost:4402"), Some(path)),
                Err(Refusal::NotSameOriginPath),
                "admitted {path:?}"
            );
        }
    }

    #[test]
    fn a_cleared_path_on_an_allowlisted_origin_is_admitted() {
        let target = admit(
            &allowlist(),
            Some("http://localhost:4401"),
            Some("/posts/abc?tab=1"),
        )
        .expect("admitted");
        assert_eq!(target.url(), "http://localhost:4401/posts/abc?tab=1");
    }

    #[test]
    fn an_origin_off_the_allowlist_is_refused() {
        assert_eq!(
            admit(&allowlist(), Some("https://evil.example"), Some("/")),
            Err(Refusal::UnknownOrigin)
        );
        assert_eq!(
            admit(&allowlist(), Some("http://localhost:4402/"), Some("/")),
            Err(Refusal::UnknownOrigin)
        );
    }

    /// The admitted rows the matrix was missing: the root, a query, and a
    /// percent-encoded segment that decodes to printable ASCII.
    #[test]
    fn root_and_query_targets_are_admitted() {
        let cases = [
            ("/", "http://localhost:4402/"),
            ("/posts?tab=1", "http://localhost:4402/posts?tab=1"),
            (
                "/posts?q=abc%2Fdef",
                "http://localhost:4402/posts?q=abc%2Fdef",
            ),
        ];
        for (path, expected) in cases {
            let target = admit(&allowlist(), Some("http://localhost:4402"), Some(path))
                .unwrap_or_else(|e| panic!("refused {path:?}: {e:?}"));
            assert_eq!(target.url(), expected);
        }
    }

    /// A target sealed while its origin was listed is refused once the operator
    /// drops that origin from the allowlist.
    #[test]
    fn an_origin_removed_after_sealing_is_refused_at_the_callback() {
        let shrunk = vec!["http://localhost:4401".to_string()];
        assert_eq!(
            admit(&shrunk, Some("http://localhost:4402"), Some("/posts")),
            Err(Refusal::UnknownOrigin)
        );
    }

    /// Lookups and writes must agree, so both sides use this one function.
    #[test]
    fn emails_are_normalised_to_lower_case() {
        assert_eq!(
            normalise_email("  Victim@Example.COM "),
            "victim@example.com"
        );
    }

    #[test]
    fn no_app_means_the_primary_origin() {
        let target = admit(&allowlist(), None, None).expect("admitted");
        assert_eq!(target, ReturnTarget::home(&allowlist()));
        assert_eq!(target.url(), "http://localhost:4402/");
    }

    /// Encoded query values are admitted, and the raw form is what is returned.
    #[test]
    fn encoded_query_values_are_admitted_as_written() {
        for (path, expected) in [
            (
                "/posts?q=hello%20world",
                "http://localhost:4402/posts?q=hello%20world",
            ),
            (
                "/posts?q=caf%C3%A9",
                "http://localhost:4402/posts?q=caf%C3%A9",
            ),
            (
                "/posts?q=a+b&tab=1",
                "http://localhost:4402/posts?q=a+b&tab=1",
            ),
            ("/posts?q=100%25", "http://localhost:4402/posts?q=100%25"),
        ] {
            let target = admit(&allowlist(), Some("http://localhost:4402"), Some(path))
                .unwrap_or_else(|e| panic!("refused {path:?}: {e:?}"));
            assert_eq!(target.url(), expected);
        }
    }

    #[test]
    fn unsafe_query_values_are_refused() {
        for path in [
            "/posts?q=%252F",
            "/posts?q=%2541",
            "/posts?q=%0d%0a",
            "/posts?q=%00",
            "/posts?q=a b",
            "/posts?q=%FF",
            "/posts?q=caf\u{e9}",
            "/posts#frag",
            "/posts/abc%2Fdef",
            "/%2F%2Fevil.example?q=1",
        ] {
            assert!(
                admit(&allowlist(), Some("http://localhost:4402"), Some(path)).is_err(),
                "admitted {path:?}"
            );
        }
    }

    /// A `+` is not a hex digit, so `%+1` is not an escape: it stays literal text,
    /// is not decoded to a control byte, and is returned as written.
    #[test]
    fn a_percent_escape_needs_two_hex_digits() {
        assert_eq!(decode_once("%+1"), Some("%+1".to_string()));
        assert_eq!(decode_once("%41"), Some("A".to_string()));
        let target = admit(
            &allowlist(),
            Some("http://localhost:4402"),
            Some("/posts?q=%+1"),
        )
        .expect("admitted as literal text");
        assert_eq!(target.url(), "http://localhost:4402/posts?q=%+1");
    }

    #[test]
    fn the_cookie_is_host_prefixed_and_hardened() {
        let set = set_cookie("v");
        assert!(set.starts_with("__Host-rv2_oauth_pending=v;"), "{set}");
        assert!(!set.contains("Domain="), "{set}");
        let max_age = format!("Max-Age={PENDING_TTL_SECS}");
        for attribute in ["Path=/", "HttpOnly", "Secure", "SameSite=Lax", &max_age] {
            assert!(set.contains(attribute), "{set}");
        }
        assert!(!set.contains("Path=/auth"), "{set}");
    }

    #[test]
    fn the_pending_cookie_is_found_among_others() {
        let header = "theme=dark; __Host-rv2_oauth_pending=abc.def; other=1";
        assert_eq!(pending_cookie(header), Ok("abc.def"));
    }

    /// The unprefixed name a sibling domain could set is not read at all.
    #[test]
    fn the_unprefixed_name_is_not_read() {
        assert_eq!(
            pending_cookie("rv2_oauth_pending=abc.def"),
            Err(Refusal::Missing)
        );
    }

    #[test]
    fn an_empty_pending_cookie_is_missing() {
        assert_eq!(
            pending_cookie("__Host-rv2_oauth_pending="),
            Err(Refusal::Missing)
        );
    }

    /// The login-CSRF shape: the attacker's valid cookie sits in front of the real one.
    #[test]
    fn a_second_pending_cookie_is_refused_wherever_it_sits() {
        assert_eq!(
            pending_cookie("__Host-rv2_oauth_pending=a.b; __Host-rv2_oauth_pending=c.d"),
            Err(Refusal::DuplicateCookie)
        );
        assert_eq!(
            pending_cookie("__Host-rv2_oauth_pending=c.d; __Host-rv2_oauth_pending="),
            Err(Refusal::DuplicateCookie)
        );
    }

    #[test]
    fn a_path_over_the_length_cap_is_refused() {
        let long = format!("/{}", "a".repeat(MAX_PATH_LEN));
        assert_eq!(
            admit(&allowlist(), Some("http://localhost:4402"), Some(&long)),
            Err(Refusal::NotSameOriginPath)
        );
    }

    #[test]
    fn a_backslash_later_in_the_path_or_a_nul_byte_is_refused() {
        for path in ["/posts\\x", "/a/b\\c", "/a\0b"] {
            assert_eq!(
                admit(&allowlist(), Some("http://localhost:4402"), Some(path)),
                Err(Refusal::NotSameOriginPath),
                "admitted {path:?}"
            );
        }
    }
}
