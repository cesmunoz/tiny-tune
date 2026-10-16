use super::{
    BROKEN_GLYPH_WIDTH, BROKEN_WINDOW_HEIGHT, BROKEN_WINDOW_WIDTH, LIBRARY_COLUMN_WIDTH,
    MAIN_PADDING_X, PLAYER_REFERENCE_HEIGHT, PLAYER_REFERENCE_WIDTH, WRAPPED_TITLE, shell_size,
};

#[test]
fn shell_uses_the_player_screenshot_and_keeps_the_broken_window_from_returning() {
    let shell = shell_size();
    let title_width = BROKEN_GLYPH_WIDTH * f32::from(u16::try_from(WRAPPED_TITLE.len()).unwrap());

    assert_eq!(WRAPPED_TITLE, "Your music is playing");
    assert_eq!(shell.width, PLAYER_REFERENCE_WIDTH);
    assert_eq!(shell.height, PLAYER_REFERENCE_HEIGHT);
    assert_eq!(shell.sidebar_width, LIBRARY_COLUMN_WIDTH);
    assert_eq!(shell.min_height, BROKEN_WINDOW_HEIGHT);
    assert_eq!(
        shell.min_width,
        LIBRARY_COLUMN_WIDTH + title_width + (MAIN_PADDING_X * 2.)
    );
    assert!(shell.min_width > BROKEN_WINDOW_WIDTH);
    assert!(shell.width > shell.min_width);
    assert!(shell.height > shell.min_height);
}
