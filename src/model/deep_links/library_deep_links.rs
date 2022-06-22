use serde::Serialize;
use std::convert::TryFrom;
use stremio_core::models::library_with_filters::LibraryRequest;

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDeepLinks {
    pub library: String,
}

impl From<&String> for LibraryDeepLinks {
    fn from(root: &String) -> Self {
        LibraryDeepLinks::from(stremio_core::deep_links::LibraryDeepLinks::from(root))
    }
}

impl From<(&String, &LibraryRequest)> for LibraryDeepLinks {
    fn from((root, request): (&String, &LibraryRequest)) -> Self {
        stremio_core::deep_links::LibraryDeepLinks::try_from((root, request))
            .ok()
            .map_or_else(Default::default, LibraryDeepLinks::from)
    }
}

impl From<stremio_core::deep_links::LibraryDeepLinks> for LibraryDeepLinks {
    fn from(deep_links: stremio_core::deep_links::LibraryDeepLinks) -> Self {
        LibraryDeepLinks {
            library: deep_links.library.replace("stremio://", "#"),
        }
    }
}
