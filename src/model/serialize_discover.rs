use crate::model::deep_links::{DiscoverDeepLinks, MetaItemDeepLinks, StreamDeepLinks};
use serde::Serialize;
use stremio_core::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME};
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::common::Loadable;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::MetaItemPreview;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManifestPreview<'a> {
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
    pub struct SelectablePage {
        pub deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub catalogs: Vec<SelectableCatalog<'a>>,
        pub extra: Vec<SelectableExtra<'a>>,
        pub prev_page: Option<SelectablePage>,
        pub next_page: Option<SelectablePage>,
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
        pub default_request: Option<&'a ResourceRequest>,
        pub page: u32,
    }
}

pub fn serialize_discover(discover: &CatalogWithFilters<MetaItemPreview>, ctx: &Ctx) -> JsValue {
    JsValue::from_serde(&model::CatalogWithFilters {
        selected: &discover.selected,
        selectable: model::Selectable {
            types: discover
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: DiscoverDeepLinks::from(&selectable_type.request),
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
                    addon: model::DescriptorPreview {
                        manifest: model::ManifestPreview {
                            name: &addon.manifest.name,
                        },
                    },
                    selected: &selectable_catalog.selected,
                    deep_links: DiscoverDeepLinks::from(&selectable_catalog.request),
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
                            deep_links: DiscoverDeepLinks::from(&option.request),
                        })
                        .collect(),
                })
                .collect(),
            prev_page: discover.selectable.prev_page.as_ref().map(|prev_page| {
                model::SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&prev_page.request),
                }
            }),
            next_page: discover.selectable.next_page.as_ref().map(|next_page| {
                model::SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&next_page.request),
                }
            }),
        },
        catalog: discover
            .catalog
            .as_ref()
            .map(|catalog| model::ResourceLoadable {
                content: match &catalog.content {
                    Some(Loadable::Ready(meta_items)) => Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| model::MetaItemPreview {
                                meta_item,
                                trailer_streams: meta_item
                                    .trailer_streams
                                    .iter()
                                    .take(1)
                                    .map(|stream| model::Stream {
                                        stream,
                                        deep_links: StreamDeepLinks::from(stream),
                                    })
                                    .collect::<Vec<_>>(),
                                in_library: ctx
                                    .library
                                    .items
                                    .get(&meta_item.id)
                                    .map(|library_item| !library_item.removed)
                                    .unwrap_or_default(),
                                deep_links: MetaItemDeepLinks::from((meta_item, &catalog.request)),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    Some(Loadable::Loading) | None => Loadable::Loading,
                    Some(Loadable::Err(error)) => Loadable::Err(error.to_string()),
                },
                installed: ctx
                    .profile
                    .addons
                    .iter()
                    .any(|addon| addon.transport_url == catalog.request.base),
            }),
        default_request: discover
            .selectable
            .types
            .first()
            .map(|first_type| &first_type.request),
        page: discover
            .selected
            .as_ref()
            .and_then(|selected| {
                selected
                    .request
                    .path
                    .get_extra_first_value(SKIP_EXTRA_NAME)
                    .and_then(|value| value.parse::<u32>().ok())
                    .map(|skip| 1 + skip / CATALOG_PAGE_SIZE as u32)
            })
            .unwrap_or(1),
    })
    .unwrap()
}
