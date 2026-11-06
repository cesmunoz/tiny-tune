use std::borrow::Cow;

use rune::{Cursor, El, GrowthSpec, ScrollSpec, col, edge, row};

use crate::{
    domain::{AppError, TrackSummary, is_spotify_track_uri},
    spotify::SpotifyCommand,
};

use super::{
    app::TinyTune,
    state::TrackPagePhase,
    theme::{
        ACCENT, ACCENT_HOVER, BOLD, BORDER, ERROR, MUTED, ON_ACCENT, REGULAR, ROUND, SEMIBOLD,
        TEXT, TEXT_SM, TEXT_XS, label, rgb,
    },
};

/// Height of one track row, including the gap below it.
const TRACK_ROW_PX: f32 = 56.;

pub fn render(app: &TinyTune) -> El {
    let Some(detail) = app.ui.selected_playlist.as_ref() else {
        return col();
    };
    let track_count = detail.tracks.len();
    let has_next = detail.next.is_some();
    let phase = &detail.phase;
    let loading = matches!(phase, TrackPagePhase::Loading);
    let failed = matches!(phase, TrackPagePhase::Failed(_));

    let mut pane = col()
        .key("playlist-tracks")
        .w_full()
        .grow(1.)
        .min_h(0.)
        .gap(12.)
        .child(pane_toolbar(
            &detail.summary.name,
            &detail.summary.uri,
            track_count,
            has_next,
        ))
        .children(phase_message(phase, track_count));

    if track_count > 0 {
        pane = pane.child(
            col()
                .key("playlist-track-scroll")
                .w_full()
                .grow(1.)
                .min_h(0.)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(
                    col()
                        .key("playlist-track-rows")
                        .w_full()
                        .growth(GrowthSpec {
                            spring: rune::GROWTH,
                            edge: edge::NO_MASK,
                        })
                        .virtual_rows(
                            track_count,
                            TRACK_ROW_PX,
                            |_: &TinyTune, index| Cow::Owned(format!("track-{index}")),
                            track_row,
                        ),
                ),
        );
    }
    if !loading && !failed && track_count == 0 {
        pane = pane.child(muted_line("This playlist has no tracks"));
    }
    if has_next && matches!(phase, TrackPagePhase::Ready) {
        pane = pane.child(text_button(
            "playlist-next-page",
            "Load next page",
            false,
            |app: &mut TinyTune| {
                if let Some(command) = app.ui.load_next_tracks() {
                    app.send(command);
                }
            },
        ));
    }
    if failed {
        pane = pane.child(text_button(
            "playlist-retry",
            "Try again",
            false,
            |app: &mut TinyTune| {
                if let Some(command) = app.ui.retry_tracks() {
                    app.send(command);
                }
            },
        ));
    }
    pane
}

fn pane_toolbar(name: &str, uri: &str, track_count: usize, has_next: bool) -> El {
    let playlist_uri = uri.to_owned();
    row()
        .key("playlist-toolbar")
        .w_full()
        .items_center()
        .justify_between()
        .gap(12.)
        .child(
            col()
                .gap(4.)
                .min_w(0.)
                .shrink()
                .child(label(name, TEXT_SM, SEMIBOLD, TEXT).truncate())
                .child(label(
                    track_count_label(track_count, has_next),
                    TEXT_XS,
                    REGULAR,
                    MUTED,
                )),
        )
        .child(
            row()
                .items_center()
                .gap(8.)
                .child(text_button(
                    "playlist-back",
                    "Library",
                    false,
                    |app: &mut TinyTune| app.ui.close_playlist(),
                ))
                .child(text_button(
                    "playlist-play",
                    "Play",
                    true,
                    move |app: &mut TinyTune| {
                        app.send(SpotifyCommand::PlayPlaylist {
                            uri: playlist_uri.clone(),
                            start_track_uri: None,
                        });
                    },
                )),
        )
}

/// Builds one virtual row. Tracks only ever append, so a row's index stays
/// the same track while the playlist is open.
fn track_row(app: &TinyTune, index: usize) -> El {
    let track = app
        .ui
        .selected_playlist
        .as_ref()
        .and_then(|detail| detail.tracks.get(index).cloned())
        .unwrap_or_else(TrackSummary::unavailable);
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

    let mut card = row()
        .key("card")
        .h(52.)
        .w_full()
        .px(12.)
        .items_center()
        .justify_between()
        .gap(12.)
        .radius(8.)
        .border(1., rgb(BORDER))
        .child(
            col()
                .min_w(0.)
                .shrink()
                .child(label(name.clone(), TEXT_SM, REGULAR, TEXT).truncate())
                .child(label(subtitle, TEXT_XS, REGULAR, MUTED).truncate()),
        )
        .child(label(duration, TEXT_XS, REGULAR, MUTED));

    if playable {
        card = card
            .role("button")
            .label(name)
            .focusable(true)
            .cursor(Cursor::Pointer)
            .hover(rgb(BORDER))
            .on_press(move |app: &mut TinyTune| {
                if let Some(command) = app.ui.play_track(index) {
                    app.send(command);
                }
            });
    } else {
        card = card.opacity(0.45);
    }
    // Virtual rows take padding rather than a stack gap.
    row().w_full().pb(4.).child(card)
}

fn phase_message(phase: &TrackPagePhase, track_count: usize) -> Option<El> {
    match phase {
        TrackPagePhase::Loading if track_count > 0 => Some(muted_line("Loading more tracks")),
        TrackPagePhase::Loading => Some(muted_line("Loading tracks")),
        TrackPagePhase::Ready => None,
        TrackPagePhase::Failed(error) => Some(label(
            tracks_failure_message(error),
            TEXT_SM,
            REGULAR,
            ERROR,
        )),
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

fn muted_line(text: &'static str) -> El {
    label(text, TEXT_SM, REGULAR, MUTED)
}

fn text_button(
    key: &'static str,
    caption: &'static str,
    primary: bool,
    on_press: impl Fn(&mut TinyTune) + 'static,
) -> El {
    let button = row()
        .key(key)
        .role("button")
        .label(caption)
        .focusable(true)
        .px(12.)
        .py(8.)
        .radius(ROUND)
        .cursor(Cursor::Pointer)
        .on_press(on_press);
    if primary {
        button
            .fill(rgb(ACCENT))
            .hover(rgb(ACCENT_HOVER))
            .child(label(caption, TEXT_XS, BOLD, ON_ACCENT))
    } else {
        button
            .border(1., rgb(BORDER))
            .hover(rgb(BORDER))
            .child(label(caption, TEXT_XS, REGULAR, TEXT))
    }
}

#[cfg(test)]
#[path = "__tests__/playlist_tracks.rs"]
mod tests;
