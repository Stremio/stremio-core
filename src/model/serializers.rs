use crate::env::WebEnv;
use crate::model::deep_links::{
    LibraryItemDeepLinks, MetaCatalogResourceDeepLinks, MetaItemDeepLinks,
};
use serde::Serialize;
use stremio_core::models::catalogs_with_extra::{
    CatalogsWithExtra, Selected as CatalogsWithExtraSelected,
};
use stremio_core::models::common::{Loadable, ResourceError};
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::types::addon::ResourceRequest;
use stremio_core::types::library::LibraryItem;
use stremio_core::types::resource::MetaItemPreview;

pub fn serialize_catalogs_with_extra<'a>(
    catalogs_with_extra: &'a CatalogsWithExtra,
    ctx: &'a Ctx<WebEnv>,
) -> Box<dyn erased_serde::Serialize + 'a> {
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
        origin_name: &'a str,
        deep_links: MetaCatalogResourceDeepLinks,
    }
    #[derive(Serialize)]
    struct _CatalogsWithExtra<'a> {
        selected: &'a Option<CatalogsWithExtraSelected>,
        catalog_resources: Vec<_ResourceLoadable<'a>>,
    }
    Box::new(_CatalogsWithExtra {
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
                origin_name: ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == catalog_resource.request.base)
                    .map(|addon| addon.manifest.name.as_ref())
                    .unwrap_or_else(|| catalog_resource.request.base.as_str()),
                deep_links: MetaCatalogResourceDeepLinks::from(catalog_resource),
            })
            .collect::<Vec<_>>(),
    })
}

pub fn serialize_continue_watching_preview<'a>(
    continue_watching_preview: &'a ContinueWatchingPreview,
) -> Box<dyn erased_serde::Serialize + 'a> {
    #[derive(Serialize)]
    struct _LibraryItem<'a> {
        #[serde(flatten)]
        library_item: &'a LibraryItem,
        deep_links: LibraryItemDeepLinks,
    }
    #[derive(Serialize)]
    struct _ContinueWatchingPreview<'a> {
        library_items: Vec<_LibraryItem<'a>>,
    }
    Box::new(_ContinueWatchingPreview {
        library_items: continue_watching_preview
            .library_items
            .iter()
            .map(|library_item| _LibraryItem {
                library_item,
                deep_links: LibraryItemDeepLinks::from(library_item),
            })
            .collect::<Vec<_>>(),
    })
}
