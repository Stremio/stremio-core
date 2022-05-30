use serde::Serialize;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::{MetaItem, MetaItemPreview};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
}

impl From<(&MetaItem, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItem, &ResourceRequest)) -> Self {
        MetaItemDeepLinks::from(stremio_core::deep_links::MetaItemDeepLinks::from((
            item, request,
        )))
    }
}

impl From<(&MetaItemPreview, &ResourceRequest)> for MetaItemDeepLinks {
    fn from((item, request): (&MetaItemPreview, &ResourceRequest)) -> Self {
        MetaItemDeepLinks::from(stremio_core::deep_links::MetaItemDeepLinks::from((
            item, request,
        )))
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
