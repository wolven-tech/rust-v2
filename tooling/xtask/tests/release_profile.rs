//! Keep the copied release examples aligned with the low-idle-cost, private-
//! response-safe profile. Container smoke still proves actual Nginx behaviour.

const WEB_FLY: &str = include_str!("../../../deploy/fly.web.toml.example");
const APP_FLY: &str = include_str!("../../../deploy/fly.app.toml.example");
const WEB_NGINX: &str = include_str!("../../../deploy/nginx.static.conf");
const APP_NGINX: &str = include_str!("../../../deploy/nginx.spa.conf");

fn has_assignment(body: &str, key: &str, expected: &str) -> bool {
    body.lines().any(|line| {
        line.split_once('=')
            .is_some_and(|(name, value)| name.trim() == key && value.trim() == expected)
    })
}

#[test]
fn static_hosts_start_without_an_unjustified_warm_floor() {
    for (name, config) in [("web", WEB_FLY), ("app", APP_FLY)] {
        assert!(
            has_assignment(config, "auto_stop_machines", "\"suspend\""),
            "{name} must suspend when idle"
        );
        assert!(
            has_assignment(config, "auto_start_machines", "true"),
            "{name} must resume on demand"
        );
        assert!(
            has_assignment(config, "min_machines_running", "0"),
            "{name} must not force a warm Machine by default"
        );
    }
}

#[test]
fn public_html_is_shareable_but_private_and_failed_responses_are_not() {
    assert!(WEB_NGINX.contains("map \"$status:$uri\" $public_cache_control"));
    assert!(WEB_NGINX.contains("default \"private, no-store\""));
    assert!(WEB_NGINX.contains("~^200:/ \"public, max-age=0, s-maxage=86400"));
    assert!(WEB_NGINX.contains("~^200:/health$ \"private, no-store\""));
    assert!(WEB_NGINX.contains("add_header Cache-Control $public_cache_control always"));
    assert!(!WEB_NGINX.contains("expires 1y"));
}

#[test]
fn app_html_remains_private_while_fingerprinted_assets_can_be_public() {
    assert!(APP_NGINX.contains("map \"$status:$uri\" $app_cache_control"));
    assert!(APP_NGINX.contains("default \"private, no-store\""));
    assert!(APP_NGINX.contains("public, max-age=31536000, immutable"));
    assert!(APP_NGINX.contains("add_header Cache-Control $app_cache_control always"));
    assert!(!APP_NGINX.contains("expires 1y"));
}
