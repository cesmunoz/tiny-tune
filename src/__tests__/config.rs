use super::resolve_client_id;

#[test]
fn falls_back_for_missing_empty_or_placeholder_client_ids() {
    for configured in [None, Some(String::new()), Some("your-client-id".into())] {
        assert_eq!(
            resolve_client_id(configured, "librespot-default".into()),
            "librespot-default"
        );
    }
}

#[test]
fn preserves_an_explicit_client_id() {
    assert_eq!(
        resolve_client_id(Some("custom-client".into()), "librespot-default".into()),
        "custom-client"
    );
}

#[test]
fn image_cache_budget_falls_back_for_missing_or_invalid_values() {
    for configured in [None, Some(""), Some("   "), Some("lots"), Some("65")] {
        assert_eq!(super::image_cache_budget_mb(configured), 8);
    }
}

#[test]
fn access_point_port_falls_back_for_missing_or_invalid_values() {
    for configured in [
        None,
        Some(""),
        Some("   "),
        Some("port"),
        Some("0"),
        Some("65536"),
    ] {
        assert_eq!(
            super::resolve_ap_port(configured),
            super::ApPortSetting::Automatic
        );
    }
}

#[test]
fn access_point_port_accepts_an_explicit_port() {
    assert_eq!(
        super::resolve_ap_port(Some("443")),
        super::ApPortSetting::Fixed(443)
    );
    assert_eq!(
        super::resolve_ap_port(Some(" 4070 ")),
        super::ApPortSetting::Fixed(4070)
    );
}

#[test]
fn automatic_access_point_tries_the_default_port_then_443() {
    let attempts = super::ApPortAttempts::from_setting(super::ApPortSetting::Automatic);
    assert_eq!(attempts.as_slice(), &[None, Some(super::FALLBACK_AP_PORT)]);
}

#[test]
fn fixed_access_point_uses_only_the_configured_port() {
    let attempts = super::ApPortAttempts::from_setting(super::ApPortSetting::Fixed(443));
    assert_eq!(attempts.as_slice(), &[Some(443)]);
}

#[test]
fn image_cache_budget_accepts_zero_through_the_maximum() {
    assert_eq!(super::image_cache_budget_mb(Some("0")), 0);
    assert_eq!(super::image_cache_budget_mb(Some(" 16 ")), 16);
    assert_eq!(super::image_cache_budget_mb(Some("64")), 64);
    assert_eq!(
        super::AppConfig {
            image_cache_budget_mb: 8
        }
        .image_cache_budget_bytes(),
        8 * 1024 * 1024
    );
}
