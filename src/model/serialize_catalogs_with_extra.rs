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
        pub r#type: &'a String,
        pub id: &'a String,
        pub name: &'a String,
        pub poster: &'a Option<String>,
        pub poster_shape: &'a PosterShape,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub title: String,
        pub content: Loadable<Vec<MetaItemPreview<'a>>, String>,
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
                !matches!(&catalog.content, Loadable::Err(ResourceError::EmptyContent))
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
                    Loadable::Ready(meta_items) => Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| model::MetaItemPreview {
                                r#type: &meta_item.r#type,
                                id: &meta_item.id,
                                name: &meta_item.name,
                                poster: &meta_item.poster,
                                poster_shape: &meta_items.first().unwrap().poster_shape,
                                deep_links: MetaItemDeepLinks::from(meta_item),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error.to_string()),
                },
                deep_links: DiscoverDeepLinks::from(&catalog.request),
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}
