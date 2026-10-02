use gpui::{Context, div, prelude::*, rgb};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    state::ViewState,
    theme::{ACCENT, ACCENT_HOVER, BORDER, ON_ACCENT},
};

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .id("previous")
                .focusable()
                .tab_stop(true)
                .px_4()
                .py_2()
                .rounded_full()
                .border_1()
                .border_color(rgb(BORDER))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(BORDER)))
                .on_click(cx.listener(|app, _, _, cx| {
                    app.send(SpotifyCommand::Previous, cx);
                }))
                .child("Previous"),
        )
        .child(
            div()
                .id("play-pause")
                .focusable()
                .tab_stop(true)
                .px_5()
                .py_3()
                .rounded_full()
                .bg(rgb(ACCENT))
                .text_color(rgb(ON_ACCENT))
                .font_weight(gpui::FontWeight::BOLD)
                .cursor_pointer()
                .hover(|style| style.bg(rgb(ACCENT_HOVER)))
                .on_click(cx.listener(|app, _, _, cx| {
                    app.send(SpotifyCommand::TogglePlayPause, cx);
                }))
                .child(if app.ui.view == ViewState::Playing {
                    "Pause"
                } else {
                    "Play"
                }),
        )
        .child(
            div()
                .id("next")
                .focusable()
                .tab_stop(true)
                .px_4()
                .py_2()
                .rounded_full()
                .border_1()
                .border_color(rgb(BORDER))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(BORDER)))
                .on_click(cx.listener(|app, _, _, cx| {
                    app.send(SpotifyCommand::Next, cx);
                }))
                .child("Next"),
        )
}
