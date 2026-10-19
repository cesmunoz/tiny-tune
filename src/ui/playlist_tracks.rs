use gpui::{
    App, Context, Entity, ListSizingBehavior, Window, div, prelude::*, px, rgb, uniform_list,
};

use crate::{
    domain::{AppError, TrackSummary, is_spotify_track_uri},
    spotify::SpotifyCommand,
};

use super::{
    app::TinyTune,
    state::TrackPagePhase,
    theme::{ACCENT, ACCENT_HOVER, BORDER, ERROR, MUTED, ON_ACCENT, TEXT},
};

struct TrackPane {
    name: String,
    uri: String,
    track_count: usize,
    has_next: bool,
    phase: TrackPagePhase,
}

pub fn render(app: &TinyTune, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let pane = app.ui.selected_playlist.as_ref().map(|detail| TrackPane {
        name: detail.summary.name.clone(),
        uri: detail.summary.uri.clone(),
        track_count: detail.tracks.len(),
        has_next: detail.next.is_some(),
        phase: detail.phase.clone(),
    });
    let entity = cx.entity();

    div()
        .w_full()
        .flex_1()
        .min_h(px(0.))
        .when_some(pane, |parent, pane| {
            parent.child(track_pane(pane, entity, cx))
        })
}

fn track_pane(
    pane: TrackPane,
    entity: Entity<TinyTune>,
    cx: &mut Context<TinyTune>,
) -> impl IntoElement + use<> {
    let track_count = pane.track_count;
    let phase = pane.phase.clone();
    let has_next = pane.has_next;
    let loading = matches!(phase, TrackPagePhase::Loading);
    let failed = matches!(phase, TrackPagePhase::Failed(_));

    div()
        .id("playlist-tracks")
        .w_full()
        .flex_1()
        .min_h(px(0.))
        .flex()
        .flex_col()
        .gap_3()
        .child(pane_toolbar(pane, cx))
        .child(phase_message(&phase, track_count))
        .when(track_count > 0, |parent| {
            parent.child(
                div().w_full().flex_1().min_h(px(0.)).child(
                    uniform_list("playlist-track-rows", track_count, move |range, _, app| {
                        render_track_rows(&entity, range, app)
                    })
                    .w_full()
                    .h_full()
                    .with_sizing_behavior(ListSizingBehavior::Infer),
                ),
            )
        })
        .when(!loading && !failed && track_count == 0, |parent| {
            parent.child(muted_line("This playlist has no tracks"))
        })
        .when(
            has_next && matches!(phase, TrackPagePhase::Ready),
            |parent| {
                parent.child(text_button(
                    "playlist-next-page",
                    "Load next page",
                    false,
                    cx.listener(|app, _, _, cx| {
                        if let Some(command) = app.ui.load_next_tracks() {
                            app.send(command, cx);
                        }
                    }),
                ))
            },
        )
        .when(failed, |parent| {
            parent.child(text_button(
                "playlist-retry",
                "Try again",
                false,
                cx.listener(|app, _, _, cx| {
                    if let Some(command) = app.ui.retry_tracks() {
                        app.send(command, cx);
                    }
                }),
            ))
        })
}

fn pane_toolbar(pane: TrackPane, cx: &mut Context<TinyTune>) -> impl IntoElement + use<> {
    let playlist_uri = pane.uri.clone();
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .overflow_hidden()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(pane.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(track_count_label(pane.track_count, pane.has_next)),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(text_button(
                    "playlist-back",
                    "Library",
                    false,
                    cx.listener(|app, _, _, cx| {
                        app.ui.close_playlist();
                        cx.notify();
                    }),
                ))
                .child(text_button(
                    "playlist-play",
                    "Play",
                    true,
                    cx.listener(move |app, _, _, cx| {
                        app.send(
                            SpotifyCommand::PlayPlaylist {
                                uri: playlist_uri.clone(),
                                start_track_uri: None,
                            },
                            cx,
                        );
                    }),
                )),
        )
}

fn render_track_rows(
    entity: &Entity<TinyTune>,
    range: std::ops::Range<usize>,
    app: &mut App,
) -> Vec<gpui::Stateful<gpui::Div>> {
    range
        .map(|index| {
            let track = entity
                .read(app)
                .ui
                .selected_playlist
                .as_ref()
                .and_then(|detail| detail.tracks.get(index).cloned());
            track_row(entity.clone(), index, track)
        })
        .collect()
}

fn track_row(
    entity: Entity<TinyTune>,
    index: usize,
    track: Option<TrackSummary>,
) -> gpui::Stateful<gpui::Div> {
    let track = track.unwrap_or_else(TrackSummary::unavailable);
    let playable = track.is_playable
        && track
            .spotify_uri
            .as_deref()
            .is_some_and(is_spotify_track_uri);
    let subtitle = track_subtitle(&track);
    let duration = if track.duration_ms > 0 {
        format_duration(track.duration_ms)
    } else {
        String::new()
    };
    let name = if track.name.is_empty() {
        "Unavailable".into()
    } else {
        track.name
    };

    let mut row = div()
        .id(("playlist-track", index))
        .h(px(52.))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .rounded_lg()
        .border_1()
        .border_color(rgb(BORDER))
        .overflow_hidden()
        .child(
            div()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .text_sm()
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(subtitle),
                ),
        )
        .child(div().text_xs().text_color(rgb(MUTED)).child(duration));

    if playable {
        row = row
            .focusable()
            .tab_stop(true)
            .cursor_pointer()
            .hover(|style| style.bg(rgb(BORDER)))
            .on_click(move |_, _, cx| {
                entity.update(cx, |app, cx| {
                    if let Some(command) = app.ui.play_track(index) {
                        app.send(command, cx);
                    }
                });
            });
    } else {
        row = row.opacity(0.45);
    }
    row
}

fn phase_message(phase: &TrackPagePhase, track_count: usize) -> gpui::AnyElement {
    match phase {
        TrackPagePhase::Loading if track_count > 0 => {
            muted_line("Loading more tracks").into_any_element()
        }
        TrackPagePhase::Loading => muted_line("Loading tracks").into_any_element(),
        TrackPagePhase::Ready => div().into_any_element(),
        TrackPagePhase::Failed(error) => div()
            .text_sm()
            .text_color(rgb(ERROR))
            .child(tracks_failure_message(error))
            .into_any_element(),
    }
}

fn tracks_failure_message(error: &AppError) -> String {
    error.tracks_message()
}

fn track_count_label(track_count: usize, has_next: bool) -> String {
    if track_count == 0 {
        return "No tracks loaded".into();
    }
    if has_next {
        format!("{track_count} tracks, more available")
    } else {
        format!("{track_count} tracks")
    }
}

fn track_subtitle(track: &TrackSummary) -> String {
    let artists = track.artist_names.join(", ");
    match (artists.is_empty(), track.album_name.is_empty()) {
        (true, true) => String::new(),
        (false, true) => artists,
        (true, false) => track.album_name.clone(),
        (false, false) => format!("{artists} · {}", track.album_name),
    }
}

fn format_duration(duration_ms: u32) -> String {
    let total_seconds = duration_ms / 1_000;
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}

fn muted_line(text: &'static str) -> gpui::Div {
    div().text_sm().text_color(rgb(MUTED)).child(text)
}

fn text_button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    listener: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .focusable()
        .tab_stop(true)
        .px_3()
        .py_2()
        .rounded_full()
        .text_xs()
        .cursor_pointer()
        .when(primary, |button| {
            button
                .bg(rgb(ACCENT))
                .text_color(rgb(ON_ACCENT))
                .font_weight(gpui::FontWeight::BOLD)
                .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        })
        .when(!primary, |button| {
            button
                .border_1()
                .border_color(rgb(BORDER))
                .text_color(rgb(TEXT))
                .hover(|style| style.bg(rgb(BORDER)))
        })
        .on_click(listener)
        .child(label)
}

#[cfg(test)]
#[path = "__tests__/playlist_tracks.rs"]
mod tests;
