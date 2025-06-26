use std::marker::PhantomData;

use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, DefaultOnNull, NoneAsEmptyString};
use stremio_watched_bitfield::{WatchedBitField, WatchedField};
use url::Url;

use crate::{
    runtime::Env,
    types::resource::{MetaItemBehaviorHints, MetaItemPreview, PosterShape, Video},
};

pub type LibraryItemId = String;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    #[serde(rename = "_id")]
    pub id: LibraryItemId,
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError<NoneAsEmptyString>")]
    pub poster: Option<Url>,
    #[serde(default)]
    pub poster_shape: PosterShape,
    pub removed: bool,
    pub temp: bool,
    /// Creation time
    #[serde(default, rename = "_ctime")]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub ctime: Option<DateTime<Utc>>,
    /// Modification time
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
    pub state: LibraryItemState,
    #[serde(default)]
    pub behavior_hints: MetaItemBehaviorHints,
}

impl LibraryItem {
    #[inline]
    pub fn should_sync<E: Env + 'static>(&self) -> bool {
        let year_ago = E::now() - Duration::days(365);
        let recently_removed = self.removed && self.mtime > year_ago;
        self.r#type != "other" && (!self.removed || recently_removed)
    }
    #[inline]
    pub fn is_in_continue_watching(&self) -> bool {
        self.r#type != "other" && (!self.removed || self.temp) && self.state.time_offset > 0
    }

    /// Returns watch progress percentage
    #[inline]
    pub fn progress(&self) -> f64 {
        if self.state.time_offset > 0 && self.state.duration > 0 {
            (self.state.time_offset as f64 / self.state.duration as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Returns whether the item has been watched
    #[inline]
    pub fn watched(&self) -> bool {
        self.state.times_watched > 0
    }

    /// Pulling notifications relies on a few key things:
    ///
    /// - LibraryItem should not be "other" or "movie", i.e. if you have "series" it will pull notifications
    /// - `LibraryItem.behavior_hints.default_video_id` should be `None`
    /// - The LibraryItem should not have been removed from the Library
    /// - The LibraryItem should not be temporary but in your LibraryItem
    pub fn should_pull_notifications(&self) -> bool {
        !self.state.no_notif
            && self.r#type != "other"
            && self.r#type != "movie"
            && self.behavior_hints.default_video_id.is_none()
            && !self.removed
            && !self.temp
    }

    #[inline]
    pub fn eq_no_mtime(&self, other: &LibraryItem) -> bool {
        self.id == other.id
            && self.removed == other.removed
            && self.temp == other.temp
            && self.ctime == other.ctime
            && self.state == other.state
            && self.name == other.name
            && self.r#type == other.r#type
            && self.poster == other.poster
            && self.poster_shape == other.poster_shape
            && self.behavior_hints == other.behavior_hints
    }

    pub fn mark_as_watched<E: Env>(&mut self, is_watched: bool) {
        if is_watched {
            self.state.times_watched = self.state.times_watched.saturating_add(1);
            self.state.last_watched = Some(E::now());
        } else {
            self.state.times_watched = 0;
        }
    }

    pub fn mark_video_as_watched<E: Env>(
        &mut self,
        watched: &WatchedBitField,
        video: &Video,
        is_watched: bool,
    ) {
        let mut watched = watched.to_owned();
        watched.set_video(&video.id, is_watched);

        self.state.watched = Some(watched.into());

        if is_watched {
            self.state.last_watched = match (&self.state.last_watched, &video.released) {
                (Some(last_watched), Some(released)) if last_watched < released => {
                    Some(released.to_owned())
                }
                (None, released) => released.to_owned(),
                (last_watched, _) => last_watched.to_owned(),
            };
        }
    }

    pub fn mark_videos_as_watched<E: Env>(
        &mut self,
        watched: &WatchedBitField,
        videos: Vec<&Video>,
        is_watched: bool,
    ) {
        let mut watched = watched.to_owned();

        for video in &videos {
            watched.set_video(&video.id, is_watched);
        }

        self.state.watched = Some(watched.into());

        if is_watched {
            self.state.last_watched = match (
                &self.state.last_watched,
                videos.last().and_then(|last_video| last_video.released),
            ) {
                (Some(last_watched), Some(released)) if last_watched < &released => {
                    Some(released.to_owned())
                }
                (None, released) => released.to_owned(),
                (last_watched, _) => last_watched.to_owned(),
            };
        }
    }
}

impl<E: Env + 'static> From<(&MetaItemPreview, PhantomData<E>)> for LibraryItem {
    fn from((meta_item, _): (&MetaItemPreview, PhantomData<E>)) -> Self {
        LibraryItem {
            id: meta_item.id.to_owned(),
            removed: true,
            temp: true,
            ctime: Some(E::now()),
            mtime: E::now(),
            state: LibraryItemState {
                last_watched: Some(E::now()),
                ..LibraryItemState::default()
            },
            name: meta_item.name.to_owned(),
            r#type: meta_item.r#type.to_owned(),
            poster: meta_item.poster.to_owned(),
            poster_shape: meta_item.poster_shape.to_owned(),
            behavior_hints: meta_item.behavior_hints.to_owned(),
        }
    }
}

impl From<(&MetaItemPreview, &LibraryItem)> for LibraryItem {
    fn from((meta_item, library_item): (&MetaItemPreview, &LibraryItem)) -> Self {
        LibraryItem {
            id: meta_item.id.to_owned(),
            name: meta_item.name.to_owned(),
            r#type: meta_item.r#type.to_owned(),
            poster: meta_item.poster.to_owned(),
            poster_shape: meta_item.poster_shape.to_owned(),
            behavior_hints: meta_item.behavior_hints.to_owned(),
            removed: library_item.removed,
            temp: library_item.temp,
            ctime: library_item.ctime.to_owned(),
            mtime: library_item.mtime.to_owned(),
            state: library_item.state.to_owned(),
        }
    }
}

#[serde_as]
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemState {
    /// Indicates the last time this [`LibraryItem`] was watched and:
    /// - When we're playing the LibraryItem we're constantly updating the field
    /// - Marking the whole [`LibraryItem`] as watched will update the field
    /// to the latest released video if `last_watched < released date`.
    /// - Marking a video from the [`LibraryItem`] will update the field
    /// to the video released date if `last_watched < released date`.
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub last_watched: Option<DateTime<Utc>>,
    /// In milliseconds
    pub time_watched: u64,
    /// In milliseconds
    pub time_offset: u64,
    /// In milliseconds
    pub overall_time_watched: u64,
    /// Shows how many times this item has been watched.
    ///
    /// Incremented once for each video watched (series)
    /// or in the case of no videos (e.g. movies) - every time
    pub times_watched: u32,
    // @TODO: consider bool that can be deserialized from an integer
    /// Flag indicating that a movie has been watched
    pub flagged_watched: u32,
    /// In milliseconds
    pub duration: u64,
    /// The last video watched.
    ///
    /// - For meta's without videos it's either `behavior_hints.default_video_id` (if present) or the `meta.id`
    /// - For meta's with video - the played video.
    #[serde(default, rename = "video_id")]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub video_id: Option<String>,
    /// Field tracking watched videos.
    /// For [`LibraryItem`]s without videos, this field should [`None`].
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    pub watched: Option<WatchedField>,
    /// Weather or not to receive notification for the given [`LibraryItem`].
    ///
    /// Default: receive notifications
    #[serde(default)]
    pub no_notif: bool,
}

impl LibraryItemState {
    pub fn watched_bitfield(&self, videos: &[Video]) -> WatchedBitField {
        let video_ids = videos
            .iter()
            .sorted_by(|a, b| {
                a.series_info
                    .as_ref()
                    .map(|info| info.season as i64)
                    .unwrap_or(i64::MIN)
                    .cmp(
                        &b.series_info
                            .as_ref()
                            .map(|info| info.season as i64)
                            .unwrap_or(i64::MIN),
                    )
                    .then(
                        a.series_info
                            .as_ref()
                            .map(|info| info.episode as i64)
                            .unwrap_or(i64::MIN)
                            .cmp(
                                &b.series_info
                                    .as_ref()
                                    .map(|info| info.episode as i64)
                                    .unwrap_or(i64::MIN),
                            ),
                    )
                    .then(
                        a.released
                            .as_ref()
                            .map(|released| released.timestamp_millis())
                            .unwrap_or(i64::MIN)
                            .cmp(
                                &b.released
                                    .as_ref()
                                    .map(|released| released.timestamp_millis())
                                    .unwrap_or(i64::MIN),
                            ),
                    )
            })
            .map(|video| &video.id)
            .cloned()
            .collect::<Vec<_>>();
        match &self.watched {
            Some(watched_field) => {
                // TODO: Construct WatchedBitField from `BitField8`
                match WatchedBitField::construct_with_videos(
                    watched_field.to_owned(),
                    video_ids.to_owned(),
                ) {
                    Ok(watched) => watched,
                    Err(_) => WatchedBitField::construct_from_array(vec![], video_ids),
                }
            }
            _ => WatchedBitField::construct_from_array(vec![], video_ids),
        }
    }
}
