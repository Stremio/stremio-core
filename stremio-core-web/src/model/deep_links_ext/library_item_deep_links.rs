use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::LibraryItemDeepLinks;

impl DeepLinksExt for LibraryItemDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            meta_details_videos: self
                .meta_details_videos
                .map(|meta_details_videos| meta_details_videos.replace("stremio://", "#")),
            meta_details_streams: self
                .meta_details_streams
                .map(|meta_details_streams| meta_details_streams.replace("stremio://", "#")),
            player: self.player.map(|player| player.replace("stremio://", "#")),
            external_player: self.external_player,
        }
    }
}
