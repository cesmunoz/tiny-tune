use rune::{Cursor, El, ScrollSpec, col, row};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    theme::{ACCENT, BORDER, REGULAR, ROUND, TEXT, TEXT_BASE, TEXT_XS, label, rgb},
};

pub fn render(app: &TinyTune) -> El {
    let selected_uri = app
        .ui
        .selected_playlist
        .as_ref()
        .map(|detail| detail.summary.uri.as_str());

    let rows = app
        .ui
        .playlists
        .iter()
        .enumerate()
        .map(|(index, playlist)| {
            let open_playlist = playlist.clone();
            let play_uri = playlist.uri.clone();
            let selected = selected_uri == Some(playlist.uri.as_str());
            // Rows use padding, not a stack gap, so the list could be virtualized.
            row()
                .key(format!("playlist-{index}"))
                .w_full()
                .pb(8.)
                .child(
                    row()
                        .key("open")
                        .role("button")
                        .label(playlist.name.clone())
                        .focusable(true)
                        .w_full()
                        .px(16.)
                        .py(12.)
                        .radius(8.)
                        .border(1., rgb(if selected { ACCENT } else { BORDER }))
                        .hover(rgb(BORDER))
                        .cursor(Cursor::Pointer)
                        .on_press(move |app: &mut TinyTune| {
                            let command = app.ui.open_playlist(open_playlist.clone());
                            app.send(command);
                        })
                        .items_center()
                        .justify_between()
                        .gap(12.)
                        .child(
                            label(playlist.name.clone(), TEXT_BASE, REGULAR, TEXT)
                                .truncate()
                                .min_w(0.)
                                .shrink(),
                        )
                        .child(
                            row()
                                .key("play")
                                .role("button")
                                .label("Play")
                                .focusable(true)
                                .px(12.)
                                .py(4.)
                                .radius(ROUND)
                                .border(1., rgb(BORDER))
                                .hover(rgb(BORDER))
                                .cursor(Cursor::Pointer)
                                .on_press(move |app: &mut TinyTune| {
                                    app.send(SpotifyCommand::PlayPlaylist {
                                        uri: play_uri.clone(),
                                        start_track_uri: None,
                                    });
                                })
                                .child(label("Play", TEXT_XS, REGULAR, TEXT)),
                        ),
                )
        });

    col()
        .key("playlists")
        .w_full()
        .grow(1.)
        .min_h(0.)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(col().w_full().children(rows))
}
