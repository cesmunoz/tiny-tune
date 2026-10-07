use gpui::{Context, Role, div, prelude::*, rgb, svg};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    state::ViewState,
    theme::{ACCENT, ACCENT_HOVER, BORDER, ON_ACCENT, TEXT},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayerAction {
    Previous,
    TogglePlayPause { playing: bool },
    Next,
    Logout,
}

impl PlayerAction {
    const fn id(self) -> &'static str {
        match self {
            Self::Previous => "previous",
            Self::TogglePlayPause { .. } => "play-pause",
            Self::Next => "next",
            Self::Logout => "logout",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Previous => "Previous",
            Self::TogglePlayPause { playing: true } => "Pause",
            Self::TogglePlayPause { playing: false } => "Play",
            Self::Next => "Next",
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
            Self::Logout => include_bytes!("../../assets/icons/log-out.svg"),
        }
    }

    const fn command(self) -> SpotifyCommand {
        match self {
            Self::Previous => SpotifyCommand::Previous,
            Self::TogglePlayPause { .. } => SpotifyCommand::TogglePlayPause,
            Self::Next => SpotifyCommand::Next,
            Self::Logout => SpotifyCommand::Logout,
        }
    }

    const fn is_primary(self) -> bool {
        matches!(self, Self::TogglePlayPause { .. })
    }
}

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let actions = [
        PlayerAction::Previous,
        PlayerAction::TogglePlayPause {
            playing: app.ui.view == ViewState::Playing,
        },
        PlayerAction::Next,
        PlayerAction::Logout,
    ];

    div()
        .flex()
        .items_center()
        .gap_3()
        .children(actions.into_iter().map(|action| {
            let command = action.command();
            let primary = action.is_primary();
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
                .when(!primary, |button| {
                    button
                        .px_4()
                        .py_2()
                        .border_1()
                        .border_color(rgb(BORDER))
                        .hover(|style| style.bg(rgb(BORDER)))
                })
                .on_click(cx.listener(move |app, _, _, cx| {
                    app.send(command.clone(), cx);
                }))
                .child(
                    svg()
                        .data(action.icon())
                        .size_5()
                        .text_color(rgb(icon_color)),
                )
        }))
}

#[cfg(test)]
#[path = "__tests__/player_controls.rs"]
mod tests;
