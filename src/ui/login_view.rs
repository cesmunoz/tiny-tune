use gpui::{Context, div, prelude::*, rgb};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    state::ViewState,
    theme::{ACCENT, ACCENT_HOVER, ON_ACCENT},
};

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let is_busy = matches!(
        app.ui.view,
        ViewState::RestoringSession | ViewState::Authenticating | ViewState::Connecting
    );

    div()
        .id("connect")
        .focusable()
        .tab_stop(true)
        .px_6()
        .py_3()
        .rounded_full()
        .bg(rgb(ACCENT))
        .text_color(rgb(ON_ACCENT))
        .font_weight(gpui::FontWeight::BOLD)
        .cursor_pointer()
        .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        .when(is_busy, |button| button.opacity(0.5))
        .on_click(cx.listener(|app, _, _, cx| {
            if !matches!(
                app.ui.view,
                ViewState::RestoringSession | ViewState::Authenticating | ViewState::Connecting
            ) {
                app.send(SpotifyCommand::LoginAndPlayLikedSongs, cx);
            }
        }))
        .child(if is_busy {
            "Waiting for Spotify…"
        } else {
            "Connect & play liked songs"
        })
}
