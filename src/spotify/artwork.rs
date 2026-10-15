use std::time::Duration;

use anyhow::{Result, anyhow};
use image::ImageReader;

use super::now_playing::ARTWORK_EDGE_PX;

pub(crate) const MAX_ARTWORK_BYTES: usize = 512 * 1024;
const ARTWORK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedArtwork {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

pub(crate) fn allowed_artwork_url(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("i.scdn.co")
        && url.path().starts_with("/image/")
        && url.username().is_empty()
        && url.password().is_none()
}

pub(crate) async fn load_display_artwork(
    client: &reqwest::Client,
    url: &str,
) -> Result<DecodedArtwork> {
    let parsed = reqwest::Url::parse(url)?;
    if !allowed_artwork_url(&parsed) {
        return Err(anyhow!("unsupported artwork url"));
    }

    let response = tokio::time::timeout(ARTWORK_TIMEOUT, client.get(parsed).send())
        .await
        .map_err(|_| anyhow!("artwork request timed out"))??
        .error_for_status()?;
    if !allowed_artwork_url(response.url()) {
        return Err(anyhow!("unsupported artwork url"));
    }

    let encoded = read_body_limited(response, MAX_ARTWORK_BYTES).await?;
    decode_for_display(&encoded, ARTWORK_EDGE_PX)
}

pub(crate) async fn read_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > u64::try_from(limit).unwrap_or(u64::MAX))
    {
        return Err(anyhow!("artwork response exceeds byte limit"));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if !append_limited(&mut body, &chunk, limit) {
            return Err(anyhow!("artwork response exceeds byte limit"));
        }
    }
    Ok(body)
}

pub(crate) fn pixel_len(width: u32, height: u32) -> Option<usize> {
    usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(4)
}

fn append_limited(body: &mut Vec<u8>, chunk: &[u8], limit: usize) -> bool {
    if body.len().saturating_add(chunk.len()) > limit {
        return false;
    }
    body.extend_from_slice(chunk);
    true
}

/// Decode `bytes` and keep only a display-sized BGRA buffer.
/// The encoded input is borrowed and is not retained.
pub(crate) fn decode_for_display(bytes: &[u8], edge: u32) -> Result<DecodedArtwork> {
    if edge == 0 {
        return Err(anyhow!("display edge must be positive"));
    }
    let image = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    let fitted = image.thumbnail(edge, edge);
    let rgba = fitted.into_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 || width > edge || height > edge {
        return Err(anyhow!("decoded artwork is outside the display size"));
    }
    let mut bgra = rgba.into_raw();
    for pixel in bgra.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let expected =
        pixel_len(width, height).ok_or_else(|| anyhow!("decoded artwork dimensions overflow"))?;
    if bgra.len() != expected {
        return Err(anyhow!("decoded artwork buffer has an unexpected length"));
    }
    Ok(DecodedArtwork {
        width,
        height,
        bgra,
    })
}

#[cfg(test)]
#[path = "__tests__/artwork.rs"]
mod tests;
