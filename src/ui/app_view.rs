use gpui::{Context, Render, Window, div, prelude::*, px, rgb};

use super::{
    app::TinyTune,
    chrome::{self, MAIN_PADDING_X},
    header, login_view, now_playing, player_controls, playlist_list, playlist_tracks,
    state::ViewState,
    theme::{BACKGROUND, BORDER, ERROR, MUTED, SURFACE, TEXT},
};

impl Render for TinyTune {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_color = if self.ui.view == ViewState::Failed {
            rgb(ERROR)
        } else {
            rgb(MUTED)
        };
        let has_playlists = !self.ui.playlists.is_empty();
        let show_tracks = self.ui.selected_playlist.is_some();
        let connected = self.connected();
        let title = title_for(self.ui.view);
        let sidebar_width = px(chrome::shell_size().sidebar_width);

        let playlists = playlist_list::render(self, cx);
        let tracks = playlist_tracks::render(self, cx);
        let login = login_view::render(self, cx);
        let controls = player_controls::render(self, cx);
        let logout = player_controls::shell_logout(connected).map(|_| player_controls::logout(cx));
        let status = self.ui.status.clone();
        #[cfg(debug_assertions)]
        let dev_banner = Some(dev_banner(self));
        #[cfg(not(debug_assertions))]
        let dev_banner: Option<gpui::Div> = None;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .children(dev_banner)
            .child(shell_header(div().children(logout)))
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .overflow_hidden()
                    .child(library_column(sidebar_width, has_playlists, playlists))
                    .child(stage(
                        title,
                        status_color,
                        status,
                        show_tracks,
                        tracks,
                        connected,
                        login,
                    )),
            )
            .when(connected, |view| view.child(player_bar(self, controls)))
    }
}

fn shell_header(logout: impl IntoElement) -> impl IntoElement {
    div()
        .px(px(MAIN_PADDING_X))
        .py_4()
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(header::render(logout))
}

fn library_column(
    sidebar_width: gpui::Pixels,
    has_playlists: bool,
    playlists: impl IntoElement,
) -> impl IntoElement {
    div()
        .id("library")
        .w(sidebar_width)
        .flex_shrink_0()
        .h_full()
        .flex()
        .flex_col()
        .gap_3()
        .bg(rgb(SURFACE))
        .border_r_1()
        .border_color(rgb(BORDER))
        .p_4()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("Library"),
        )
        .when(has_playlists, |sidebar| sidebar.child(playlists))
        .when(!has_playlists, |sidebar| {
            sidebar.child(
                div()
                    .text_sm()
                    .text_color(rgb(MUTED))
                    .child("Connect to see your playlists"),
            )
        })
}

fn stage(
    title: &'static str,
    status_color: gpui::Rgba,
    status: String,
    show_tracks: bool,
    tracks: impl IntoElement,
    connected: bool,
    login: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_hidden()
        .px(px(MAIN_PADDING_X))
        .py_6()
        .child(
            div()
                .text_2xl()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .whitespace_nowrap()
                .overflow_hidden()
                .child(title),
        )
        .child(div().text_sm().text_color(status_color).child(status))
        .when(show_tracks, |stage| stage.child(tracks))
        .when(!connected, |stage| stage.child(login))
}

/// Resource readout for debug builds only; release builds compile it out.
#[cfg(debug_assertions)]
fn dev_banner(app: &TinyTune) -> gpui::Div {
    use super::theme::{ACCENT, ON_ACCENT};

    let memory = app.memory_mb.map_or_else(
        || "RSS unavailable".into(),
        |value| format!("{value} MB RSS"),
    );
    div()
        .flex()
        .items_center()
        .gap_2()
        .px(px(MAIN_PADDING_X))
        .py_1()
        .border_b_1()
        .border_color(rgb(BORDER))
        .bg(rgb(SURFACE))
        .text_xs()
        .text_color(rgb(MUTED))
        .child(
            div()
                .px_1()
                .rounded_sm()
                .bg(rgb(ACCENT))
                .text_color(rgb(ON_ACCENT))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("DEV"),
        )
        .child(format!("Window ready in {} ms", app.startup_ms))
        .child(div().flex_1())
        .child(memory)
}

fn player_bar(app: &TinyTune, controls: impl IntoElement) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap_4()
        .border_t_1()
        .border_color(rgb(BORDER))
        .bg(rgb(SURFACE))
        .px(px(MAIN_PADDING_X))
        .py_3()
        .child(now_playing::render(app))
        .child(controls)
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
