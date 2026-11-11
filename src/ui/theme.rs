use rune::{Color, El, TextStyle, color::srgb_decode, el, text};

pub const BACKGROUND: u32 = 0x10_13_11;
pub const SURFACE: u32 = 0x18_1d_1a;
pub const BORDER: u32 = 0x29_32_2c;
pub const TEXT: u32 = 0xf2_f5_f3;
pub const MUTED: u32 = 0x9b_a8_a0;
pub const ACCENT: u32 = 0x1e_d7_60;
pub const ACCENT_HOVER: u32 = 0x39_e4_77;
pub const ERROR: u32 = 0xff_7a_7a;
pub const ON_ACCENT: u32 = 0x07_13_0b;

/// Type scale, in points.
pub const TEXT_XS: f32 = 12.;
pub const TEXT_SM: f32 = 14.;
pub const TEXT_BASE: f32 = 16.;
pub const TEXT_XL: f32 = 20.;
pub const TEXT_2XL: f32 = 24.;

pub const REGULAR: f32 = 400.;
pub const SEMIBOLD: f32 = 600.;
pub const BOLD: f32 = 700.;

/// Fully rounded corners for pills and dots.
pub const ROUND: f32 = 9999.;

/// Converts a `0xRRGGBB` sRGB colour to Rune's linear colour.
#[must_use]
pub fn rgb(hex: u32) -> Color {
    let channel = |shift: u32| {
        let byte = u8::try_from((hex >> shift) & 0xff).unwrap_or(u8::MAX);
        srgb_decode(f32::from(byte) / 255.)
    };
    Color {
        r: channel(16),
        g: channel(8),
        b: channel(0),
        a: 1.,
    }
}

/// A single line of text in the given size, weight and colour.
#[must_use]
pub fn label(content: impl Into<String>, size: f32, weight: f32, color: u32) -> El {
    text(content)
        .font(TextStyle::new(size, weight))
        .color(rgb(color))
}

/// A 1 pt horizontal rule. Rune borders draw on every side, so single-side
/// borders are drawn as dividers.
#[must_use]
pub fn h_divider() -> El {
    el().w_full().h(1.).fill(rgb(BORDER))
}

/// A 1 pt vertical rule.
#[must_use]
pub fn v_divider() -> El {
    el().w(1.).h_full().fill(rgb(BORDER))
}
