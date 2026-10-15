use gpui::{Context, Render, Window, div, prelude::*, rgb};

use super::{
    app::TinyTune,
    header, login_view, now_playing, player_controls, playlist_list,
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
        let memory = self.memory_mb.map_or_else(
            || "RSS unavailable".into(),
            |value| format!("{value} MB RSS"),
        );
        let has_playlists = !self.ui.playlists.is_empty();
        let connected = self.connected();
        let title = title_for(self.ui.view);

        let playlists = playlist_list::render(self, cx);
        let login = login_view::render(self, cx);
        let controls = player_controls::render(self, cx);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .p_8()
            .gap_6()
            .child(header::render())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap_5()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(SURFACE))
                    .p_8()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(status_color)
                            .child(self.ui.status.clone()),
                    )
                    .when(has_playlists, |view| view.child(playlists))
                    .when(!connected, |view| view.child(login)),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(format!("Window ready in {} ms", self.startup_ms))
                    .child(memory),
            )
            .when(connected, |view| {
                view.child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap_4()
                        .rounded_xl()
                        .border_1()
                        .border_color(rgb(BORDER))
                        .bg(rgb(SURFACE))
                        .px_4()
                        .py_3()
                        .child(now_playing::render(self))
                        .child(controls),
                )
            })
    }
}

fn title_for(state: ViewState) -> &'static str {
    match state {
        ViewState::Playing => "Your music is playing",
        ViewState::Paused => "Playback paused",
        ViewState::Authenticating => "Spotify sign in",
        ViewState::RestoringSession => "Welcome back",
        ViewState::Connecting | ViewState::Loading => "Almost there",
        ViewState::Ready => "Tiny Tune is connected",
        ViewState::ReauthRequired => "Reconnect Spotify",
        ViewState::SignedOut | ViewState::Failed => "A smaller Spotify player",
    }
}
