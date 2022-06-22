use serde::Serialize;
use std::convert::TryFrom;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::{MetaItem, MetaItemPreview};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
}

impl From<(&MetaItem, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItem, &ResourceRequest)) -> Self {
        stremio_core::deep_links::MetaItemDeepLinks::try_from((item, request))
            .ok()
            .map_or_else(Default::default, MetaItemDeepLinks::from)
    }
}

impl From<(&MetaItemPreview, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItemPreview, &ResourceRequest)) -> Self {
        stremio_core::deep_links::MetaItemDeepLinks::try_from((item, request))
            .ok()
            .map_or_else(Default::default, MetaItemDeepLinks::from)
    }
}

impl From<stremio_core::deep_links::MetaItemDeepLinks> for MetaItemDeepLinks {
    fn from(deep_links: stremio_core::deep_links::MetaItemDeepLinks) -> Self {
        MetaItemDeepLinks {
            meta_details_videos: deep_links
                .meta_details_videos
                .map(|meta_details_videos| meta_details_videos.replace("stremio://", "#")),
            meta_details_streams: deep_links
                .meta_details_streams
                .map(|meta_details_streams| meta_details_streams.replace("stremio://", "#")),
            player: deep_links
                .player
                .map(|player| player.replace("stremio://", "#")),
        }
    }
}
