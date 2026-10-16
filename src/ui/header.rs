use gpui::{div, prelude::*, rgb};

use super::theme::ACCENT;

pub fn render(trailing: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
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
        .child(trailing)
}
