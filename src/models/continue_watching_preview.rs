use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::models::common::eq_update;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::notifications::NotificationsBucket;
use lazysort::SortedBy;
use serde::Serialize;

#[derive(Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
/// The continue watching section in the app
pub struct ContinueWatchingPreview {
    pub library_items: Vec<LibraryItem>,
}

impl ContinueWatchingPreview {
    pub fn new(library: &LibraryBucket, notifications: &NotificationsBucket) -> (Self, Effects) {
        let mut library_items = vec![];
        let effects = library_items_update(
            &mut library_items,
            library,
            notifications,
            CATALOG_PREVIEW_SIZE,
        );
        (Self { library_items }, effects.unchanged())
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for ContinueWatchingPreview {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Internal(Internal::LibraryChanged(_)) => library_items_update(
                &mut self.library_items,
                &ctx.library,
                &ctx.notifications,
                None,
            ),
            Msg::Internal(Internal::NotificationsChanged) => library_items_update(
                &mut self.library_items,
                &ctx.library,
                &ctx.notifications,
                None,
            ),
            _ => Effects::none().unchanged(),
        }
    }
}

/// It will update and sort the give library items in Continue watching,
/// based on the `LibraryItem`s in the `LibraryBucket` and the videos notifications.
/// It will also limit the result to `limit` `LibraryItem`s, default being [`CATALOG_PREVIEW_SIZE`].
///
/// # Sorting
///
/// We compare either the release time of the video or the modified time of the `LibraryItem`,
/// if there's no release time.
/// If a user starts playing a new LibraryItem it should go in front of
/// the Continue Watching list. If a new episode comes out for a movie series
/// in that time it will be placed in first position.
fn library_items_update(
    library_items: &mut Vec<LibraryItem>,
    library: &LibraryBucket,
    notifications: &NotificationsBucket,
    limit: impl Into<Option<usize>>,
) -> Effects {
    let next_library_items = library
        .items
        .values()
        .filter(|library_item| library_item.is_in_continue_watching())
        .sorted_by(
            |library_item_a: &&LibraryItem, library_item_b: &&LibraryItem| {
                let datetime_a = notifications
                    .items
                    .get(&library_item_a.id)
                    .map(|notification| notification.video.released)
                    .flatten()
                    .unwrap_or(library_item_a.mtime);

                let datetime_b = notifications
                    .items
                    .get(&library_item_b.id)
                    .map(|notification| notification.video.released)
                    .flatten()
                    .unwrap_or(library_item_b.mtime);

                let cmp = datetime_b.cmp(&datetime_a);
                dbg!(library_item_a, library_item_b, cmp, datetime_a, datetime_b);
                cmp
            },
        )
        .take(limit.into().unwrap_or(CATALOG_PREVIEW_SIZE))
        .cloned()
        .collect::<Vec<_>>();
    eq_update(library_items, next_library_items)
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use chrono::{Duration, Utc};

    use crate::{
        constants::CATALOG_PREVIEW_SIZE,
        types::{
            library::{LibraryBucket, LibraryItem, LibraryItemState},
            notifications::{NotificationItem, NotificationsBucket},
            resource::{MetaItemBehaviorHints, MetaType, PosterShape, SeriesInfo, Video},
        },
    };

    use super::library_items_update;

    lazy_static::lazy_static! {
        /// The Last of Us
        ///
        /// The last video was released 2 days ago and we've watched it on that day.
        pub static ref SERIES_1: LibraryItem = {
            let last_video_released = Utc::now() - Duration::days(2);
            let item = LibraryItem {
                id: "tt3581920".into(),
                name: "The Last of Us".into(),
                r#type: MetaType::Series.to_string(),
                poster: Some("https://live.metahub.space/poster/small/tt3581920/img".parse().unwrap()),
                poster_shape: PosterShape::Poster,
                removed: false,
                temp: false,
                ctime: Some(Utc::now() - Duration::weeks(2)),
                mtime: last_video_released.clone(),
                state: LibraryItemState {
                    last_video_released: Some(last_video_released),
                    notifications_disabled: false,
                    // should be greater than 1!
                    time_offset: 1,
                    ..Default::default()
                },
                behavior_hints: MetaItemBehaviorHints::default(),
            };

            assert!(item.is_in_continue_watching(), "Should be in continue watching!");
            item
        };

        /// The Good Doctor
        ///
        /// The last video was released last week and it was the time we watched it too
        /// so `mtime` is the same.
        pub static ref SERIES_2: LibraryItem = {
            let last_video_released = Utc::now() - Duration::weeks(1);

            let item = LibraryItem {
                id: "tt6470478".into(),
                name: "The Good Doctor".into(),
                r#type: MetaType::Series.to_string(),
                poster: Some("https://live.metahub.space/poster/small/tt6470478/img".parse().unwrap()),
                poster_shape: PosterShape::Poster,
                removed: false,
                temp: false,
                ctime: Some(Utc::now() - Duration::weeks(2)),
                mtime: last_video_released.clone(),
                state: LibraryItemState {
                    last_video_released: Some(last_video_released),
                    notifications_disabled: false,
                    // should be greater than 1!
                    time_offset: 1,
                    ..Default::default()
                },
                behavior_hints: MetaItemBehaviorHints::default(),
            };
            assert!(item.is_in_continue_watching(), "Should be in continue watching!");
            item
        };

        /// Babylon
        ///
        /// Started watching it yesterday (10 mins only), `mtime` is yesterday.
        pub static ref MOVIE_1: LibraryItem = {
            let watched_yesterday = Utc::now() - Duration::days(1);
            let time_offset = u64::try_from(Duration::minutes(10).num_milliseconds()).unwrap();

            let item = LibraryItem {
                id: "tt10640346".into(),
                name: "Babylon".into(),
                r#type: MetaType::Movie.to_string(),
                poster: Some("https://images.metahub.space/poster/small/tt10640346/img".parse().unwrap()),
                poster_shape: PosterShape::Poster,
                removed: false,
                temp: false,
                // added yesterday
                ctime: Some(watched_yesterday.clone()),
                // TODO: Check this modified date and when it is updated
                mtime: watched_yesterday.clone(),
                state: LibraryItemState {
                    // watched yesterday
                    last_watched: Some(watched_yesterday.clone()),
                    // We've watched only 10 minutes of the movie
                    time_watched: time_offset.clone(),
                    // should be greater than 1!
                    time_offset,
                    last_video_released: None,
                    notifications_disabled: false,
                    times_watched: 0,
                    flagged_watched: 0,
                    duration: 11348509,
                    video_id: Some("tt10640346".into()),
                    // watched: Some("undefined:1:eJwDAAAAAAE=".parse().unwrap()),
                    watched: None,
                    overall_time_watched: 0,
                },
                behavior_hints: MetaItemBehaviorHints::default(),
            };
            assert!(item.is_in_continue_watching(), "Should be in continue watching!");
            item
        };

        pub static ref NOTIFICATIONS: NotificationsBucket = {
            NotificationsBucket {
                uid: None,
                items: vec![
                    // Last Of Us episode from 2 days ago, but we've watched it already
                    (
                        SERIES_1.id.clone(),
                        NotificationItem {
                            id: SERIES_1.id.clone(),
                            video: Video {
                                id: "tt3581920:1:8".into(),
                                title: "When We Are in Need".into(),
                                released: Some(Utc::now() - Duration::days(2)),
                                overview: None,
                                thumbnail: Some(
                                    "https://episodes.metahub.space/tt3581920/1/8/w780.jpg"
                                        .parse()
                                        .unwrap(),
                                ),
                                streams: vec![],
                                series_info: Some(SeriesInfo {
                                    season: 1,
                                    episode: 8,
                                }),
                                trailer_streams: vec![],
                            },
                        },
                    ),
                    // The Good Doctor episode released today (5 hours ago), but we've NOT watched it yet
                    (
                        SERIES_2.id.clone(),
                        NotificationItem {
                            id: SERIES_2.id.clone(),
                            video: Video {
                                id: "tt6470478:6:15".into(),
                                title: "Old Friends".into(),
                                released: Some(Utc::now() - Duration::hours(5)),
                                overview: Some("Dr. Jared Kalu makes a surprise return to San Jose’s St. Bonaventure Hospital with his billionaire patient. Meanwhile, Park must treat the man his wife had an affair with and try to find a way to forgive him.".into()),
                                thumbnail: Some(
                                    "https://episodes.metahub.space/tt6470478/6/15/w780.jpg"
                                        .parse()
                                        .unwrap(),
                                ),
                                streams: vec![],
                                series_info: Some(SeriesInfo {
                                    season: 6,
                                    episode: 15,
                                }),
                                trailer_streams: vec![],
                            },
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                created: Utc::now() - Duration::weeks(2),
            }
        };
    }

    /// Only Movie Series
    /// The Good Doctor does not (yet) have a new episode and we've already watched the new episode of The Last Of Us 2 days ago.
    #[test]
    fn sorting_continue_watching_only_movie_series() {
        let mut continue_watching_library_items = vec![SERIES_1.clone(), SERIES_2.clone()];

        let library_bucket = LibraryBucket {
            uid: None,
            items: vec![
                (SERIES_1.id.clone(), SERIES_1.clone()),
                (SERIES_2.id.clone(), SERIES_2.clone()),
            ]
            .into_iter()
            .collect(),
        };

        // New episode of The Good Doctor has came out of the Notifications.
        let effects = library_items_update(
            &mut continue_watching_library_items,
            &library_bucket,
            &NOTIFICATIONS,
            CATALOG_PREVIEW_SIZE,
        );

        assert!(
            effects.has_changed,
            "Continue watching library should have changed because we expect different ordering"
        );
        assert_eq!(
            continue_watching_library_items[0].id, SERIES_2.id,
            "The Good Doctor should be shown first in Continue Watching as there's a new episode"
        );
        assert_eq!(
            continue_watching_library_items[1].id, SERIES_1.id,
            "The Last Of Us should be shown second in Continue Watching"
        );
    }

    /// Movie Series and a Movie
    /// The Good Doctor does not (yet) have a new episode and we've watched
    /// the new episode of The Last Of Us released 2 days ago when it was released.
    /// We also have watched a bit of Babylon yesterday.
    #[test]
    fn sorting_continue_watching_with_movie_series_and_a_movie() {
        let mut continue_watching_library_items =
            vec![MOVIE_1.clone(), SERIES_1.clone(), SERIES_2.clone()];

        let library_bucket = LibraryBucket {
            uid: None,
            items: vec![
                (SERIES_1.id.clone(), SERIES_1.clone()),
                (SERIES_2.id.clone(), SERIES_2.clone()),
                (MOVIE_1.id.clone(), MOVIE_1.clone()),
            ]
            .into_iter()
            .collect(),
        };

        let effects = library_items_update(
            &mut continue_watching_library_items,
            &library_bucket,
            &NOTIFICATIONS,
            CATALOG_PREVIEW_SIZE,
        );

        assert!(
            effects.has_changed,
            "Continue watching library should have changed because we expect different ordering"
        );

        assert_eq!(
                continue_watching_library_items[0].id, SERIES_2.id,
                "The Good Doctor should be shown first in Continue Watching as there's a new Notification for today"
            );
        assert_eq!(
                continue_watching_library_items[1].id, MOVIE_1.id,
                "Babylon should be on second position in Continue Watching, as we watched some part of it yesterday"
            );
        assert_eq!(
            continue_watching_library_items[2].id, SERIES_1.id,
            "The Last Of Us should be shown last in Continue Watching, as we watched it 2 days ago"
        );
    }
}
