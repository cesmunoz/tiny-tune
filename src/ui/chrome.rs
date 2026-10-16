/// Window measurements taken from the issue screenshots.
///
/// Issue 9 is the broken window: 123×851, with “Your music is playing”
/// stacked one glyph per line. The widest of those glyphs is 15px.
/// Issue 16’s player screenshot is 1919×1042, and its library column
/// ends at x=428, just before the gutter.
pub const BROKEN_WINDOW_WIDTH: f32 = 123.;
pub const BROKEN_WINDOW_HEIGHT: f32 = 851.;
pub const BROKEN_GLYPH_WIDTH: f32 = 15.;
pub const WRAPPED_TITLE: &str = "Your music is playing";
pub const PLAYER_REFERENCE_WIDTH: f32 = 1919.;
pub const PLAYER_REFERENCE_HEIGHT: f32 = 1042.;
pub const LIBRARY_COLUMN_WIDTH: f32 = 428.;
/// Horizontal inset of the main column. Kept here so the minimum width
/// includes the same padding the shell draws.
pub const MAIN_PADDING_X: f32 = 24.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellSize {
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub sidebar_width: f32,
}

#[must_use]
pub fn shell_size() -> ShellSize {
    let title_width = BROKEN_GLYPH_WIDTH * f32::from(title_chars());
    let min_width = LIBRARY_COLUMN_WIDTH + title_width + (MAIN_PADDING_X * 2.);
    assert!(
        min_width > BROKEN_WINDOW_WIDTH,
        "minimum width must be wider than the broken window"
    );
    ShellSize {
        width: PLAYER_REFERENCE_WIDTH,
        height: PLAYER_REFERENCE_HEIGHT,
        min_width,
        min_height: BROKEN_WINDOW_HEIGHT,
        sidebar_width: LIBRARY_COLUMN_WIDTH,
    }
}

fn title_chars() -> u16 {
    u16::try_from(WRAPPED_TITLE.len()).unwrap_or(0)
}

#[cfg(test)]
#[path = "__tests__/chrome.rs"]
mod tests;
