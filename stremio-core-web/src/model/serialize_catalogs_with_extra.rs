use itertools::Itertools;
use serde::Serialize;
use url::Url;

#[cfg(feature = "wasm")]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::JsValue};

use crate::model::deep_links_ext::DeepLinksExt;

pub use stremio_core::{
    deep_links::{DiscoverDeepLinks, MetaItemDeepLinks, StreamDeepLinks},
    models::{
        catalogs_with_extra::Selected,
        common::{spotlight_items, spotlight_page_answered, CatalogPage, Loadable},
        ctx::Ctx,
        streaming_server::StreamingServer,
    },
    types::{addon::ResourceRequest, resource::PosterShape},
};

const CATALOG_ITEMS_COUNT: usize = 10;

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
    pub poster_shape: &'a PosterShape,
    pub trailer_streams: Vec<Stream<'a>>,
    pub watched: bool,
    pub in_library: bool,
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
    pub spotlight: Option<Loadable<Vec<MetaItemPreview<'a>>, String>>,
}

fn catalog_items(
    meta_items: &[stremio_core::types::resource::MetaItemPreview],
) -> Vec<&stremio_core::types::resource::MetaItemPreview> {
    meta_items
        .iter()
        .unique_by(|meta_item| &meta_item.id)
        .take(CATALOG_ITEMS_COUNT)
        .collect()
}

fn serialize_item<'a>(
    meta_item: &'a stremio_core::types::resource::MetaItemPreview,
    poster_shape: &'a PosterShape,
    request: &ResourceRequest,
    ctx: &'a Ctx,
    streaming_server_url: Option<&Url>,
) -> MetaItemPreview<'a> {
    MetaItemPreview {
        meta_item,
        poster_shape,
        trailer_streams: meta_item
            .trailer_streams
            .iter()
            .take(1)
            .map(|stream| Stream {
                stream,
                deep_links: StreamDeepLinks::from((
                    stream,
                    streaming_server_url,
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
        deep_links: MetaItemDeepLinks::from((meta_item, request)).into_web_deep_links(),
    }
}

fn serialize_page<'a>(
    catalog: &'a CatalogPage<stremio_core::types::resource::MetaItemPreview>,
    ctx: &'a Ctx,
    streaming_server_url: Option<&Url>,
) -> Option<ResourceLoadable<'a>> {
    ctx.profile
        .addons
        .iter()
        .filter(|addon| addon.transport_url == catalog.request.base)
        .find_map(|addon| {
            addon
                .manifest
                .catalogs
                .iter()
                .find(|manifest_catalog| {
                    manifest_catalog.id == catalog.request.path.id
                        && manifest_catalog.r#type == catalog.request.path.r#type
                })
                .map(|manifest_catalog| (addon, manifest_catalog))
        })
        .map(|(addon, manifest_catalog)| ResourceLoadable {
            id: manifest_catalog.id.to_string(),
            name: manifest_catalog
                .name
                .as_ref()
                .unwrap_or(&addon.manifest.name)
                .to_string(),
            r#type: manifest_catalog.r#type.to_string(),
            addon: DescriptorPreview {
                manifest: ManifestPreview {
                    id: &addon.manifest.id,
                    name: &addon.manifest.name,
                },
            },
            content: match &catalog.content {
                Some(Loadable::Ready(meta_items)) => {
                    let poster_shape = meta_items.first().map(|meta_item| &meta_item.poster_shape);
                    Some(Loadable::Ready(
                        catalog_items(meta_items)
                            .into_iter()
                            .map(|meta_item| {
                                serialize_item(
                                    meta_item,
                                    poster_shape.unwrap_or(&meta_item.poster_shape),
                                    &catalog.request,
                                    ctx,
                                    streaming_server_url,
                                )
                            })
                            .collect::<Vec<_>>(),
                    ))
                }
                Some(Loadable::Loading) => Some(Loadable::Loading),
                Some(Loadable::Err(error)) => Some(Loadable::Err(error.to_string())),
                None => None,
            },
            deep_links: DiscoverDeepLinks::from(&catalog.request).into_web_deep_links(),
        })
}

fn serialize_spotlight<'a>(
    pages: &'a [CatalogPage<stremio_core::types::resource::MetaItemPreview>],
    ctx: &'a Ctx,
    streaming_server_url: Option<&Url>,
) -> Option<Loadable<Vec<MetaItemPreview<'a>>, String>> {
    if pages.is_empty() {
        return None;
    }

    let items = spotlight_items(pages.iter());
    if items.is_empty() && !pages.iter().all(spotlight_page_answered) {
        return Some(Loadable::Loading);
    }

    Some(Loadable::Ready(
        items
            .into_iter()
            .map(|(request, meta_item)| {
                serialize_item(
                    meta_item,
                    &meta_item.poster_shape,
                    request,
                    ctx,
                    streaming_server_url,
                )
            })
            .collect::<Vec<_>>(),
    ))
}

impl<'a> CatalogsWithExtra<'a> {
    pub fn new(
        catalogs_with_extra: &'a stremio_core::models::catalogs_with_extra::CatalogsWithExtra,
        ctx: &'a Ctx,
        streaming_server: &StreamingServer,
    ) -> Self {
        Self::with_streaming_server_url(
            catalogs_with_extra,
            ctx,
            streaming_server.base_url.as_ref(),
        )
    }

    fn with_streaming_server_url(
        catalogs_with_extra: &'a stremio_core::models::catalogs_with_extra::CatalogsWithExtra,
        ctx: &'a Ctx,
        streaming_server_url: Option<&Url>,
    ) -> Self {
        Self {
            selected: &catalogs_with_extra.selected,
            catalogs: catalogs_with_extra
                .catalogs
                .iter()
                .filter_map(|catalog| catalog.first())
                .filter_map(|catalog| serialize_page(catalog, ctx, streaming_server_url))
                .collect::<Vec<_>>(),
            spotlight: serialize_spotlight(
                &catalogs_with_extra.spotlight,
                ctx,
                streaming_server_url,
            ),
        }
    }
}

#[cfg(feature = "wasm")]
impl super::SerializeModel<wasm_bindgen::JsValue> for CatalogsWithExtra<'_> {
    type Error = serde_json::Error;

    fn serialize_model(&self) -> Result<wasm_bindgen::JsValue, Self::Error> {
        wasm_bindgen::JsValue::try_from(self)
    }
}

#[cfg(feature = "wasm")]
impl<'a> TryFrom<&CatalogsWithExtra<'a>> for JsValue {
    type Error = serde_json::Error;

    fn try_from(catalogs: &CatalogsWithExtra<'a>) -> Result<Self, Self::Error> {
        <JsValue as JsValueSerdeExt>::from_serde(&catalogs)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};
    use stremio_core::{
        models::{catalogs_with_extra, common},
        types::{notifications::NotificationsBucket, profile::Profile},
    };

    use super::{CatalogsWithExtra, Ctx};

    #[test]
    fn board_rows_keep_core_indexes_when_addons_share_a_url() {
        let addon = |id, transport_url, catalogs: &[&str]| {
            let catalogs = catalogs
                .iter()
                .map(|id| json!({ "id": id, "type": "movie" }))
                .collect::<Vec<_>>();
            serde_json::from_value(json!({
                "transportUrl": transport_url,
                "manifest": {
                    "id": id,
                    "name": id,
                    "version": "1.0.0",
                    "types": ["movie"],
                    "resources": ["catalog"],
                    "catalogs": catalogs,
                },
            }))
            .unwrap()
        };
        let shared_url = "https://shared.example/manifest.json";
        let ctx = Ctx::new(
            Profile {
                addons: vec![
                    addon("first", shared_url, &["first-row"]),
                    addon("second", shared_url, &["second-row"]),
                    addon("last", "https://other.example/manifest.json", &["last-row"]),
                ],
                ..Default::default()
            },
            Default::default(),
            Default::default(),
            Default::default(),
            NotificationsBucket {
                uid: None,
                items: Default::default(),
                last_updated: None,
                created: Default::default(),
            },
            Default::default(),
            Default::default(),
        );
        let board = catalogs_with_extra::CatalogsWithExtra {
            selected: Some(catalogs_with_extra::Selected {
                r#type: None,
                extra: vec![],
            }),
            catalogs: common::board_plan(&ctx.profile)
                .into_iter()
                .map(|(_, _, request)| {
                    vec![common::ResourceLoadable {
                        request,
                        content: None,
                    }]
                })
                .collect(),
            spotlight: vec![],
        };

        let state = to_value(CatalogsWithExtra::with_streaming_server_url(
            &board, &ctx, None,
        ))
        .unwrap();
        let rows = state["catalogs"].as_array().unwrap();
        let row_ids = rows
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(row_ids, ["first-row", "second-row", "last-row"]);
        assert_eq!(rows[1]["addon"]["manifest"]["id"], "second");
    }
}
