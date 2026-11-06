use rune::{El, Pixels, SurfaceFit, col, el, row};

use super::{
    app::TinyTune,
    theme::{BORDER, MUTED, REGULAR, ROUND, SEMIBOLD, TEXT, TEXT_SM, TEXT_XS, label, rgb},
};

const COVER_PX: f32 = 72.;

pub fn render(app: &TinyTune) -> El {
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

    let details = col()
        .gap(4.)
        .min_w(0.)
        .shrink()
        .child(label(title, TEXT_SM, SEMIBOLD, TEXT).truncate())
        .children(artists.map(|artists| label(artists, TEXT_XS, REGULAR, MUTED).truncate()));

    row()
        .key("now-playing")
        .grow(1.)
        .min_w(0.)
        .items_center()
        .gap(12.)
        .child(cover(app.displayed_artwork()))
        .child(details)
}

fn cover(artwork: Option<&Pixels>) -> El {
    let cover = el().key("cover").size(COVER_PX, COVER_PX).radius(8.);
    match artwork {
        Some(artwork) => cover.pixels_fit(artwork, SurfaceFit::Fill),
        None => cover
            .fill(rgb(BORDER))
            .center()
            .child(el().size(16., 16.).radius(ROUND).fill(rgb(MUTED))),
    }
}
