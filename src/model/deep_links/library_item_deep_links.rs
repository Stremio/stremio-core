use serde::Serialize;
use stremio_core::deep_links::ExternalPlayerLink;
use stremio_core::types::library::LibraryItem;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDeepLinks {
    pub meta_details_videos: Option<String>,
    pub meta_details_streams: Option<String>,
    pub player: Option<String>,
    pub external_player: Option<ExternalPlayerLink>,
}

impl From<&LibraryItem> for LibraryItemDeepLinks {
    fn from(item: &LibraryItem) -> Self {
        LibraryItemDeepLinks::from(stremio_core::deep_links::LibraryItemDeepLinks::from(item))
    }
}

impl From<stremio_core::deep_links::LibraryItemDeepLinks> for LibraryItemDeepLinks {
    fn from(deep_links: stremio_core::deep_links::LibraryItemDeepLinks) -> Self {
        LibraryItemDeepLinks {
            meta_details_videos: deep_links
                .meta_details_videos
                .map(|meta_details_videos| meta_details_videos.replace("stremio://", "#")),
            meta_details_streams: deep_links
                .meta_details_streams
                .map(|meta_details_streams| meta_details_streams.replace("stremio://", "#")),
            player: deep_links
                .player
                .map(|player| player.replace("stremio://", "#")),
            external_player: deep_links.external_player,
        }
    }
}
