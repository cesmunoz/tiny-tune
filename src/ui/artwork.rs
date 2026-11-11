use rune::Pixels;

use crate::spotify::ArtworkBytes;

pub(super) fn render_artwork(artwork: ArtworkBytes) -> Option<Pixels> {
    let expected = usize::try_from(artwork.width)
        .ok()?
        .checked_mul(usize::try_from(artwork.height).ok()?)?
        .checked_mul(4)?;
    if artwork.width == 0 || artwork.height == 0 || artwork.rgba.len() != expected {
        return None;
    }

    let pixels = Pixels::new();
    pixels.set(artwork.width, artwork.height, artwork.rgba);
    Some(pixels)
}
