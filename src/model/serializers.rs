use crate::env::WebEnv;
use crate::model::deep_links::{DiscoverDeepLinks, MetaItemDeepLinks, StreamDeepLinks};
use serde::Serialize;
use stremio_core::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME};
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::resource::{MetaItemPreview, Stream};
use wasm_bindgen::JsValue;

pub fn serialize_discover(
    discover: &CatalogWithFilters<MetaItemPreview>,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableExtraOption<'a> {
        value: &'a Option<String>,
        selected: &'a bool,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableExtra<'a> {
        name: &'a String,
        is_required: &'a bool,
        options: Vec<_SelectableExtraOption<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableCatalog<'a> {
        catalog: &'a String,
        addon_name: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectableType<'a> {
        r#type: &'a String,
        selected: &'a bool,
        request: &'a ResourceRequest,
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _SelectablePage {
        deep_links: DiscoverDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        catalogs: Vec<_SelectableCatalog<'a>>,
        extra: Vec<_SelectableExtra<'a>>,
        prev_page: Option<_SelectablePage>,
        next_page: Option<_SelectablePage>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _Stream<'a> {
        #[serde(flatten)]
        stream: &'a Stream,
        deep_links: StreamDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _MetaItemPreview<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItemPreview,
        trailer_streams: Vec<_Stream<'a>>,
        in_library: bool,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _ResourceLoadable<'a> {
        content: Loadable<Vec<_MetaItemPreview<'a>>, &'a ResourceError>,
        addon_name: Option<&'a String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct _CatalogWithFilters<'a> {
        selected: &'a Option<CatalogWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Option<_ResourceLoadable<'a>>,
        default_request: Option<&'a ResourceRequest>,
        page: u32,
    }
    JsValue::from_serde(&_CatalogWithFilters {
        selected: &discover.selected,
        selectable: _Selectable {
            types: discover
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    request: &selectable_type.request,
                    deep_links: DiscoverDeepLinks::from(&selectable_type.request),
                })
                .collect(),
            catalogs: discover
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| _SelectableCatalog {
                    catalog: &selectable_catalog.catalog,
                    addon_name: &selectable_catalog.addon_name,
                    selected: &selectable_catalog.selected,
                    request: &selectable_catalog.request,
                    deep_links: DiscoverDeepLinks::from(&selectable_catalog.request),
                })
                .collect(),
            extra: discover
                .selectable
                .extra
                .iter()
                .map(|selectable_extra| _SelectableExtra {
                    name: &selectable_extra.name,
                    is_required: &selectable_extra.is_required,
                    options: selectable_extra
                        .options
                        .iter()
                        .map(|option| _SelectableExtraOption {
                            value: &option.value,
                            selected: &option.selected,
                            deep_links: DiscoverDeepLinks::from(&option.request),
                        })
                        .collect(),
                })
                .collect(),
            prev_page: discover
                .selectable
                .prev_page
                .as_ref()
                .map(|prev_page| _SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&prev_page.request),
                }),
            next_page: discover
                .selectable
                .next_page
                .as_ref()
                .map(|next_page| _SelectablePage {
                    deep_links: DiscoverDeepLinks::from(&next_page.request),
                }),
        },
        catalog: discover.catalog.as_ref().map(|catalog| _ResourceLoadable {
            content: match &catalog.content {
                Loadable::Ready(meta_items) => Loadable::Ready(
                    meta_items
                        .iter()
                        .map(|meta_item| _MetaItemPreview {
                            meta_item,
                            trailer_streams: meta_item
                                .trailer_streams
                                .iter()
                                .map(|stream| _Stream {
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
