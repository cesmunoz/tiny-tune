use gpui::{Context, div, prelude::*, rgb};

use super::{
    app::TinyTune,
    state::ConnectButton,
    theme::{ACCENT, ACCENT_HOVER, ON_ACCENT},
};

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let button = app.ui.connect_button();
    let is_busy = button == ConnectButton::Waiting;

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
            if let Some(command) = app.ui.connect_button().command() {
                app.send(command, cx);
            }
        }))
        .child(button.label())
}
