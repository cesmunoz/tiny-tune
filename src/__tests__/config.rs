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
