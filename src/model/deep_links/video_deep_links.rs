use serde::Serialize;
use std::convert::TryFrom;
use stremio_core::deep_links::ExternalPlayerLink;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::Video;

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDeepLinks {
    pub meta_details_streams: String,
    pub player: Option<String>,
    pub external_player: Option<ExternalPlayerLink>,
}

impl From<(&Video, &ResourceRequest)> for VideoDeepLinks {
    fn from((video, request): (&Video, &ResourceRequest)) -> Self {
        stremio_core::deep_links::VideoDeepLinks::try_from((video, request))
            .ok()
            .map_or_else(Default::default, VideoDeepLinks::from)
    }
}

impl From<stremio_core::deep_links::VideoDeepLinks> for VideoDeepLinks {
    fn from(deep_links: stremio_core::deep_links::VideoDeepLinks) -> Self {
        VideoDeepLinks {
            meta_details_streams: deep_links.meta_details_streams.replace("stremio://", "#"),
            player: deep_links
                .player
                .map(|player| player.replace("stremio://", "#")),
            external_player: deep_links.external_player,
        }
    }
}
