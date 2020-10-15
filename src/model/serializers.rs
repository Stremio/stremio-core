use crate::env::WebEnv;
use crate::model::deep_links::{
    ContinueWatchingDeepLinks, LibraryItemDeepLinks, MetaCatalogResourceDeepLinks,
    MetaItemDeepLinks,
};
use serde::Serialize;
use stremio_core::models::catalog_with_filters::{
    CatalogWithFilters, Selected as CatalogWithFiltersSelected,
};
use stremio_core::models::catalogs_with_extra::{
    CatalogsWithExtra, Selected as CatalogsWithExtraSelected,
};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::{Descriptor, ExtraExt, ExtraValue, ResourceRequest};
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
        addon: Option<&'a Descriptor>,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _CatalogsWithExtra<'a> {
        selected: &'a Option<CatalogsWithExtraSelected>,
        catalog_resources: Vec<_ResourceLoadable<'a>>,
    }
    JsValue::from_serde(&_CatalogsWithExtra {
        selected: &catalogs_with_extra.selected,
        catalog_resources: catalogs_with_extra
            .catalog_resources
            .iter()
            .map(|catalog_resource| _ResourceLoadable {
                request: &catalog_resource.request,
                content: match &catalog_resource.content {
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
                addon: ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog_resource.request.base),
                deep_links: MetaCatalogResourceDeepLinks::from(&catalog_resource.request),
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
        deep_links: ContinueWatchingDeepLinks,
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
        deep_links: ContinueWatchingDeepLinks::default(),
    })
    .unwrap()
}

pub fn serialize_discover(
    discover: &CatalogWithFilters<MetaItemPreview>,
    ctx: &Ctx<WebEnv>,
) -> JsValue {
    #[derive(Serialize)]
    struct _ExtraOption<'a> {
        value: Option<&'a String>,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _SelectableType<'a> {
        name: &'a String,
        request: &'a ResourceRequest,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _SelectableCatalog<'a> {
        name: &'a String,
        request: &'a ResourceRequest,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _ExtraProp<'a> {
        name: &'a String,
        is_required: &'a bool,
        options: Vec<_ExtraOption<'a>>,
    }
    #[derive(Serialize)]
    struct _Selectable<'a> {
        types: Vec<_SelectableType<'a>>,
        catalogs: Vec<_SelectableCatalog<'a>>,
        extra: Vec<_ExtraProp<'a>>,
        has_prev_page: &'a bool,
        has_next_page: &'a bool,
    }
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
        addon: Option<&'a Descriptor>,
    }
    #[derive(Serialize)]
    struct _CatalogWithFilters<'a> {
        selected: &'a Option<CatalogWithFiltersSelected>,
        selectable: _Selectable<'a>,
        catalog: Option<_ResourceLoadable<'a>>,
    }
    JsValue::from_serde(&_CatalogWithFilters {
        selected: &discover.selected,
        selectable: _Selectable {
            types: discover
                .selectable
                .types
                .iter()
                .map(|selectable_type| _SelectableType {
                    name: &selectable_type.name,
                    request: &selectable_type.request,
                    deep_links: MetaCatalogResourceDeepLinks::from(&selectable_type.request),
                })
                .collect(),
            catalogs: discover
                .selectable
                .catalogs
                .iter()
                .map(|selectable_catalog| _SelectableCatalog {
                    name: &selectable_catalog.name,
                    request: &selectable_catalog.request,
                    deep_links: MetaCatalogResourceDeepLinks::from(&selectable_catalog.request),
                })
                .collect(),
            extra: match &discover.selected {
                Some(selected) => discover
                    .selectable
                    .extra
                    .iter()
                    .map(|extra_prop| _ExtraProp {
                        name: &extra_prop.name,
                        is_required: &extra_prop.is_required,
                        options: match &extra_prop.options {
                            Some(options) => {
                                let none_option = if !extra_prop.is_required {
                                    Some(_ExtraOption {
                                        value: None,
                                        deep_links: MetaCatalogResourceDeepLinks::from((
                                            &selected.request.base,
                                            &selected.request.path.type_,
                                            &selected.request.path.id,
                                            selected
                                                .request
                                                .path
                                                .extra
                                                .extend_one_ref(&extra_prop, None),
                                        )),
                                    })
                                } else {
                                    None
                                };
                                let options = options.iter().map(|value| _ExtraOption {
                                    value: Some(value),
                                    deep_links: MetaCatalogResourceDeepLinks::from((
                                        &selected.request.base,
                                        &selected.request.path.type_,
                                        &selected.request.path.id,
                                        selected.request.path.extra.extend_one_ref(
                                            &extra_prop,
                                            Some(&ExtraValue {
                                                name: extra_prop.name.to_owned(),
                                                value: value.to_owned(),
                                            }),
                                        ),
                                    )),
                                });
                                none_option.into_iter().chain(options).collect()
                            }
                            _ => vec![],
                        },
                    })
                    .collect(),
                _ => vec![],
            },
            has_prev_page: &discover.selectable.has_prev_page,
            has_next_page: &discover.selectable.has_next_page,
        },
        catalog: discover.catalog.as_ref().map(|catalog| _ResourceLoadable {
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
            addon: ctx
                .profile
                .addons
                .iter()
                .find(|addon| addon.transport_url == catalog.request.base),
        }),
    })
    .unwrap()
}
