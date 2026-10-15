use librespot::metadata::{audio::AudioItem, audio::UniqueFields, audio::item::CoverImage};

use crate::domain::{ImageRef, TrackId, TrackSummary};

pub(crate) const ARTWORK_EDGE_PX: u32 = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CoverChoice {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub(crate) fn project_audio_item(item: &AudioItem) -> TrackSummary {
    let spotify_uri = if item.uri.is_empty() {
        item.track_id.to_uri().ok()
    } else {
        Some(item.uri.clone())
    };
    let id = spotify_uri.as_deref().and_then(track_id_from_uri);
    TrackSummary {
        id,
        spotify_uri,
        name: item.name.clone(),
        artist_names: artist_names(item),
        album_name: album_name(item),
        duration_ms: item.duration_ms,
        image: choose_cover(&cover_choices(&item.covers), ARTWORK_EDGE_PX),
        is_playable: item.availability.is_ok(),
    }
}

pub(crate) fn choose_cover(covers: &[CoverChoice], target_edge: u32) -> Option<ImageRef> {
    let mut best_fit: Option<&CoverChoice> = None;
    let mut largest: Option<&CoverChoice> = None;

    for cover in covers {
        if cover.url.is_empty() {
            continue;
        }
        let Some(edge) = cover_edge(cover) else {
            continue;
        };
        if edge >= target_edge {
            let closer = best_fit
                .and_then(cover_edge)
                .is_none_or(|current| edge < current);
            if closer {
                best_fit = Some(cover);
            }
        }
        let bigger = largest
            .and_then(cover_edge)
            .is_none_or(|current| edge > current);
        if bigger {
            largest = Some(cover);
        }
    }

    best_fit.or(largest).map(image_ref).or_else(|| {
        covers
            .iter()
            .find(|cover| !cover.url.is_empty())
            .map(image_ref)
    })
}

fn cover_choices(covers: &[CoverImage]) -> Vec<CoverChoice> {
    covers
        .iter()
        .map(|cover| CoverChoice {
            url: cover.url.clone(),
            width: u32::try_from(cover.width).ok().filter(|width| *width > 0),
            height: u32::try_from(cover.height)
                .ok()
                .filter(|height| *height > 0),
        })
        .collect()
}

fn image_ref(cover: &CoverChoice) -> ImageRef {
    ImageRef {
        url: cover.url.clone(),
        width: cover.width,
        height: cover.height,
    }
}

fn cover_edge(cover: &CoverChoice) -> Option<u32> {
    Some(cover.width?.min(cover.height?))
}

fn track_id_from_uri(uri: &str) -> Option<TrackId> {
    uri.strip_prefix("spotify:track:")
        .filter(|id| !id.is_empty() && !id.contains(':'))
        .map(TrackId::new)
}

fn artist_names(item: &AudioItem) -> Vec<String> {
    match &item.unique_fields {
        UniqueFields::Track { artists, .. } => primary_artists(artists),
        UniqueFields::Local { artists, .. } => artists
            .iter()
            .filter(|name| !name.is_empty())
            .cloned()
            .collect(),
        UniqueFields::Episode { show_name, .. } => non_empty(show_name),
    }
}

fn album_name(item: &AudioItem) -> String {
    match &item.unique_fields {
        UniqueFields::Track { album, .. } => album.clone(),
        UniqueFields::Local { album, .. } => album.clone().unwrap_or_default(),
        UniqueFields::Episode { show_name, .. } => show_name.clone(),
    }
}

fn primary_artists(artists: &librespot::metadata::artist::ArtistsWithRole) -> Vec<String> {
    use librespot::protocol::metadata::artist_with_role::ArtistRole;

    let primary: Vec<String> = artists
        .iter()
        .filter(|artist| {
            matches!(
                artist.role,
                ArtistRole::ARTIST_ROLE_UNKNOWN
                    | ArtistRole::ARTIST_ROLE_MAIN_ARTIST
                    | ArtistRole::ARTIST_ROLE_FEATURED_ARTIST
            )
        })
        .map(|artist| artist.name.clone())
        .filter(|name| !name.is_empty())
        .collect();
    if !primary.is_empty() {
        return primary;
    }
    artists
        .iter()
        .map(|artist| artist.name.clone())
        .filter(|name| !name.is_empty())
        .collect()
}

fn non_empty(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        vec![value.to_owned()]
    }
}

#[cfg(test)]
#[path = "__tests__/now_playing.rs"]
mod tests;
