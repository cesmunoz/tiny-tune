use std::sync::Arc;

use gpui::{ObjectFit, RenderImage, StyledImage, div, img, prelude::*, px, rgb};

use super::{
    app::TinyTune,
    theme::{BORDER, MUTED, TEXT},
};

const COVER_PX: f32 = 72.;

pub fn render(app: &TinyTune) -> impl IntoElement {
    let now_playing = app.ui.now_playing();
    let title = now_playing.map_or("Nothing playing", |now_playing| {
        if now_playing.track.name.is_empty() {
            "Loading track"
        } else {
            now_playing.track.name.as_str()
        }
    });
    let artists = now_playing
        .map(|now_playing| now_playing.track.artist_names.join(", "))
        .filter(|artists| !artists.is_empty());

    div()
        .id("now-playing")
        .flex()
        .flex_1()
        .min_w(px(0.))
        .items_center()
        .gap_3()
        .child(cover(app.displayed_artwork()))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .min_w(px(0.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(TEXT))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(title.to_owned()),
                )
                .when_some(artists, |view, artists| {
                    view.child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(artists),
                    )
                }),
        )
}

fn cover(artwork: Option<Arc<RenderImage>>) -> impl IntoElement {
    let show_placeholder = artwork.is_none();

    div()
        .w(px(COVER_PX))
        .h(px(COVER_PX))
        .flex_shrink_0()
        .rounded_lg()
        .overflow_hidden()
        .when(show_placeholder, |cover| {
            cover
                .bg(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .child(div().size_4().rounded_full().bg(rgb(MUTED)))
        })
        .when_some(artwork, |cover, artwork| {
            cover.child(img(artwork).size_full().object_fit(ObjectFit::Cover))
        })
}
