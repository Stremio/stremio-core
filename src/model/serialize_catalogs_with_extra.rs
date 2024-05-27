use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use itertools::Itertools;
use serde::Serialize;
use stremio_core::deep_links::{DiscoverDeepLinks, MetaItemDeepLinks};
use stremio_core::models::catalogs_with_extra::{CatalogsWithExtra, Selected};
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::resource::PosterShape;
use wasm_bindgen::JsValue;

mod model {

    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
        pub id: &'a str,
        pub name: &'a str,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        pub manifest: ManifestPreview<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MetaItemPreview<'a> {
        #[serde(flatten)]
        pub meta_item: &'a stremio_core::types::resource::MetaItemPreview,
        pub poster_shape: &'a PosterShape,
        pub watched: bool,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub id: String,
        pub name: String,
        pub r#type: String,
        pub addon: DescriptorPreview<'a>,
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
    <JsValue as JsValueSerdeExt>::from_serde(&model::CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalogs: catalogs_with_extra
            .catalogs
            .iter()
            .filter_map(|catalog| catalog.first())
            .filter_map(|catalog| {
                ctx.profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog.request.base)
                    .and_then(|addon| {
                        addon
                            .manifest
                            .catalogs
                            .iter()
                            .find(|manifest_catalog| {
                                manifest_catalog.id == catalog.request.path.id
                                    && manifest_catalog.r#type == catalog.request.path.r#type
                            })
                            .map(|manifest_catalog| (addon, manifest_catalog, catalog))
                    })
            })
            .map(
                |(addon, manifest_catalog, catalog)| model::ResourceLoadable {
                    id: manifest_catalog.id.to_string(),
                    name: manifest_catalog
                        .name
                        .as_ref()
                        .unwrap_or(&addon.manifest.name)
                        .to_string(),
                    r#type: manifest_catalog.r#type.to_string(),
                    addon: model::DescriptorPreview {
                        manifest: model::ManifestPreview {
                            id: &addon.manifest.id,
                            name: &addon.manifest.name,
                        },
                    },
                    content: match &catalog.content {
                        Some(Loadable::Ready(meta_items)) => {
                            let poster_shape =
                                meta_items.first().map(|meta_item| &meta_item.poster_shape);
                            Some(Loadable::Ready(
                                meta_items
                                    .iter()
                                    .unique_by(|meta_item| &meta_item.id)
                                    .take(10)
                                    .map(|meta_item| model::MetaItemPreview {
                                        meta_item,
                                        poster_shape: poster_shape
                                            .unwrap_or(&meta_item.poster_shape),
                                        watched: ctx
                                            .library
                                            .items
                                            .get(&meta_item.id)
                                            .map(|library_item| library_item.watched())
                                            .unwrap_or_default(),
                                        deep_links: MetaItemDeepLinks::from((
                                            meta_item,
                                            &catalog.request,
                                        ))
                                        .into_web_deep_links(),
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        Some(Loadable::Loading) => Some(Loadable::Loading),
                        Some(Loadable::Err(error)) => Some(Loadable::Err(error.to_string())),
                        None => None,
                    },
                    deep_links: DiscoverDeepLinks::from(&catalog.request).into_web_deep_links(),
                },
            )
            .collect::<Vec<_>>(),
    })
    .expect("JsValue from model::CatalogsWithExtra")
}
