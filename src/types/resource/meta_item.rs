use crate::types::deserialize_single_as_vec;
use crate::types::resource::{Stream, StreamBehaviorHints, StreamSource};
use chrono::{DateTime, Utc};
use derivative::Derivative;
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use url::Url;

const TRANSPORT_URL: &str = "https://v3-cinemeta.strem.io/manifest.json";
const URI_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

fn deserialize_and_sort_videos<'de, D>(deserializer: D) -> Result<Vec<Video>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut videos: Vec<Video> = Vec::<Video>::deserialize(deserializer)?;
    let some_sessons = videos.iter().any(|v| v.series_info.is_some());

    videos.sort_by(|a, b| {
        if let (Some(a), Some(b)) = (a.series_info.as_ref(), b.series_info.as_ref()) {
            let a_season = if a.season == 0 { u32::MAX } else { a.season };
            let b_season = if b.season == 0 { u32::MAX } else { b.season };
            a_season.cmp(&b_season).then(a.episode.cmp(&b.episode))
        } else if a.series_info.is_some() {
            std::cmp::Ordering::Less
        } else if b.series_info.is_some() {
            std::cmp::Ordering::Greater
        } else if let (Some(a), Some(b)) = (a.released, b.released) {
            if some_sessons {
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
    });

    Ok(videos)
}

#[derive(Clone, PartialEq, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
struct Trailer {
    source: String,
    #[serde(rename = "type")]
    trailer_type: String,
}

#[derive(Clone, PartialEq, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
struct MetaItemLegacyPreview {
    id: String,
    r#type: String,
    #[serde(default)]
    name: String,
    poster: Option<String>,
    background: Option<String>,
    logo: Option<String>,
    description: Option<String>,
    release_info: Option<String>,
    runtime: Option<String>,
    released: Option<DateTime<Utc>>,
    #[serde(default)]
    poster_shape: PosterShape,
    imdb_rating: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    links: Vec<Link>,
    #[serde(default)]
    trailers: Vec<Trailer>,
    #[serde(default)]
    trailer_streams: Vec<Stream>,
    #[serde(default)]
    behavior_hints: MetaItemBehaviorHints,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase", try_from = "MetaItemLegacyPreview")]
pub struct MetaItemPreview {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub runtime: Option<String>,
    pub released: Option<DateTime<Utc>>,
    pub poster_shape: PosterShape,
    pub links: Vec<Link>,
    pub trailer_streams: Vec<Stream>,
    pub behavior_hints: MetaItemBehaviorHints,
}

impl TryFrom<MetaItemLegacyPreview> for MetaItemPreview {
    type Error = url::ParseError;
    fn try_from(legacy_item: MetaItemLegacyPreview) -> Result<Self, Self::Error> {
        let item_id = legacy_item.id.clone();
        let item_type = legacy_item.r#type.clone();
        let imdb_link: Result<Vec<Link>, url::ParseError> =
            legacy_item.imdb_rating.map_or(Ok(vec![]), |rating| {
                let enc_item_id = utf8_percent_encode(&item_id, URI_COMPONENT_ENCODE_SET);
                Ok(vec![Link {
                    name: rating,
                    category: "imdb".to_owned(),
                    url: Url::parse(format!("https://imdb.com/title/{}", enc_item_id).as_ref())?,
                }])
            });
        let genres_links: Result<Vec<Link>, url::ParseError> = legacy_item
            .genres
            .iter()
            .map(|genre| {
                let enc_transport = utf8_percent_encode(TRANSPORT_URL, URI_COMPONENT_ENCODE_SET);
                let enc_item_type = utf8_percent_encode(&item_type, URI_COMPONENT_ENCODE_SET);
                let enc_genre = utf8_percent_encode(&genre, URI_COMPONENT_ENCODE_SET);
                Ok(Link {
                    name: genre.to_owned(),
                    category: "Genres".to_owned(),
                    url: Url::parse(
                        format!(
                            "stremio:///discover/{}/{}/top?genre={}",
                            enc_transport, enc_item_type, enc_genre
                        )
                        .as_ref(),
                    )?,
                })
            })
            .collect();
        Ok(Self {
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
            // TODO: deduplicate links?
            links: [imdb_link?, genres_links?, legacy_item.links].concat(),
            trailer_streams: [
                legacy_item
                    .trailers
                    .into_iter()
                    .map(|trailer| Stream {
                        source: StreamSource::YouTube {
                            yt_id: trailer.source,
                        },
                        name: None,
                        description: None,
                        thumbnail: None,
                        subtitles: vec![],
                        behavior_hints: StreamBehaviorHints::default(),
                    })
                    .collect(),
                legacy_item.trailer_streams,
            ]
            .concat(),
            behavior_hints: legacy_item.behavior_hints,
        })
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    #[serde(flatten)]
    pub preview: MetaItemPreview,
    #[serde(default, deserialize_with = "deserialize_and_sort_videos")]
    pub videos: Vec<Video>,
}

#[derive(Derivative, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Square,
    Landscape,
    #[derivative(Default)]
    #[serde(other)]
    Poster,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(test, derive(Default))]
pub struct SeriesInfo {
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    #[serde(default, alias = "name")]
    pub title: String,
    pub released: Option<DateTime<Utc>>,
    pub overview: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(
        alias = "stream",
        deserialize_with = "deserialize_single_as_vec",
        default
    )]
    pub streams: Vec<Stream>,
    #[serde(flatten)]
    pub series_info: Option<SeriesInfo>,
    #[serde(default)]
    pub trailer_streams: Vec<Stream>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Link {
    pub name: String,
    pub category: String,
    pub url: Url,
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
