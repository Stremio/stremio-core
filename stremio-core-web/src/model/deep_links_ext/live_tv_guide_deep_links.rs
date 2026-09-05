use crate::model::deep_links_ext::DeepLinksExt;
use stremio_core::deep_links::LiveTvGuideDeepLinks;

impl DeepLinksExt for LiveTvGuideDeepLinks {
    fn into_web_deep_links(self) -> Self {
        Self {
            live_tv_guide: self.live_tv_guide.replace("stremio://", "#"),
        }
    }
}
