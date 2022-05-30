use serde::Serialize;
use stremio_core::deep_links::ExternalPlayerLink;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::Stream;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDeepLinks {
    pub player: String,
    pub external_player: ExternalPlayerLink,
}

impl From<&Stream> for StreamDeepLinks {
    fn from(stream: &Stream) -> Self {
        StreamDeepLinks::from(stremio_core::deep_links::StreamDeepLinks::from(stream))
    }
}

impl From<(&Stream, &ResourceRequest, &ResourceRequest)> for StreamDeepLinks {
    fn from(
        (stream, stream_request, meta_request): (&Stream, &ResourceRequest, &ResourceRequest),
    ) -> Self {
        StreamDeepLinks::from(stremio_core::deep_links::StreamDeepLinks::from((
            stream,
            stream_request,
            meta_request,
        )))
    }
}

impl From<stremio_core::deep_links::StreamDeepLinks> for StreamDeepLinks {
    fn from(deep_links: stremio_core::deep_links::StreamDeepLinks) -> Self {
        StreamDeepLinks {
            player: deep_links.player.replace("stremio://", "#"),
            external_player: deep_links.external_player,
        }
    }
}
