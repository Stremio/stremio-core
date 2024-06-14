use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::StreamDeepLinks;

impl DeepLinksExt for StreamDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            player: self.player.replace("stremio://", "#"),
            external_player: self.external_player,
        }
    }
}
