use boolinator::Boolinator;
use gloo_utils::format::JsValueSerdeExt;
use itertools::Itertools;

use serde::Serialize;
use wasm_bindgen::JsValue;

use stremio_core::deep_links::{DiscoverDeepLinks, MetaItemDeepLinks, StreamDeepLinks};
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::types::resource::MetaItemPreview;

use crate::model::deep_links_ext::DeepLinksExt;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
        pub id: &'a str,
        pub name: &'a String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        pub manifest: ManifestPreview<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableExtraOption<'a> {
        pub value: &'a Option<String>,
        pub selected: &'a bool,
        pub deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableExtra<'a> {
        pub name: &'a String,
        pub is_required: &'a bool,
        pub options: Vec<SelectableExtraOption<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableCatalog<'a> {
        pub id: &'a String,
        pub name: &'a String,
        pub r#type: &'a str,
        pub addon: DescriptorPreview<'a>,
        pub selected: &'a bool,
        pub deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableType<'a> {
        pub r#type: &'a String,
        pub selected: &'a bool,
        pub deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub catalogs: Vec<SelectableCatalog<'a>>,
        pub extra: Vec<SelectableExtra<'a>>,
        pub next_page: bool,
    }
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
        pub watched: bool,
        pub in_library: bool,
        pub deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceLoadable<'a> {
        pub content: Loadable<Vec<MetaItemPreview<'a>>, String>,
        pub installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CatalogWithFilters<'a> {
        pub selected: &'a Option<CatalogWithFiltersSelected>,
        pub selectable: Selectable<'a>,
        pub catalog: Option<ResourceLoadable<'a>>,
    }
}

pub fn serialize_discover(
    discover: &CatalogWithFilters<MetaItemPreview>,
    ctx: &Ctx,
    streaming_server: &StreamingServer,
) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::CatalogWithFilters {
        selected: &discover.selected,
        selectable: model::Selectable {
            types: discover
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: DiscoverDeepLinks::from(&selectable_type.request)
                        .into_web_deep_links(),
                })
                .collect(),
            catalogs: discover
                .selectable
                .catalogs
                .iter()
                .filter_map(|selectable_catalog| {
                    ctx.profile
                        .addons
                        .iter()
                        .find(|addon| addon.transport_url == selectable_catalog.request.base)
                        .map(|addon| (addon, selectable_catalog))
                })
                .map(|(addon, selectable_catalog)| model::SelectableCatalog {
                    id: &selectable_catalog.request.path.id,
                    name: &selectable_catalog.catalog,
                    r#type: &selectable_catalog.request.path.r#type,
                    addon: model::DescriptorPreview {
                        manifest: model::ManifestPreview {
                            id: &addon.manifest.id,
                            name: &addon.manifest.name,
                        },
                    },
                    selected: &selectable_catalog.selected,
                    deep_links: DiscoverDeepLinks::from(&selectable_catalog.request)
                        .into_web_deep_links(),
                })
                .collect(),
            extra: discover
                .selectable
                .extra
                .iter()
                .map(|selectable_extra| model::SelectableExtra {
                    name: &selectable_extra.name,
                    is_required: &selectable_extra.is_required,
                    options: selectable_extra
                        .options
                        .iter()
                        .map(|option| model::SelectableExtraOption {
                            value: &option.value,
                            selected: &option.selected,
                            deep_links: DiscoverDeepLinks::from(&option.request)
                                .into_web_deep_links(),
                        })
                        .collect(),
                })
                .collect(),
            next_page: discover.selectable.next_page.is_some(),
        },
        catalog: (!discover.catalog.is_empty()).as_option().map(|_| {
            let first_page = discover
                .catalog
                .first()
                .expect("discover catalog first page");
            model::ResourceLoadable {
                content: match &first_page.content {
                    Some(Loadable::Ready(_)) => Loadable::Ready(
                        discover
                            .catalog
                            .iter()
                            .filter_map(|page| page.content.as_ref())
                            .filter_map(|page_content| page_content.ready())
                            .flat_map(|meta_items| {
                                meta_items.iter().map(|meta_item| model::MetaItemPreview {
                                    meta_item,
                                    trailer_streams: meta_item
                                        .trailer_streams
                                        .iter()
                                        .take(1)
                                        .map(|stream| model::Stream {
                                            stream,
                                            deep_links: StreamDeepLinks::from((
                                                stream,
                                                &streaming_server.base_url,
                                                &ctx.profile.settings,
                                            ))
                                            .into_web_deep_links(),
                                        })
                                        .collect::<Vec<_>>(),
                                    watched: ctx
                                        .library
                                        .items
                                        .get(&meta_item.id)
                                        .map(|library_item| library_item.watched())
                                        .unwrap_or_default(),
                                    in_library: ctx
                                        .library
                                        .items
                                        .get(&meta_item.id)
                                        .map(|library_item| !library_item.removed)
                                        .unwrap_or_default(),
                                    deep_links: MetaItemDeepLinks::from((
                                        meta_item,
                                        &first_page.request,
                                    ))
                                    .into_web_deep_links(),
                                })
                            })
                            // it is possible that they are duplicates returned in 2 different pages
                            // so we deduplicate all the results at once
                            .unique_by(|meta| &meta.meta_item.id)
                            .collect::<Vec<_>>(),
                    ),
                    Some(Loadable::Loading) | None => Loadable::Loading,
                    Some(Loadable::Err(error)) => Loadable::Err(error.to_string()),
                },
                installed: ctx
                    .profile
                    .addons
                    .iter()
                    .any(|addon| addon.transport_url == first_page.request.base),
            }
        }),
    })
    .expect("JsValue from Discover model::CatalogWithFilters")
}
