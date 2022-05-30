use serde::Serialize;
use stremio_core::models::installed_addons_with_filters::InstalledAddonsRequest;
use stremio_core::types::addon::ResourceRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonsDeepLinks {
    pub addons: String,
}

impl From<&ResourceRequest> for AddonsDeepLinks {
    fn from(request: &ResourceRequest) -> Self {
        AddonsDeepLinks::from(stremio_core::deep_links::AddonsDeepLinks::from(request))
    }
}

impl From<&InstalledAddonsRequest> for AddonsDeepLinks {
    fn from(request: &InstalledAddonsRequest) -> Self {
        AddonsDeepLinks::from(stremio_core::deep_links::AddonsDeepLinks::from(request))
    }
}

impl From<stremio_core::deep_links::AddonsDeepLinks> for AddonsDeepLinks {
    fn from(deep_links: stremio_core::deep_links::AddonsDeepLinks) -> Self {
        AddonsDeepLinks {
            addons: deep_links.addons.replace("stremio://", "#"),
        }
    }
}
