use crate::types::addon::DescriptorPreview;
use crate::types::resource::{MetaItem, MetaItemPreview, Stream, Subtitles};
use derive_more::TryInto;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, TryInto, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceResponse {
    Metas {
        metas: Vec<MetaItemPreview>,
    },
    MetasDetailed {
        #[serde(rename = "camelCase")]
        metas_detailed: Vec<MetaItem>,
    },
    Meta {
        meta: MetaItem,
    },
    Streams {
        streams: Vec<Stream>,
    },
    Subtitles {
        subtitles: Vec<Subtitles>,
    },
    Addons {
        addons: Vec<DescriptorPreview>,
    },
}
