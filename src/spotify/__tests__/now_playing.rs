use librespot::{
    core::spotify_uri::SpotifyUri,
    metadata::{
        audio::{AudioItem, UniqueFields, item::CoverImage},
        image::ImageSize,
    },
    protocol::metadata::artist_with_role::ArtistRole,
};

use crate::domain::TrackId;

use super::{CoverChoice, choose_cover, project_audio_item};

fn uri(value: &str) -> SpotifyUri {
    SpotifyUri::from_uri(value).expect("valid Spotify URI")
}

fn cover(url: &str, edge: u32) -> CoverChoice {
    CoverChoice {
        url: url.into(),
        width: Some(edge),
        height: Some(edge),
    }
}

#[test]
fn chooses_the_smallest_cover_that_covers_the_display_edge() {
    let covers = [
        cover("https://i.scdn.co/image/large", 640),
        cover("https://i.scdn.co/image/medium", 300),
        cover("https://i.scdn.co/image/small", 64),
    ];

    let chosen = choose_cover(&covers, 128).expect("cover");

    assert_eq!(chosen.url, "https://i.scdn.co/image/medium");
    assert_eq!(chosen.width, Some(300));
}

#[test]
fn falls_back_to_the_largest_cover_when_none_reach_the_display_edge() {
    let covers = [
        cover("https://i.scdn.co/image/tiny", 32),
        cover("https://i.scdn.co/image/small", 64),
    ];

    let chosen = choose_cover(&covers, 128).expect("cover");

    assert_eq!(chosen.url, "https://i.scdn.co/image/small");
}

#[test]
fn projects_track_metadata_and_keeps_only_the_selected_image() {
    let item = AudioItem {
        track_id: uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh"),
        uri: "spotify:track:4iV5W9uYEdYUVa79Axb7Rh".into(),
        files: librespot::metadata::audio::AudioFiles::default(),
        name: "Night Owl".into(),
        covers: vec![
            CoverImage {
                url: "https://i.scdn.co/image/large".into(),
                size: ImageSize::DEFAULT,
                width: 640,
                height: 640,
            },
            CoverImage {
                url: "https://i.scdn.co/image/near".into(),
                size: ImageSize::SMALL,
                width: 160,
                height: 160,
            },
        ],
        language: Vec::new(),
        duration_ms: 180_000,
        is_explicit: false,
        availability: Ok(()),
        alternatives: None,
        unique_fields: UniqueFields::Track {
            artists: librespot::metadata::artist::ArtistsWithRole(vec![
                librespot::metadata::artist::ArtistWithRole {
                    id: uri("spotify:artist:0TnOYISbd1XYRBk9myaseg"),
                    name: "Ada".into(),
                    role: ArtistRole::ARTIST_ROLE_MAIN_ARTIST,
                },
                librespot::metadata::artist::ArtistWithRole {
                    id: uri("spotify:artist:1uNFoZAHBGtllmzznpCI3s"),
                    name: "Composer".into(),
                    role: ArtistRole::ARTIST_ROLE_COMPOSER,
                },
            ]),
            album: "Hours".into(),
            album_artists: Vec::new(),
            popularity: 1,
            number: 1,
            disc_number: 1,
        },
    };

    let summary = project_audio_item(&item);

    assert_eq!(summary.id, Some(TrackId::new("4iV5W9uYEdYUVa79Axb7Rh")));
    assert_eq!(summary.name, "Night Owl");
    assert_eq!(summary.artist_names, vec!["Ada".to_owned()]);
    assert_eq!(summary.album_name, "Hours");
    assert_eq!(summary.duration_ms, 180_000);
    assert_eq!(
        summary.image.expect("image").url,
        "https://i.scdn.co/image/near"
    );
    assert!(summary.is_playable);
}
