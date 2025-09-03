use core::cmp::Ordering;

use std::borrow::Cow;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use either::Either;
use itertools::Itertools;
use percent_encoding::utf8_percent_encode;
use serde::{Deserialize, Serialize};
use serde_with::{
    formats::PreferMany, serde_as, DefaultOnNull, DeserializeAs, NoneAsEmptyString, OneOrMany,
    PickFirst, TimestampMilliSeconds,
};
use url::Url;

use crate::constants::{
    CATALOG_RESOURCE_NAME, CINEMETA_TOP_CATALOG_ID, CINEMETA_URL, GENRES_LINK_CATEGORY,
    IMDB_LINK_CATEGORY, IMDB_TITLE_PATH, IMDB_URL, URI_COMPONENT_ENCODE_SET,
};
use crate::deep_links::DiscoverDeepLinks;
use crate::types::addon::{ExtraValue, ResourcePath, ResourceRequest};
use crate::types::resource::{Stream, StreamSource};
use crate::types::{NumberAsString, SortedVec, SortedVecAdapter, UniqueVec, UniqueVecAdapter};

/// The [`MetaItem`] Id type to improve the readability of the code.
///
/// For example when using the id as key in a [`HashMap`].
pub type MetaItemId = String;

pub type MetaItemType = String;

#[derive(Clone, PartialEq, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
struct Trailer {
    source: String,
    r#type: String,
}

impl<'de> DeserializeAs<'de, Trailer> for Stream {
    fn deserialize_as<D>(deserializer: D) -> Result<Trailer, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let stream = Stream::deserialize(deserializer)?;
        match &stream.source {
            StreamSource::YouTube { yt_id } => Ok(Trailer {
                source: yt_id.to_owned(),
                r#type: "Trailer".to_owned(),
            }),
            _ => Err(serde::de::Error::custom("Unsuported Trailer StreamSource")),
        }
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
struct MetaItemPreviewLegacy {
    id: MetaItemId,
    r#type: String,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull")]
    name: String,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<NoneAsEmptyString>")]
    poster: Option<Url>,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<NoneAsEmptyString>")]
    background: Option<Url>,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<NoneAsEmptyString>")]
    logo: Option<Url>,
    description: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "Option<NumberAsString>")]
    release_info: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "Option<NumberAsString>")]
    runtime: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "PickFirst<(_, Option<TimestampMilliSeconds<i64>>)>")]
    released: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    poster_shape: PosterShape,
    #[serde(default)]
    #[serde_as(deserialize_as = "Option<NumberAsString>")]
    imdb_rating: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    genres: Vec<String>,
    #[serde(default)]
    links: Option<Vec<Link>>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull<Vec<PickFirst<(_, Stream)>>>")]
    trailers: Vec<Trailer>,
    #[serde(default)]
    trailer_streams: Option<Vec<Stream>>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    behavior_hints: MetaItemBehaviorHints,
}

///
/// ```
/// use stremio_core::types::resource::{MetaItemPreview, MetaItemBehaviorHints, PosterShape};
///
/// let expected = MetaItemPreview {
///     id: "tt:1111111".into(),
///     r#type: "movie".into(),
///     name: "Movie name 404".into(),
///     poster: None,
///     background: None,
///     logo: None,
///     description: None,
///     release_info: None,
///     runtime: None,
///     released: None,
///     poster_shape: PosterShape::default(),
///     links: vec![],
///     trailer_streams: vec![],
///     behavior_hints: MetaItemBehaviorHints::default(),
/// };
///
/// let null_fields = serde_json::json!({
///     "id": "tt:1111111",
///     "type": "movie",
///     "name": "Movie name 404",
///     "posterShape": null,
///     "links": null,
///     "trailers": null,
///     "trailerStreams": null,
///     "behaviorHints": null,
/// });
/// let meta_preview = serde_json::from_value::<MetaItemPreview>(null_fields).unwrap();
/// //assert_eq!(expected, meta_preview);
///
/// let no_fields = serde_json::json!({
///     "id": "tt:1111111",
///     "type": "movie",
///     "name": "Movie name 404",
/// });
///
/// let meta_preview = serde_json::from_value::<MetaItemPreview>(no_fields).unwrap();
/// assert_eq!(expected, meta_preview);
/// ```
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase", try_from = "MetaItemPreviewLegacy")]
pub struct MetaItemPreview {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub poster: Option<Url>,
    pub background: Option<Url>,
    pub logo: Option<Url>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub released: Option<DateTime<Utc>>,
    pub poster_shape: PosterShape,
    pub links: Vec<Link>,
    pub trailer_streams: Vec<Stream>,
    pub behavior_hints: MetaItemBehaviorHints,
}

impl From<MetaItemPreviewLegacy> for MetaItemPreview {
    fn from(legacy_item: MetaItemPreviewLegacy) -> Self {
        let links = match legacy_item.links {
            Some(links) => links,
            _ => {
                let id = legacy_item.id.to_owned();
                let r#type = legacy_item.r#type.to_owned();
                let imdb_link = legacy_item.imdb_rating.map(|imdb_rating| Link {
                    name: imdb_rating,
                    category: IMDB_LINK_CATEGORY.to_owned(),
                    url: IMDB_URL
                        .join(&format!("{IMDB_TITLE_PATH}/"))
                        .expect("IMDB url build failed")
                        .join(&utf8_percent_encode(&id, URI_COMPONENT_ENCODE_SET).to_string())
                        .expect("IMDB url build failed"),
                });
                let genres_links = legacy_item.genres.into_iter().map(|genre| {
                    let deep_links = DiscoverDeepLinks::from(&ResourceRequest {
                        base: CINEMETA_URL.to_owned(),
                        path: ResourcePath {
                            id: CINEMETA_TOP_CATALOG_ID.to_owned(),
                            resource: CATALOG_RESOURCE_NAME.to_owned(),
                            r#type: r#type.to_owned(),
                            extra: vec![ExtraValue {
                                name: "genre".to_owned(),
                                value: genre.to_owned(),
                            }],
                        },
                    });
                    Link {
                        name: genre,
                        category: GENRES_LINK_CATEGORY.to_owned(),
                        url: Url::parse(&deep_links.discover).expect("Link url parse failed"),
                    }
                });
                imdb_link.into_iter().chain(genres_links).collect()
            }
        };
        let trailer_streams = match legacy_item.trailer_streams {
            Some(trailer_streams) => trailer_streams,
            _ => legacy_item
                .trailers
                .into_iter()
                .filter(|trailer| trailer.r#type == "Trailer")
                .map(|trailer| Stream {
                    source: StreamSource::YouTube {
                        yt_id: trailer.source,
                    },
                    name: None,
                    description: None,
                    thumbnail: None,
                    subtitles: vec![],
                    behavior_hints: Default::default(),
                })
                .collect(),
        };
        Self {
            id: legacy_item.id,
            r#type: legacy_item.r#type,
            name: legacy_item.name,
            poster: legacy_item.poster,
            background: legacy_item.background,
            logo: legacy_item.logo,
            description: legacy_item.description,
            release_info: legacy_item.release_info,
            runtime: legacy_item.runtime,
            released: legacy_item.released,
            poster_shape: legacy_item.poster_shape,
            links,
            trailer_streams,
            behavior_hints: legacy_item.behavior_hints,
        }
    }
}

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    #[serde(flatten)]
    pub preview: MetaItemPreview,
    #[serde(default)]
    #[serde_as(
        as = "DefaultOnNull<SortedVec<UniqueVec<Vec<_>, VideoUniqueVecAdapter>, VideoSortedVecAdapter>>"
    )]
    pub videos: Vec<Video>,
}

impl MetaItem {
    /// Returns an iterator over references to Video, skipping special episodes, sorted by released and series_info, oldest first
    pub fn videos_iter(&self) -> impl DoubleEndedIterator<Item = &Video> {
        if self.preview.r#type == "series" {
            Either::Left(self.videos.iter().filter(|video| match video.series_info {
                Some(SeriesInfo { season, .. }) => season != 0,
                _ => false,
            }))
        } else {
            Either::Right(self.videos.iter().rev())
        }
    }

    /// Returns a vector of videos for a given season
    pub fn videos_by_season(&self, season: u32) -> Vec<&Video> {
        self.videos
            .iter()
            .filter(|video| {
                video
                    .series_info
                    .as_ref()
                    .is_some_and(|series_info| series_info.season == season)
            })
            .collect_vec()
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Square,
    Landscape,
    #[default]
    #[serde(other)]
    Poster,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}

/// The [`Video`] Id type to improve the readability of the code.
///
/// For example when using the id as key in a [`HashMap`].
pub type VideoId = String;

#[serde_as]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(default, alias = "name")]
    pub title: String,
    #[serde(default)]
    pub released: Option<DateTime<Utc>>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(alias = "stream", default)]
    #[serde_as(deserialize_as = "OneOrMany<_, PreferMany>")]
    pub streams: Vec<Stream>,
    #[serde(default, flatten)]
    pub series_info: Option<SeriesInfo>,
    #[serde(default)]
    pub trailer_streams: Vec<Stream>,
}

impl Video {
    pub fn stream(&self) -> Option<Cow<'_, Stream>> {
        self.streams
            .iter()
            .exactly_one()
            .ok()
            .map(Cow::Borrowed)
            .or_else(|| {
                if self.streams.is_empty() {
                    Stream::youtube(&self.id).map(Cow::Owned)
                } else {
                    None
                }
            })
    }
}

struct VideoUniqueVecAdapter;

impl UniqueVecAdapter for VideoUniqueVecAdapter {
    type Input = Video;
    type Output = String;

    fn hash(video: &Self::Input) -> Self::Output {
        video.id.to_owned()
    }
}

struct VideoSortedVecAdapter;

impl SortedVecAdapter for VideoSortedVecAdapter {
    type Input = Video;
    type Args = bool;

    fn args(videos: &[Video]) -> Self::Args {
        videos.iter().any(|video| video.series_info.is_some())
    }
    fn cmp(a: &Self::Input, b: &Self::Input, is_series: &Self::Args) -> Ordering {
        if let (Some(a), Some(b)) = (a.series_info.as_ref(), b.series_info.as_ref()) {
            let a_season = if a.season == 0 { u32::MAX } else { a.season };
            let b_season = if b.season == 0 { u32::MAX } else { b.season };
            a_season.cmp(&b_season).then(a.episode.cmp(&b.episode))
        } else if a.series_info.is_some() {
            std::cmp::Ordering::Less
        } else if b.series_info.is_some() {
            std::cmp::Ordering::Greater
        } else if let (Some(a), Some(b)) = (a.released, b.released) {
            if *is_series {
                a.cmp(&b)
            } else {
                b.cmp(&a)
            }
        } else if a.released.is_some() {
            std::cmp::Ordering::Less
        } else if b.released.is_some() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Link {
    pub name: String,
    pub category: String,
    pub url: Url,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemBehaviorHints {
    #[serde(default)]
    pub default_video_id: Option<String>,
    #[serde(default)]
    pub featured_video_id: Option<String>,
    #[serde(default)]
    pub has_scheduled_videos: bool,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
