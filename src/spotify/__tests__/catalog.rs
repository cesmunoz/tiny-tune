use super::{PlaylistPageDto, PlaylistSummary};

#[test]
fn deserializes_and_projects_playlist_page() {
    let page: PlaylistPageDto =
        serde_json::from_str(r#"{"items":[{"name":"Focus","uri":"spotify:playlist:123"}]}"#)
            .expect("valid playlist fixture");

    let playlists = page
        .items
        .into_iter()
        .map(|item| PlaylistSummary {
            name: item.name,
            uri: item.uri,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        playlists,
        vec![PlaylistSummary {
            name: "Focus".into(),
            uri: "spotify:playlist:123".into(),
        }]
    );
}
