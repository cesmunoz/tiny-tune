use rune::{Cursor, El, row};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    state::ViewState,
    theme::{ACCENT, ACCENT_HOVER, BOLD, ON_ACCENT, ROUND, TEXT_BASE, label, rgb},
};

const fn is_busy(view: ViewState) -> bool {
    matches!(
        view,
        ViewState::RestoringSession | ViewState::Authenticating | ViewState::Connecting
    )
}

pub fn render(app: &TinyTune) -> El {
    let busy = is_busy(app.ui.view);
    let caption = if busy {
        "Waiting for Spotify…"
    } else {
        "Connect & play liked songs"
    };

    row()
        .key("connect")
        .role("button")
        .label(caption)
        .focusable(true)
        .self_start()
        .px(24.)
        .py(12.)
        .radius(ROUND)
        .fill(rgb(ACCENT))
        .hover(rgb(ACCENT_HOVER))
        .opacity(if busy { 0.5 } else { 1. })
        .cursor(Cursor::Pointer)
        .on_press(|app: &mut TinyTune| {
            if !is_busy(app.ui.view) {
                app.send(SpotifyCommand::LoginAndPlayLikedSongs);
            }
        })
        .child(label(caption, TEXT_BASE, BOLD, ON_ACCENT))
}
