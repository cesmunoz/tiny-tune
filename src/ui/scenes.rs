//! Named states for Rune's headless tooling (`--snap`, `--tree`, `--script`).
//!
//! They feed fictional data through the normal reducer, so snapshots show
//! the real views without Spotify. Run them with, for example:
//! `cargo run -- --snap player.png --scene player`.

use crate::{
    domain::{NowPlaying, Page, PlaybackState, PlaylistSummary, TrackId, TrackSummary},
    spotify::SpotifyEvent,
};

use super::app::TinyTune;

type Scene = (&'static str, fn(&mut TinyTune));

pub const SCENES: &[Scene] = &[("library", library), ("player", player)];

fn library(app: &mut TinyTune) {
    let ui = &mut app.ui;
    ui.apply(SpotifyEvent::Ready);
    ui.apply(SpotifyEvent::PlaylistsLoaded(
        (1..=12)
            .map(|number| PlaylistSummary {
                name: format!("Sample playlist {number}"),
                uri: format!("spotify:playlist:sample{number:016}"),
            })
            .collect(),
    ));
    ui.apply(SpotifyEvent::VolumeChanged { percent: 60 });
}

fn player(app: &mut TinyTune) {
    library(app);
    let ui = &mut app.ui;
    let Some(playlist) = ui.playlists.first().cloned() else {
        return;
    };
    let _ = ui.open_playlist(playlist.clone());
    let generation = ui
        .selected_playlist
        .as_ref()
        .map_or(0, |detail| detail.generation);
    let tracks: Vec<TrackSummary> = (1..=40).map(sample_track).collect();
    ui.apply(SpotifyEvent::PlaylistTracksLoaded {
        playlist_uri: playlist.uri,
        generation,
        request_cursor: None,
        tracks: Page {
            items: tracks.clone(),
            next: None,
        },
    });
    ui.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Playing {
        now_playing: NowPlaying {
            track: tracks[0].clone(),
            position_ms: 42_000,
        },
    }));
}

fn sample_track(number: u32) -> TrackSummary {
    let id = format!("sample{number:016}");
    TrackSummary {
        spotify_uri: Some(format!("spotify:track:{id}")),
        id: Some(TrackId::new(&id)),
        name: format!("Sample track {number}"),
        artist_names: vec!["Sample artist".into()],
        album_name: "Sample album".into(),
        duration_ms: 180_000 + number * 1_000,
        image: None,
        is_playable: !number.is_multiple_of(7),
    }
}
