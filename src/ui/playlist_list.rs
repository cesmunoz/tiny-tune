use gpui::{Context, div, prelude::*, px, rgb};

use crate::spotify::SpotifyCommand;

use super::{app::TinyTune, theme::BORDER};

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    div()
        .id("playlists")
        .w_full()
        .max_w(px(520.))
        .max_h(px(220.))
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
                    let uri = playlist.uri.clone();
                    div()
                        .id(("playlist", index))
                        .focusable()
                        .tab_stop(true)
                        .w_full()
                        .px_4()
                        .py_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(BORDER))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(BORDER)))
                        .on_click(cx.listener(move |app, _, _, cx| {
                            app.send(SpotifyCommand::PlayPlaylist { uri: uri.clone() }, cx);
                        }))
                        .child(playlist.name)
                }),
        )
}
