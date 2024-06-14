use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::VideoDeepLinks;

impl DeepLinksExt for VideoDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            meta_details_streams: self.meta_details_streams.replace("stremio://", "#"),
            player: self.player.map(|player| player.replace("stremio://", "#")),
            external_player: self.external_player,
        }
    }
}
