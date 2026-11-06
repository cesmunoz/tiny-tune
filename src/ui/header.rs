use rune::{El, el, row};

use super::theme::{ACCENT, BOLD, ROUND, TEXT, TEXT_XL, label, rgb};

pub fn render(trailing: Option<El>) -> El {
    row()
        .w_full()
        .items_center()
        .justify_between()
        .gap(12.)
        .child(
            row()
                .items_center()
                .gap(12.)
                .child(el().size(12., 12.).radius(ROUND).fill(rgb(ACCENT)))
                .child(label("Tiny Tune", TEXT_XL, BOLD, TEXT)),
        )
        .children(trailing)
}
