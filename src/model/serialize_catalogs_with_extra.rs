use crate::env::WebEnv;
use crate::model::deep_links::{DiscoverDeepLinks, MetaItemDeepLinks, StreamDeepLinks};
use serde::Serialize;
use stremio_core::models::catalogs_with_extra::{CatalogsWithExtra, Selected};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::ResourceRequest;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Stream<'a> {
        #[serde(flatten)]
        pub stream: &'a stremio_core::types::resource::Stream,
        pub deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaItemPreview<'a> {
        #[serde(flatten)]
        pub meta_item: &'a stremio_core::types::resource::MetaItemPreview,
        pub trailer_streams: Vec<Stream<'a>>,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub request: &'a ResourceRequest,
        pub content: Loadable<Vec<MetaItemPreview<'a>>, &'a ResourceError>,
        pub addon_name: Option<&'a String>,
        pub deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CatalogsWithExtra<'a> {
        pub selected: &'a Option<Selected>,
        pub catalogs: Vec<ResourceLoadable<'a>>,
    }
}

pub fn serialize_catalogs_with_extra(
    catalogs_with_extra: &CatalogsWithExtra,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    JsValue::from_serde(&model::CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalogs: catalogs_with_extra
            .catalogs
            .iter()
            .map(|catalog| model::ResourceLoadable {
                request: &catalog.request,
                content: match &catalog.content {
                    Loadable::Ready(meta_items) => Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| model::MetaItemPreview {
                                meta_item,
                                trailer_streams: meta_item
                                    .trailer_streams
                                    .iter()
                                    .map(|stream| model::Stream {
                                        stream,
                                        deep_links: StreamDeepLinks::from(stream),
                                    })
                                    .collect::<Vec<_>>(),
                                deep_links: MetaItemDeepLinks::from(meta_item),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(&error),
                },
                addon_name: ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog.request.base)
                    .map(|addon| &addon.manifest.name),
                deep_links: DiscoverDeepLinks::from(&catalog.request),
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}
