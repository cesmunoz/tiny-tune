use gpui::{Context, Render, Window, div, prelude::*, px, rgb};

use super::{
    app::{TinyTune, ViewState},
    header, login_view, player_controls, playlist_list,
    theme::{BACKGROUND, BORDER, ERROR, MUTED, SURFACE, TEXT},
};

impl Render for TinyTune {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_color = if self.state == ViewState::Failed {
            rgb(ERROR)
        } else {
            rgb(MUTED)
        };
        let memory = self.memory_mb.map_or_else(
            || "RSS unavailable".into(),
            |value| format!("{value} MB RSS"),
        );
        let has_playlists = !self.playlists.is_empty();
        let connected = self.connected();
        let title = title_for(self.state);

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
                            .child(self.status.clone()),
                    )
                    .when(has_playlists, |view| view.child(playlists))
                    .when_some(self.track_uri.clone(), |view, uri| {
                        view.child(
                            div()
                                .max_w(px(480.))
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .child(uri),
                        )
                    })
                    .when(!connected, |view| view.child(login))
                    .when(connected, |view| view.child(controls)),
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
    }
}

fn title_for(state: ViewState) -> &'static str {
    match state {
        ViewState::Playing => "Your music is playing",
        ViewState::Paused => "Playback paused",
        ViewState::Authenticating => "Spotify sign in",
        ViewState::Connecting | ViewState::Loading => "Almost there",
        ViewState::Ready => "Tiny Tune is connected",
        ViewState::SignedOut | ViewState::Failed => "A smaller Spotify player",
    }
}
