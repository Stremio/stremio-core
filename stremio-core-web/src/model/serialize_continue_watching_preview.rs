use gloo_utils::format::JsValueSerdeExt;
use url::Url;
use wasm_bindgen::JsValue;

use stremio_core::{
    models::continue_watching_preview::ContinueWatchingPreview,
    types::{profile::Settings, streams::StreamsBucket},
};

pub fn serialize_continue_watching_preview(
    continue_watching_preview: &ContinueWatchingPreview,
    streams_bucket: &StreamsBucket,
    streaming_server_url: Option<&Url>,
    settings: &Settings,
) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::ContinueWatchingPreview::from((
        continue_watching_preview,
        streams_bucket,
        streaming_server_url,
        settings,
    )))
    .expect("JsValue from model::ContinueWatchingPreview")
}

mod model {
    use serde::Serialize;
    use url::Url;

    use stremio_core::{
        deep_links::{LibraryDeepLinks, LibraryItemDeepLinks},
        types::{
            profile::Settings,
            resource::PosterShape,
            streams::{StreamsBucket, StreamsItem, StreamsItemKey},
        },
    };

    use crate::model::deep_links_ext::DeepLinksExt;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinueWatchingPreview<'a> {
        pub items: Vec<Item<'a>>,
        pub deep_links: LibraryDeepLinks,
    }

    impl<'a>
        From<(
            &'a stremio_core::models::continue_watching_preview::ContinueWatchingPreview,
            &StreamsBucket,
            Option<&Url>,
            &Settings,
        )> for ContinueWatchingPreview<'a>
    {
        fn from(
            (continue_watching_preview, streams_bucket, streaming_server_url, settings): (
                &'a stremio_core::models::continue_watching_preview::ContinueWatchingPreview,
                &StreamsBucket,
                Option<&Url>,
                &Settings,
            ),
        ) -> Self {
            Self {
                items: continue_watching_preview
                    .items
                    .iter()
                    .map(|core_cw_item| {
                        let library_item_stream = core_cw_item
                            .library_item
                            .state
                            .video_id
                            .clone()
                            .and_then(|video_id| {
                                streams_bucket.items.get(&StreamsItemKey {
                                    meta_id: core_cw_item.library_item.id.clone(),
                                    video_id,
                                })
                            });

                        Item::from((
                            core_cw_item,
                            library_item_stream,
                            streaming_server_url,
                            settings,
                        ))
                    })
                    .collect::<Vec<_>>(),
                deep_links: LibraryDeepLinks::from(&"continuewatching".to_owned())
                    .into_web_deep_links(),
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Item<'a> {
        #[serde(flatten)]
        library_item: LibraryItem<'a>,
        /// a count of the total notifications we have for this item
        notifications: usize,
    }

    impl<'a>
        From<(
            &'a stremio_core::models::continue_watching_preview::Item,
            Option<&StreamsItem>,
            Option<&Url>,
            &Settings,
        )> for Item<'a>
    {
        fn from(
            (item, stream_item, streaming_server_url, settings): (
                &'a stremio_core::models::continue_watching_preview::Item,
                Option<&StreamsItem>,
                Option<&Url>,
                &Settings,
            ),
        ) -> Self {
            Self {
                library_item: LibraryItem::from((
                    &item.library_item,
                    stream_item,
                    streaming_server_url,
                    settings,
                )),
                notifications: item.notifications,
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItem<'a> {
        #[serde(rename = "_id")]
        pub id: &'a String,
        pub name: &'a String,
        pub r#type: &'a String,
        pub poster: &'a Option<Url>,
        pub poster_shape: &'a PosterShape,
        pub progress: f64,
        pub deep_links: LibraryItemDeepLinks,
        pub state: LibraryItemState<'a>,
    }

    impl<'a>
        From<(
            &'a stremio_core::types::library::LibraryItem,
            Option<&StreamsItem>,
            Option<&Url>,
            &Settings,
        )> for LibraryItem<'a>
    {
        fn from(
            (library_item, streams_item, streaming_server_url, settings): (
                &'a stremio_core::types::library::LibraryItem,
                Option<&StreamsItem>,
                Option<&Url>,
                &Settings,
            ),
        ) -> Self {
            LibraryItem {
                id: &library_item.id,
                name: &library_item.name,
                r#type: &library_item.r#type,
                poster: &library_item.poster,
                poster_shape: match library_item.poster_shape {
                    // override poster shape if it's Landscape to over be a Square.
                    PosterShape::Landscape => &PosterShape::Square,
                    // else use the provided shape
                    _ => &library_item.poster_shape,
                },
                progress: library_item.progress(),
                deep_links: LibraryItemDeepLinks::from((
                    library_item,
                    streams_item,
                    streaming_server_url,
                    settings,
                ))
                .into_web_deep_links(),
                state: LibraryItemState::from(&library_item.state),
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItemState<'a> {
        pub video_id: Option<&'a String>,
    }

    impl<'a> From<&'a stremio_core::types::library::LibraryItemState> for LibraryItemState<'a> {
        fn from(state: &'a stremio_core::types::library::LibraryItemState) -> Self {
            Self {
                video_id: state.video_id.as_ref(),
            }
        }
    }
}
