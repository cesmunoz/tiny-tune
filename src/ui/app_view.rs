use rune::{Cx, El, col, row};

use super::{
    app::TinyTune,
    chrome::{self, MAIN_PADDING_X},
    header, login_view, now_playing, player_controls, playlist_list, playlist_tracks,
    state::ViewState,
    theme::{
        BACKGROUND, ERROR, MUTED, REGULAR, SEMIBOLD, SURFACE, TEXT, TEXT_2XL, TEXT_SM, h_divider,
        label, rgb, v_divider,
    },
};

/// Rune view: rebuilt from state after every handler and backend event.
pub fn view(app: &TinyTune, _cx: &mut Cx<TinyTune>) -> El {
    let connected = app.connected();
    let logout = player_controls::shell_logout(connected).map(|_| player_controls::logout());

    let mut shell = col().w_full().h_full().fill(rgb(BACKGROUND));
    #[cfg(debug_assertions)]
    {
        shell = shell.child(dev_banner(app)).child(h_divider());
    }
    shell = shell.child(shell_header(logout)).child(h_divider()).child(
        row()
            .key("body")
            .w_full()
            .grow(1.)
            .min_h(0.)
            .child(library_column(app))
            .child(v_divider())
            .child(stage(app)),
    );
    if connected {
        shell = shell.child(h_divider()).child(player_bar(app));
    }
    shell
}

fn shell_header(logout: Option<El>) -> El {
    row()
        .w_full()
        .px(MAIN_PADDING_X)
        .py(16.)
        .child(header::render(logout))
}

fn library_column(app: &TinyTune) -> El {
    let column = col()
        .key("library")
        .w(chrome::shell_size().sidebar_width)
        .h_full()
        .min_h(0.)
        .gap(12.)
        .fill(rgb(SURFACE))
        .padding(16.)
        .child(label("Library", TEXT_SM, SEMIBOLD, TEXT));
    if app.ui.playlists.is_empty() {
        column.child(label(
            "Connect to see your playlists",
            TEXT_SM,
            REGULAR,
            MUTED,
        ))
    } else {
        column.child(playlist_list::render(app))
    }
}

fn stage(app: &TinyTune) -> El {
    let status_color = if app.ui.view == ViewState::Failed {
        ERROR
    } else {
        MUTED
    };
    let mut stage = col()
        .key("stage")
        .grow(1.)
        .min_w(0.)
        .h_full()
        .min_h(0.)
        .gap(16.)
        .px(MAIN_PADDING_X)
        .py(24.)
        .child(label(title_for(app.ui.view), TEXT_2XL, SEMIBOLD, TEXT).truncate())
        .child(label(app.ui.status.clone(), TEXT_SM, REGULAR, status_color));
    if app.ui.selected_playlist.is_some() {
        stage = stage.child(playlist_tracks::render(app));
    }
    if !app.connected() {
        stage = stage.child(login_view::render(app));
    }
    stage
}

/// Resource readout for debug builds only; release builds compile it out.
#[cfg(debug_assertions)]
fn dev_banner(app: &TinyTune) -> El {
    use super::theme::{ACCENT, BOLD, ON_ACCENT, TEXT_XS};

    let memory = app.memory_mb.map_or_else(
        || "RSS unavailable".into(),
        |value| format!("{value} MB RSS"),
    );
    row()
        .key("dev-banner")
        .w_full()
        .items_center()
        .gap(8.)
        .px(MAIN_PADDING_X)
        .py(4.)
        .fill(rgb(SURFACE))
        .child(
            row()
                .px(4.)
                .radius(2.)
                .fill(rgb(ACCENT))
                .child(label("DEV", TEXT_XS, BOLD, ON_ACCENT)),
        )
        .child(label(
            format!("Window ready in {} ms", app.startup_ms),
            TEXT_XS,
            REGULAR,
            MUTED,
        ))
        .child(rune::spacer())
        .child(label(memory, TEXT_XS, REGULAR, MUTED))
}

fn player_bar(app: &TinyTune) -> El {
    row()
        .key("player-bar")
        .w_full()
        .items_center()
        .gap(16.)
        .fill(rgb(SURFACE))
        .px(MAIN_PADDING_X)
        .py(12.)
        .child(now_playing::render(app))
        .child(player_controls::render(app))
}

fn title_for(state: ViewState) -> &'static str {
    match state {
        ViewState::Playing => chrome::WRAPPED_TITLE,
        ViewState::Paused => "Playback paused",
        ViewState::Authenticating => "Spotify sign in",
        ViewState::RestoringSession => "Welcome back",
        ViewState::Connecting | ViewState::Loading => "Almost there",
        ViewState::Ready => "Tiny Tune is connected",
        ViewState::ReauthRequired => "Reconnect Spotify",
        ViewState::SignedOut | ViewState::Failed => "A smaller Spotify player",
    }
}

#[cfg(test)]
#[path = "__tests__/app_view.rs"]
mod tests;
