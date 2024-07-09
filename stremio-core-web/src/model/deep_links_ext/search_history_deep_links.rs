use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::SearchHistoryItemDeepLinks;

impl DeepLinksExt for SearchHistoryItemDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            search: self.search.replace("stremio://", "#"),
        }
    }
}
