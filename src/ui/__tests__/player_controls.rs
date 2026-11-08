use super::PlayerAction;
use crate::{
    spotify::{SpotifyCommand, SpotifyEvent},
    ui::state::AppState,
};

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
        assert_eq!(action.command(&AppState::default()), Some(command));
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
    assert!(!PlayerAction::VolumeDown.is_primary());
    assert!(!PlayerAction::VolumeUp.is_primary());
}

#[test]
fn volume_actions_step_the_reported_volume() {
    let mut ui = AppState::default();
    assert_eq!(PlayerAction::VolumeUp.command(&ui), None);

    ui.apply(SpotifyEvent::VolumeChanged { percent: 50 });
    assert_eq!(
        PlayerAction::VolumeUp.command(&ui),
        Some(SpotifyCommand::SetVolume { percent: 60 })
    );
    assert_eq!(
        PlayerAction::VolumeDown.command(&ui),
        Some(SpotifyCommand::SetVolume { percent: 40 })
    );

    for (action, id, label) in [
        (PlayerAction::VolumeDown, "volume-down", "Volume down"),
        (PlayerAction::VolumeUp, "volume-up", "Volume up"),
    ] {
        assert_eq!(action.id(), id);
        assert_eq!(action.label(), label);
        assert!(action.is_quiet());
        assert!(action.icon().starts_with(b"<svg"));
    }
}

#[test]
fn transport_stays_in_the_player_bar_and_logout_moves_to_the_shell() {
    assert_eq!(
        super::transport_actions(false).map(PlayerAction::id),
        ["previous", "play-pause", "next"]
    );
    assert_eq!(
        super::transport_actions(true).map(PlayerAction::id),
        ["previous", "play-pause", "next"]
    );
    assert!(
        super::transport_actions(false)
            .into_iter()
            .all(|action| action != PlayerAction::Logout)
    );
    assert_eq!(super::shell_logout(false), None);
    assert_eq!(super::shell_logout(true), Some(PlayerAction::Logout));
}
