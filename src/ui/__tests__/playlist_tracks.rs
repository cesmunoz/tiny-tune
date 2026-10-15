use super::{format_duration, track_count_label, track_subtitle};
use crate::domain::TrackSummary;

#[test]
fn formats_track_rows_without_extra_metadata() {
    assert_eq!(format_duration(0), "0:00");
    assert_eq!(format_duration(61_000), "1:01");
    assert_eq!(track_count_label(0, false), "No tracks loaded");
    assert_eq!(track_count_label(50, true), "50 tracks, more available");
    assert_eq!(
        track_subtitle(&TrackSummary {
            id: None,
            spotify_uri: None,
            name: "Night".into(),
            artist_names: vec!["Ada".into(), "Bea".into()],
            album_name: "Hours".into(),
            duration_ms: 1_000,
            image: None,
            is_playable: true,
        }),
        "Ada, Bea · Hours"
    );
}
