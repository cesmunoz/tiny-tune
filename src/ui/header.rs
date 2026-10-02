use gpui::{div, prelude::*, rgb};

use super::theme::{ACCENT, MUTED};

pub fn render() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().size_3().rounded_full().bg(rgb(ACCENT)))
                .child(
                    div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("Tiny Tune"),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("RUST · GPUI · NO ELECTRON"),
        )
}
