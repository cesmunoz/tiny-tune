use super::PlayerAction;
use crate::spotify::SpotifyCommand;

#[test]
fn actions_define_stable_ui_metadata_and_commands() {
    let cases = [
        (
            PlayerAction::Previous,
            "previous",
            "Previous",
            SpotifyCommand::Previous,
        ),
        (
            PlayerAction::TogglePlayPause { playing: false },
            "play-pause",
            "Play",
            SpotifyCommand::TogglePlayPause,
        ),
        (
            PlayerAction::TogglePlayPause { playing: true },
            "play-pause",
            "Pause",
            SpotifyCommand::TogglePlayPause,
        ),
        (PlayerAction::Next, "next", "Next", SpotifyCommand::Next),
        (
            PlayerAction::Logout,
            "logout",
            "Log out",
            SpotifyCommand::Logout,
        ),
    ];

    for (action, id, label, command) in cases {
        assert_eq!(action.id(), id);
        assert_eq!(action.label(), label);
        assert_eq!(action.command(), command);
        assert!(action.icon().starts_with(b"<svg"));
    }
}

#[test]
fn play_pause_is_the_only_primary_action() {
    assert!(PlayerAction::TogglePlayPause { playing: false }.is_primary());
    assert!(PlayerAction::TogglePlayPause { playing: true }.is_primary());
    assert!(!PlayerAction::Previous.is_primary());
    assert!(!PlayerAction::Next.is_primary());
    assert!(!PlayerAction::Logout.is_primary());
}
