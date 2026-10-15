use std::sync::Arc;

use gpui::RenderImage;
use image::RgbaImage;
use smallvec::SmallVec;

use crate::spotify::ArtworkBytes;

pub(super) fn render_artwork(artwork: ArtworkBytes) -> Option<Arc<RenderImage>> {
    let expected = usize::try_from(artwork.width)
        .ok()?
        .checked_mul(usize::try_from(artwork.height).ok()?)?
        .checked_mul(4)?;
    if artwork.width == 0 || artwork.height == 0 || artwork.bgra.len() != expected {
        return None;
    }

    let buffer = RgbaImage::from_raw(artwork.width, artwork.height, artwork.bgra)?;
    let frame = image::Frame::from_parts(buffer, 0, 0, image::Delay::from_numer_denom_ms(0, 1));
    let frames: SmallVec<[image::Frame; 1]> = [frame].into();
    Some(Arc::new(RenderImage::new(frames)))
}
