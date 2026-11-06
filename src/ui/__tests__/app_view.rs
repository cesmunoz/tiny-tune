use std::time::Instant;

use rune::{Env, Runtime};

use super::view;
use crate::ui::{SCENES, TinyTune};

const VIEWPORT: (f32, f32) = (1280., 860.);

fn runtime(scene: Option<&str>) -> Runtime<TinyTune> {
    let mut app = TinyTune::new(Instant::now());
    if let Some(scene) = scene {
        let (_, apply) = SCENES
            .iter()
            .find(|(name, _)| *name == scene)
            .expect("scene exists");
        apply(&mut app);
    }
    let env = Env {
        system_dark: true,
        scale: 2.,
    };
    let mut rt = Runtime::new(app, view, VIEWPORT, env);
    rt.frame(0.);
    rt
}

fn click(rt: &mut Runtime<TinyTune>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("{key} is on screen"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1. / 60.);
}

#[test]
fn signed_out_window_shows_the_connect_button() {
    let rt = runtime(None);
    assert!(rt.find("connect").is_some());
    assert!(rt.find("player-bar").is_none());

    let rt = runtime(Some("library"));
    assert!(rt.find("connect").is_none());
    assert!(rt.find("player-bar").is_some());
}

#[test]
fn pressing_a_playlist_opens_its_tracks() {
    let mut rt = runtime(Some("library"));
    assert!(rt.state.ui.selected_playlist.is_none());

    click(&mut rt, "playlist-1/open");

    let selected = rt.state.ui.selected_playlist.as_ref().expect("opened");
    assert_eq!(selected.summary.name, "Sample playlist 2");
    assert!(rt.find("playlist-tracks").is_some());
}

#[test]
fn the_play_pill_does_not_also_open_its_playlist() {
    let mut rt = runtime(Some("library"));

    click(&mut rt, "playlist-0/open/play");

    assert!(rt.state.ui.selected_playlist.is_none());
}

#[test]
fn player_scene_virtualizes_track_rows() {
    let rt = runtime(Some("player"));

    assert!(rt.find("track-0").is_some());
    assert!(
        rt.find("track-39").is_none(),
        "rows far below the viewport are not mounted"
    );
    assert!(rt.find("play-pause").is_some());
    assert!(rt.find("volume").is_some());
}
