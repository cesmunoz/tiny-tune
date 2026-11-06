use rune::{Cursor, El, Icon, el, icon, row};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    icons,
    state::{AppState, ViewState},
    theme::{
        ACCENT, ACCENT_HOVER, BORDER, MUTED, ON_ACCENT, REGULAR, ROUND, TEXT, TEXT_XS, label, rgb,
    },
};

const VOLUME_STEP: i8 = 10;
const VOLUME_BAR_PX: f32 = 72.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerAction {
    Previous,
    TogglePlayPause { playing: bool },
    Next,
    VolumeDown,
    VolumeUp,
    Logout,
}

impl PlayerAction {
    const fn id(self) -> &'static str {
        match self {
            Self::Previous => "previous",
            Self::TogglePlayPause { .. } => "play-pause",
            Self::Next => "next",
            Self::VolumeDown => "volume-down",
            Self::VolumeUp => "volume-up",
            Self::Logout => "logout",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Previous => "Previous",
            Self::TogglePlayPause { playing: true } => "Pause",
            Self::TogglePlayPause { playing: false } => "Play",
            Self::Next => "Next",
            Self::VolumeDown => "Volume down",
            Self::VolumeUp => "Volume up",
            Self::Logout => "Log out",
        }
    }

    fn icon(self) -> &'static Icon {
        match self {
            Self::Previous => &icons::SKIP_BACK,
            Self::TogglePlayPause { playing: true } => &icons::PAUSE,
            Self::TogglePlayPause { playing: false } => &icons::PLAY,
            Self::Next => &icons::SKIP_FORWARD,
            Self::VolumeDown => &icons::VOLUME_1,
            Self::VolumeUp => &icons::VOLUME_2,
            Self::Logout => &icons::LOG_OUT,
        }
    }

    fn command(self, ui: &AppState) -> Option<SpotifyCommand> {
        match self {
            Self::Previous => Some(SpotifyCommand::Previous),
            Self::TogglePlayPause { .. } => Some(SpotifyCommand::TogglePlayPause),
            Self::Next => Some(SpotifyCommand::Next),
            Self::VolumeDown => ui.nudge_volume(-VOLUME_STEP),
            Self::VolumeUp => ui.nudge_volume(VOLUME_STEP),
            Self::Logout => Some(SpotifyCommand::Logout),
        }
    }

    const fn is_primary(self) -> bool {
        matches!(self, Self::TogglePlayPause { .. })
    }

    const fn is_quiet(self) -> bool {
        matches!(self, Self::VolumeDown | Self::VolumeUp)
    }
}

pub(super) const fn transport_actions(playing: bool) -> [PlayerAction; 3] {
    [
        PlayerAction::Previous,
        PlayerAction::TogglePlayPause { playing },
        PlayerAction::Next,
    ]
}

pub(super) const fn shell_logout(connected: bool) -> Option<PlayerAction> {
    if connected {
        Some(PlayerAction::Logout)
    } else {
        None
    }
}

pub(super) fn render(app: &TinyTune) -> El {
    let actions = transport_actions(app.ui.view == ViewState::Playing);

    row()
        .key("player-controls")
        .items_center()
        .gap(24.)
        .child(
            row()
                .items_center()
                .gap(12.)
                .children(actions.into_iter().map(action_button)),
        )
        .children(app.ui.volume.map(volume))
}

fn volume(percent: u8) -> El {
    let filled = VOLUME_BAR_PX * f32::from(percent) / 100.;

    row()
        .key("volume")
        .items_center()
        .gap(4.)
        .child(action_button(PlayerAction::VolumeDown))
        .child(
            row()
                .key("volume-bar")
                .role("progressbar")
                .label("Volume")
                .range_value(f64::from(percent), 0., 100.)
                .w(VOLUME_BAR_PX)
                .h(4.)
                .radius(ROUND)
                .fill(rgb(BORDER))
                .child(el().w(filled).h_full().radius(ROUND).fill(rgb(ACCENT))),
        )
        .child(action_button(PlayerAction::VolumeUp))
        .child(
            row()
                .w(36.)
                .child(label(format!("{percent}%"), TEXT_XS, REGULAR, MUTED)),
        )
}

pub(super) fn logout() -> El {
    action_button(PlayerAction::Logout)
}

fn action_button(action: PlayerAction) -> El {
    let primary = action.is_primary();
    let quiet = action.is_quiet();
    let icon_color = if primary { ON_ACCENT } else { TEXT };

    let button = row()
        .key(action.id())
        .role("button")
        .label(action.label())
        .focusable(true)
        .radius(ROUND)
        .cursor(Cursor::Pointer)
        .on_press(move |app: &mut TinyTune| {
            if let Some(command) = action.command(&app.ui) {
                app.send(command);
            }
        })
        .child(icon(action.icon(), 20.).color(rgb(icon_color)));

    if primary {
        button
            .px(20.)
            .py(12.)
            .fill(rgb(ACCENT))
            .hover(rgb(ACCENT_HOVER))
    } else if quiet {
        button.padding(8.).hover(rgb(BORDER))
    } else {
        button
            .px(16.)
            .py(8.)
            .border(1., rgb(BORDER))
            .hover(rgb(BORDER))
    }
}

#[cfg(test)]
#[path = "__tests__/player_controls.rs"]
mod tests;
