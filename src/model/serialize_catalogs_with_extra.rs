use serde::Serialize;
use stremio_core::models::catalogs_with_extra::{CatalogsWithExtra, Selected};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::ctx::Ctx;
use stremio_core::types::resource::PosterShape;
use stremio_deeplinks::{DiscoverDeepLinks, MetaItemDeepLinks};
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaItemPreview<'a> {
        #[serde(flatten)]
        pub meta_item: &'a stremio_core::types::resource::MetaItemPreview,
        pub poster_shape: &'a PosterShape,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub title: String,
        pub content: Option<Loadable<Vec<MetaItemPreview<'a>>, String>>,
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
    ctx: &Ctx,
) -> JsValue {
    JsValue::from_serde(&model::CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalogs: catalogs_with_extra
            .catalogs
            .iter()
            .filter(|catalog| {
                !matches!(
                    &catalog.content,
                    Some(Loadable::Err(ResourceError::EmptyContent))
                )
            })
            .filter_map(|catalog| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog.request.base)
                    .map(|addon| (addon, catalog))
            })
            .map(|(addon, catalog)| model::ResourceLoadable {
                title: format!(
                    "{} - {} {}",
                    &addon.manifest.name, &catalog.request.path.id, &catalog.request.path.r#type
                ),
                content: match &catalog.content {
                    Some(Loadable::Ready(meta_items)) => Some(Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| model::MetaItemPreview {
                                meta_item,
                                poster_shape: &meta_items.first().unwrap().poster_shape,
                                deep_links: MetaItemDeepLinks::from(meta_item),
                            })
                            .collect::<Vec<_>>(),
                    )),
                    Some(Loadable::Loading) => Some(Loadable::Loading),
                    Some(Loadable::Err(error)) => Some(Loadable::Err(error.to_string())),
                    None => None,
                },
                deep_links: DiscoverDeepLinks::from(&catalog.request),
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}
