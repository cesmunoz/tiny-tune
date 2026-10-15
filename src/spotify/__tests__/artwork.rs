use std::io::Cursor;

use image::{Rgba, RgbaImage};

use super::{allowed_artwork_url, append_limited, decode_for_display};

#[test]
fn accepts_only_spotify_image_cdn_urls() {
    let allowed = reqwest::Url::parse("https://i.scdn.co/image/ab67616d00001e02").unwrap();
    let http = reqwest::Url::parse("http://i.scdn.co/image/ab67616d00001e02").unwrap();
    let other = reqwest::Url::parse("https://example.com/image/ab67616d00001e02").unwrap();
    let path = reqwest::Url::parse("https://i.scdn.co/track/ab67616d00001e02").unwrap();

    assert!(allowed_artwork_url(&allowed));
    assert!(!allowed_artwork_url(&http));
    assert!(!allowed_artwork_url(&other));
    assert!(!allowed_artwork_url(&path));
}

#[test]
fn stops_appending_once_the_byte_limit_is_reached() {
    let mut body = vec![1, 2, 3];

    assert!(!append_limited(&mut body, &[4, 5], 4));
    assert_eq!(body, vec![1, 2, 3]);
    assert!(append_limited(&mut body, &[4], 4));
    assert_eq!(body, vec![1, 2, 3, 4]);
}

#[test]
fn decodes_near_display_size_and_swaps_to_bgra() {
    let image = RgbaImage::from_pixel(300, 180, Rgba([10, 20, 30, 255]));
    let mut encoded = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png)
        .unwrap();

    let decoded = decode_for_display(&encoded, 128).unwrap();

    assert!(decoded.width <= 128);
    assert!(decoded.height <= 128);
    assert_eq!(decoded.width, 128);
    assert_eq!(
        decoded.bgra.len(),
        usize::try_from(decoded.width).unwrap() * usize::try_from(decoded.height).unwrap() * 4
    );
    assert_eq!(&decoded.bgra[..4], &[30, 20, 10, 255]);
}
