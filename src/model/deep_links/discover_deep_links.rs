use serde::Serialize;
use stremio_core::types::addon::ResourceRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverDeepLinks {
    pub discover: String,
}

impl From<&ResourceRequest> for DiscoverDeepLinks {
    fn from(request: &ResourceRequest) -> Self {
        DiscoverDeepLinks::from(stremio_core::deep_links::DiscoverDeepLinks::from(request))
    }
}

impl From<stremio_core::deep_links::DiscoverDeepLinks> for DiscoverDeepLinks {
    fn from(deep_links: stremio_core::deep_links::DiscoverDeepLinks) -> Self {
        DiscoverDeepLinks {
            discover: deep_links.discover.replace("stremio://", "#"),
        }
    }
}
