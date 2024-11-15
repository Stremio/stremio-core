use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::DiscoverDeepLinks;

impl DeepLinksExt for DiscoverDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            discover: self.discover.replace("stremio://", "#"),
        }
    }
}
