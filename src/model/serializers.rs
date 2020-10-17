use crate::env::WebEnv;
use crate::model::deep_links::{
    LibraryDeepLinks, LibraryItemDeepLinks, MetaCatalogResourceDeepLinks, MetaItemDeepLinks,
};
use serde::Serialize;
use stremio_core::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME};
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::catalogs_with_extra::{
    CatalogsWithExtra, Selected as CatalogsWithExtraSelected,
};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::library::LibraryItem;
use stremio_core::types::resource::MetaItemPreview;
use wasm_bindgen::JsValue;

pub fn serialize_catalogs_with_extra(
    catalogs_with_extra: &CatalogsWithExtra,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    struct _MetaItemPreview<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItemPreview,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    struct _ResourceLoadable<'a> {
        request: &'a ResourceRequest,
        content: Loadable<Vec<_MetaItemPreview<'a>>, &'a ResourceError>,
        addon_name: Option<&'a String>,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _CatalogsWithExtra<'a> {
        selected: &'a Option<CatalogsWithExtraSelected>,
        catalogs: Vec<_ResourceLoadable<'a>>,
    }
    JsValue::from_serde(&_CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalogs: catalogs_with_extra
            .catalogs
            .iter()
            .map(|catalog| _ResourceLoadable {
                request: &catalog.request,
                content: match &catalog.content {
                    Loadable::Ready(meta_items) => Loadable::Ready(
                        meta_items
                            .iter()
                            .map(|meta_item| _MetaItemPreview {
                                meta_item,
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
                deep_links: MetaCatalogResourceDeepLinks::from(&catalog.request),
            })
            .collect::<Vec<_>>(),
    })
    .unwrap()
}

pub fn serialize_continue_watching_preview(
    continue_watching_preview: &ContinueWatchingPreview,
) -> JsValue {
    #[derive(Serialize)]
    struct _LibraryItem<'a> {
        #[serde(flatten)]
        library_item: &'a LibraryItem,
        deep_links: LibraryItemDeepLinks,
    }
    #[derive(Serialize)]
    struct _ContinueWatchingPreview<'a> {
        library_items: Vec<_LibraryItem<'a>>,
        deep_links: LibraryDeepLinks,
    }
    JsValue::from_serde(&_ContinueWatchingPreview {
        library_items: continue_watching_preview
            .library_items
            .iter()
            .map(|library_item| _LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect::<Vec<_>>(),
        deep_links: LibraryDeepLinks::from(&"continuewatching".to_owned()),
    })
    .unwrap()
}

pub fn serialize_discover(
    discover: &CatalogWithFilters<MetaItemPreview>,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    struct _SelectableExtraOption<'a> {
        value: &'a Option<String>,
        selected: &'a bool,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _SelectableExtra<'a> {
        name: &'a String,
        is_required: &'a bool,
        options: Vec<_SelectableExtraOption<'a>>,
    }
    #[derive(Serialize)]
    struct _SelectableCatalog<'a> {
        catalog: &'a String,
        addon_name: &'a String,
        selected: &'a bool,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _SelectableType<'a> {
        r#type: &'a String,
        selected: &'a bool,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct SelectablePage {
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        catalogs: Vec<_SelectableCatalog<'a>>,
        extra: Vec<_SelectableExtra<'a>>,
        prev_page: Option<SelectablePage>,
        next_page: Option<SelectablePage>,
    }
    #[derive(Serialize)]
    struct _MetaItemPreview<'a> {
        #[serde(flatten)]
        meta_item: &'a MetaItemPreview,
        deep_links: MetaItemDeepLinks,
    }
    #[derive(Serialize)]
    struct _ResourceLoadable<'a> {
        content: Loadable<Vec<_MetaItemPreview<'a>>, &'a ResourceError>,
        addon_name: Option<&'a String>,
    }
    #[derive(Serialize)]
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
                    deep_links: MetaCatalogResourceDeepLinks::from(&selectable_type.request),
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
                    deep_links: MetaCatalogResourceDeepLinks::from(&selectable_catalog.request),
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
                            deep_links: MetaCatalogResourceDeepLinks::from(&option.request),
                        })
                        .collect(),
                })
                .collect(),
            prev_page: discover
                .selectable
                .prev_page
                .as_ref()
                .map(|request| SelectablePage {
                    deep_links: MetaCatalogResourceDeepLinks::from(request),
                }),
            next_page: discover
                .selectable
                .next_page
                .as_ref()
                .map(|request| SelectablePage {
                    deep_links: MetaCatalogResourceDeepLinks::from(request),
                }),
        },
        catalog: discover.catalog.as_ref().map(|catalog| _ResourceLoadable {
            content: match &catalog.content {
                Loadable::Ready(meta_items) => Loadable::Ready(
                    meta_items
                        .iter()
                        .map(|meta_item| _MetaItemPreview {
                            meta_item,
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
