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
