//! JS notification logic rewritten in Rust

use std::{ops::Div, time::Duration};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use percent_encoding::utf8_percent_encode;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use url::Url;

use crate::{
    addon_transport::{AddonHTTPTransport, AddonTransport},
    constants::{
        CATALOG_RESOURCE_NAME, INTRO_FEED_ID, METAHUB_URL, META_RESOURCE_NAME, OFFICIAL_ADDONS,
        URI_COMPONENT_ENCODE_SET,
    },
    runtime::{Env, EnvError},
    types::{
        addon::{ExtraValue, ResourcePath, ResourceResponse},
        resource::{MetaItem, SeriesInfo},
    },
};

// const redis = task.redis
// const feed = task.feed
const HOUR: Duration = Duration::from_secs(60 * 60);
pub const SERIES_LAST_VIDEOS: u8 = 3;
pub const CHANNEL_LAST_VIDEOS: u8 = 8;
// 10 minutes
pub const CACHE_BREAK_FREQ: Duration = Duration::from_secs(10 * 60);
/// 30 days
pub const MAX_NOTIF_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

lazy_static! {
    /// 6 hours
    pub static ref EPISODE_HAS_BEEN_OUT_FOR: chrono::Duration = chrono::Duration::from_std(Duration::from_secs(6 * 60 * 60)).expect("It is a valid Std duration!");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_as]
/// A user notification for movie series, youtube channels or movies.
// TODO: move to Models
pub struct Notification {
    /// notification id in the format: "{meta.preview.id} {series_info.season} {series_info.episode}"
    /// _id: [id, ep.season, epNumber].join(' '),
    #[serde(rename = "_id")]
    pub id: String,
    /// IMDB id
    /// imdb_id: meta.imdb_id,
    // TODO: Check if we have and need this info in `core`
    pub imdb_id: String,
    /// season: ep.season,
    /// episode: epNumber,
    #[serde(flatten)]
    pub series_info: SeriesInfo,
    /// Meta id
    /// item_id: id,
    pub item_id: String,
    /// Video id in the format: `{video.id}:{series_info.season}:{series_info.episode}`
    /// video_id: [id, ep.season, epNumber].join(':'),
    pub video_id: String,
    /// The notification type, e.g. `series`
    // TODO: Enum?
    // item_type: 'series',
    pub item_type: String,
    /// Meta name
    // item_name: meta.name,
    pub item_name: String,
    /// Background image of the notification
    // background: background,
    pub background: Url,
    /// Notification title, e.g. episode title
    // title: ep.name || ep.title,
    pub title: String,
    /// Meta name
    // name: meta.name,
    pub name: String,
    /// The type of the notification.
    // type: 'notification',
    pub r#type: String,
    // published: new Date(ep.firstAired),
    // TODO: Check what should happen if we don't have `released`, i.e. it's `None`
    // TODO: Check if it should be `published` or `released`!
    pub released: Option<DateTime<Utc>>,
    /// Created on DateTime
    // created: new Date(),
    pub created: DateTime<Utc>,
    /// Creation DateTime
    // _ctime: new Date(),
    #[serde_as(deserialize_as = "DefaultOnNull<NoneAsEmptyString>")]
    #[serde(default, rename = "_ctime")]
    // todo: Should this be Option or not?
    pub ctime: Option<DateTime<Utc>>,
    /// Modification DateTime
    // _mtime: new Date(),
    #[serde(rename = "_mtime")]
    pub mtime: DateTime<Utc>,
}

pub struct Feed {
    pub id: String,
}

pub async fn feeds_worker<E: Env>() -> Result<(), EnvError> {
    let feed = Feed {
        id: "tt:123123".into(),
    };
    // const isIntro = feed.id == consts.INTRO_FEED_ID
    let is_intro = feed.id == INTRO_FEED_ID;

    // const isSeries = feed.id.startsWith('tt')
    let is_series = feed.id.starts_with("tt");
    // const isChannel = feed.id.startsWith('yt_id')
    let is_channel = feed.id.starts_with("yt_id");

    let cinemeta_addon = OFFICIAL_ADDONS
        .iter()
        .find(|addon| addon.transport_url.as_str().contains("cinemeta.strem.io"))
        .expect("Cinemeta should exist in official addons!");

    let channels_addon = OFFICIAL_ADDONS
        .iter()
        .find(|addon| addon.transport_url.as_str().contains("channels.strem.io"))
        .expect("Channels should exist in official addons!");

    // const addon = isSeries ? addons.cinemeta : addons.channels
    let addon = if is_series {
        cinemeta_addon
    } else {
        channels_addon
    };

    // const type = isSeries ? 'series' : 'channel'
    let meta_type = if is_series { "series" } else { "channel" };

    // const lastVideos = isSeries ? consts.SERIES_LASTVIDEOS : consts.CHANNEL_LASTVIDEOS
    // TODO: Is this redundant? We use a different endpoint than the notifications in api.
    let last_videos = if is_series {
        SERIES_LAST_VIDEOS
    } else {
        CHANNEL_LAST_VIDEOS
    };

    // const cacheBreak = Math.floor(Date.now() / consts.CACHE_BREAK_FREQ)
    // by default `div` will floor the result for integers
    let cache_break = E::now()
        .timestamp_millis()
        .div(CACHE_BREAK_FREQ.as_millis() as i64);

    let transport = AddonHTTPTransport::<E>::new(addon.transport_url.clone());

    // Sort ids in ASC order!
    let video_ids = {
        let mut ids = ["tt7440726", "tt0944947"];
        ids.sort_unstable();
        ids.join(",")
    };
    let last_videos_ids = ExtraValue {
        name: "lastVideosIds".into(),
        value: utf8_percent_encode(&video_ids, URI_COMPONENT_ENCODE_SET).to_string(),
    };

    // https://v3-cinemeta.strem.io/catalog/series/last-videos/lastVideosIds=tt7440726,tt0944947.json
    // This is wrong:
    // `addon.get('meta', type, feed.id, { lastVideos, cacheBreak })`
    let response = transport
        .resource(&ResourcePath {
            resource: CATALOG_RESOURCE_NAME.into(),
            r#type: meta_type.into(),
            id: "last-videos".into(),
            extra: vec![last_videos_ids],
        })
        // .then(function(resp) {
        .await?;

    // const newerThan = Date.now() - consts.MAX_NOTIF_AGE
    let newer_than =
        E::now() - chrono::Duration::from_std(MAX_NOTIF_AGE).expect("Should be in range");

    if let ResourceResponse::MetasDetailed { metas_detailed } = response {
        // (released, Notification)
        // const notifs = mapMetaToNotifs(resp.meta)
        // .filter(x => x.published.getTime() > newerThan)
        let notifications: Vec<Notification> = metas_detailed
            .iter()
            .filter_map(|meta| {
                match meta.preview.r#type.as_str() {
                    // movie series
                    "series" => {
                        // meta.videos

                        let notifications = meta_to_notifs_series::<E>(meta)
                            .into_iter()
                            .filter(|notification| match notification.released {
                                Some(released) if released > newer_than => true,
                                _ => false,
                            })
                            .collect::<Vec<_>>();
                        Some(notifications)
                    }
                    // youtube
                    "channel" => {
                        todo!()
                    }
                    _ => None,
                }
            })
            .flatten()
            .collect();

        // 	const toAdd = notifs.map(n => [n.published.getTime(), n._id])
        let to_add = notifications
            .iter()
            .filter_map(|notification| {
                notification
                    .released
                    .map(|released| (released, notification.id.clone()))
            })
            .collect::<Vec<_>>();

        dbg!(metas_detailed);

        // const cmd = redis.multi()

        // Update the notification mtimes for that feed
        // TODO: This should be based on a single id for each meta item id, i.e. 2 calls for tt7440726 and tt0944947
        // 	const mtimesKey = consts.FEEDS_MTIME_KEY+feed.id
        // modify times? Redis key
        let mtimes_key = format!("feed:{}", feed.id);

        // 	cmd.zremrangebyscore(mtimesKey, 0, newerThan)
        // 	if (toAdd.length) {
        // 		// flatten toAdd for zadd
        // 		cmd.zadd(mtimesKey, toAdd.reduce((a, b) => a.concat(b), []))
        // 	}

        // 	// Save the notifications themselves
        // 	notifs.forEach(function(notif) {
        // 		// NOTE: this means the EXPIRES time will always be updated,
        // 		// but this is OK because the notification will go out of the lastVideos window
        // 		cmd.setex(consts.NOTIFS_KEY+notif._id, Math.floor(consts.MAX_NOTIF_AGE/1000), JSON.stringify(notif))
        // 	})

        // 	// Set the feed updated time
        // 	// @TODO: NOTE: we can set the updated to now + a few months, if resp.meta.status === 'Ended' as an optimization; essentially 'snooze'
        // 	cmd.hset(consts.FEEDS_UPDATED_KEY, feed.id, Date.now())

        // 	cmd.exec(cb)
    }

    // })
    // .catch(function(err) {
    // 	cb(err)
    // })
    Ok(())
}

pub fn meta_to_notifs_series<E: Env>(meta: &MetaItem) -> Vec<Notification> {
    let background = METAHUB_URL
        .join(&format!(
            "/background/small/{id}/img?from=notifs",
            id = &meta.preview.id
        ))
        .expect("Valid URL path");

    // return videos.filter(function(vid, i) {
    let released_videos = meta.videos.iter().filter(|vid| {
        // if (vid.streams && vid.streams.length) return true
        if !vid.streams.is_empty() {
            true
        } else {
            //     const t = new Date(vid.firstAired).getTime()
            vid.released
                .map(|released| {
                    let now = E::now();

                    // has the video been released earlier than (NOW - EPISODE_HAS_BEEN_OUT_FOR)
                    // return t < Date.now() - consts.EP_HAS_BEEN_OUT_FOR
                    released < (now - *EPISODE_HAS_BEEN_OUT_FOR)
                })
                // if no `released` available, skip this video
                // TODO: Check what should happen if we don't have `released`, i.e. it's `None`
                .unwrap_or(false)
        }
    });
    // })

    // .map(function(ep) {
    let notifications = released_videos.filter_map(|video| {
        // if no series_info is available, skip the video
        // const epNumber = ep.episode || ep.number
        let series_info = video.series_info.clone()?;
        let notif_id = format!(
            "{meta_id} {season} {episode}",
            meta_id = meta.preview.id,
            season = series_info.episode,
            episode = series_info.season
        );

        // video_id: [id, ep.season, epNumber].join(':'),
        let video_id = format!(
            "{meta_id}:{season}:{episode}",
            meta_id = meta.preview.id,
            season = series_info.episode,
            episode = series_info.season
        );

        // return {
        let notif = Notification {
            // _id: [id, ep.season, epNumber].join(' '),
            id: notif_id,
            // imdb_id: meta.imdb_id,
            imdb_id: "TODO".into(),
            // season: ep.season,
            // episode: epNumber,
            series_info,
            // item_id: id,
            item_id: meta.preview.id.clone(),
            // video_id: [id, ep.season, epNumber].join(':'),
            video_id,
            // item_type: 'series',
            item_type: "series".into(),
            // item_name: meta.name,
            item_name: meta.preview.name.clone(),
            // background: background,
            background: background.clone(),
            // title: ep.name || ep.title,
            title: video.title.clone(),
            // name: meta.name,
            name: meta.preview.name.clone(),
            // type: 'notification',
            r#type: "notification".into(),
            // published: new Date(ep.firstAired),
            // TODO: Check what should happen if we don't have `released`, i.e. it's `None`
            released: video.released.clone(),
            // created: new Date(),
            created: E::now(),
            // _ctime: new Date(),
            ctime: Some(E::now()),
            // _mtime: new Date(),
            mtime: E::now(),
        };
        // }

        Some(notif)
    });
    // })

    notifications.collect::<Vec<_>>()
}
