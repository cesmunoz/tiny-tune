use gpui::{Context, Role, div, prelude::*, px, rgb, svg};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    state::{AppState, ViewState},
    theme::{ACCENT, ACCENT_HOVER, BORDER, MUTED, ON_ACCENT, TEXT},
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

    const fn icon(self) -> &'static [u8] {
        match self {
            Self::Previous => include_bytes!("../../assets/icons/skip-back.svg"),
            Self::TogglePlayPause { playing: true } => {
                include_bytes!("../../assets/icons/pause.svg")
            }
            Self::TogglePlayPause { playing: false } => {
                include_bytes!("../../assets/icons/play.svg")
            }
            Self::Next => include_bytes!("../../assets/icons/skip-forward.svg"),
            Self::VolumeDown => include_bytes!("../../assets/icons/volume-1.svg"),
            Self::VolumeUp => include_bytes!("../../assets/icons/volume-2.svg"),
            Self::Logout => include_bytes!("../../assets/icons/log-out.svg"),
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

pub(super) fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let actions = transport_actions(app.ui.view == ViewState::Playing);

    let volume = app.ui.volume.map(|percent| volume(percent, cx));

    div()
        .flex()
        .items_center()
        .gap_6()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .children(actions.into_iter().map(|action| action_button(action, cx))),
        )
        .children(volume)
}

fn volume(percent: u8, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let filled = VOLUME_BAR_PX * f32::from(percent) / 100.;

    div()
        .id("volume")
        .flex()
        .items_center()
        .gap_1()
        .child(action_button(PlayerAction::VolumeDown, cx))
        .child(
            div()
                .w(px(VOLUME_BAR_PX))
                .h(px(4.))
                .rounded_full()
                .bg(rgb(BORDER))
                .child(div().w(px(filled)).h_full().rounded_full().bg(rgb(ACCENT))),
        )
        .child(action_button(PlayerAction::VolumeUp, cx))
        .child(
            div()
                .w(px(36.))
                .text_xs()
                .text_color(rgb(MUTED))
                .child(format!("{percent}%")),
        )
}

pub(super) fn logout(cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    action_button(PlayerAction::Logout, cx)
}

fn action_button(action: PlayerAction, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let primary = action.is_primary();
    let quiet = action.is_quiet();
    let icon_color = if primary { ON_ACCENT } else { TEXT };

    div()
        .id(action.id())
        .focusable()
        .tab_stop(true)
        .role(Role::Button)
        .aria_label(action.label())
        .rounded_full()
        .cursor_pointer()
        .when(primary, |button| {
            button
                .px_5()
                .py_3()
                .bg(rgb(ACCENT))
                .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        })
        .when(quiet, |button| {
            button.p_2().hover(|style| style.bg(rgb(BORDER)))
        })
        .when(!primary && !quiet, |button| {
            button
                .px_4()
                .py_2()
                .border_1()
                .border_color(rgb(BORDER))
                .hover(|style| style.bg(rgb(BORDER)))
        })
        .on_click(cx.listener(move |app, _, _, cx| {
            if let Some(command) = action.command(&app.ui) {
                app.send(command, cx);
            }
        }))
        .child(
            svg()
                .data(action.icon())
                .size_5()
                .text_color(rgb(icon_color)),
        )
}

#[cfg(test)]
#[path = "__tests__/player_controls.rs"]
mod tests;
