use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use stremio_core::deep_links::AddonsDeepLinks;
use stremio_core::models::catalog_with_filters::{CatalogWithFilters, Selected};
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::DescriptorPreview;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableCatalog<'a> {
        pub id: &'a String,
        pub name: &'a String,
        pub selected: &'a bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableType<'a> {
        pub r#type: &'a String,
        pub selected: &'a bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub catalogs: Vec<SelectableCatalog<'a>>,
        pub types: Vec<SelectableType<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        #[serde(flatten)]
        pub addon: &'a stremio_core::types::addon::DescriptorPreview,
        pub installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub content: Loadable<Vec<DescriptorPreview<'a>>, String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CatalogWithFilters<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Option<ResourceLoadable<'a>>,
    }
}

pub fn serialize_remote_addons(
    remote_addons: &CatalogWithFilters<DescriptorPreview>,
    ctx: &Ctx,
) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::CatalogWithFilters {
        selected: &remote_addons.selected,
        selectable: model::Selectable {
            catalogs: remote_addons
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| model::SelectableCatalog {
                    id: &selectable_catalog.request.path.id,
                    name: &selectable_catalog.catalog,
                    selected: &selectable_catalog.selected,
                    deep_links: AddonsDeepLinks::from(&selectable_catalog.request)
                        .into_web_deep_links(),
                })
                .collect(),
            types: remote_addons
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: AddonsDeepLinks::from(&selectable_type.request)
                        .into_web_deep_links(),
                })
                .collect(),
        },
        catalog: remote_addons
            .catalog
            .first()
            .map(|catalog| model::ResourceLoadable {
                content: match &catalog.content {
                    Some(Loadable::Ready(addons)) => Loadable::Ready(
                        addons
                            .iter()
                            .map(|addon| model::DescriptorPreview {
                                addon,
                                installed: ctx
                                    .profile
                                    .addons
                                    .iter()
                                    .map(|addon| &addon.transport_url)
                                    .any(|transport_url| *transport_url == addon.transport_url),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Some(Loadable::Loading) | None => Loadable::Loading,
                    Some(Loadable::Err(error)) => Loadable::Err(error.to_string()),
                },
            }),
    })
    .expect("JsValue from model::CatalogWithFilters")
}
