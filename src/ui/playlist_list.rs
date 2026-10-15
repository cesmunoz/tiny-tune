use gpui::{Context, div, prelude::*, px, rgb};

use crate::spotify::SpotifyCommand;

use super::{
    app::TinyTune,
    theme::{ACCENT, BORDER, TEXT},
};

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let list_height = if app.ui.selected_playlist.is_some() {
        150.
    } else {
        220.
    };
    let selected_uri = app
        .ui
        .selected_playlist
        .as_ref()
        .map(|detail| detail.summary.uri.clone());

    div()
        .id("playlists")
        .w_full()
        .max_w(px(640.))
        .max_h(px(list_height))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_2()
        .children(
            app.ui
                .playlists
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, playlist)| {
                    let open_playlist = playlist.clone();
                    let play_uri = playlist.uri.clone();
                    let selected = selected_uri.as_deref() == Some(playlist.uri.as_str());
                    div()
                        .id(("playlist", index))
                        .focusable()
                        .tab_stop(true)
                        .w_full()
                        .px_4()
                        .py_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(if selected { ACCENT } else { BORDER }))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(BORDER)))
                        .on_click(cx.listener(move |app, _, _, cx| {
                            let command = app.ui.open_playlist(open_playlist.clone());
                            app.send(command, cx);
                        }))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .child(div().overflow_hidden().child(playlist.name))
                        .child(
                            div()
                                .id(("playlist-play", index))
                                .focusable()
                                .tab_stop(true)
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .border_1()
                                .border_color(rgb(BORDER))
                                .text_xs()
                                .text_color(rgb(TEXT))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(BORDER)))
                                .on_click(cx.listener(move |app, _, _, cx| {
                                    cx.stop_propagation();
                                    app.send(
                                        SpotifyCommand::PlayPlaylist {
                                            uri: play_uri.clone(),
                                            start_track_uri: None,
                                        },
                                        cx,
                                    );
                                }))
                                .child("Play"),
                        )
                }),
        )
}
