#[cfg(feature = "wasm")]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::JsValue};

pub use model::*;

mod model {
    use serde::Serialize;
    use url::Url;

    use stremio_core::{
        models::{board_catalogs::Selected, common::Loadable},
        types::addon::ResourceRequest,
    };

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
    pub struct Preview<'a> {
        pub background: &'a Url,
        pub logo: Option<&'a Url>,
        pub name: &'a str,
        pub count: usize,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BoardCatalog<'a> {
        pub id: &'a str,
        pub name: &'a str,
        pub r#type: &'a str,
        pub addon: DescriptorPreview<'a>,
        pub request: &'a ResourceRequest,
        pub selected: Option<usize>,
        pub default: bool,
        pub content: Option<Loadable<Option<Preview<'a>>, String>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BoardCatalogs<'a> {
        pub selected: &'a Option<Selected>,
        pub catalogs: Vec<BoardCatalog<'a>>,
        pub spotlight: Option<Preview<'a>>,
    }
}

#[cfg(feature = "wasm")]
fn preview<'a>(
    items: &[(
        &'a stremio_core::types::addon::ResourceRequest,
        &'a stremio_core::types::resource::MetaItemPreview,
    )],
) -> Option<model::Preview<'a>> {
    items.first().and_then(|(_, item)| {
        item.background.as_ref().map(|background| model::Preview {
            background,
            logo: item.logo.as_ref(),
            name: &item.name,
            count: items.len(),
        })
    })
}

#[cfg(feature = "wasm")]
pub fn serialize_board_catalogs(
    board_catalogs: &stremio_core::models::board_catalogs::BoardCatalogs,
    ctx: &stremio_core::models::ctx::Ctx,
) -> JsValue {
    use stremio_core::models::common::{
        board_plan, board_spotlight_requests, default_spotlight_requests, spotlight_items, Loadable,
    };

    let spotlight_requests = board_spotlight_requests(&ctx.profile);
    let catalogs = board_plan(&ctx.profile);
    let default_requests = default_spotlight_requests(&catalogs);
    <JsValue as JsValueSerdeExt>::from_serde(&model::BoardCatalogs {
        selected: &board_catalogs.selected,
        catalogs: board_catalogs
            .catalogs
            .iter()
            .filter_map(|page| {
                catalogs
                    .iter()
                    .find(|(_, _, request)| *request == page.request)
                    .map(|(addon, catalog, _)| (addon, catalog, page))
            })
            .map(|(addon, catalog, page)| model::BoardCatalog {
                id: &catalog.id,
                name: catalog.name.as_deref().unwrap_or(&addon.manifest.name),
                r#type: &catalog.r#type,
                addon: model::DescriptorPreview {
                    manifest: model::ManifestPreview {
                        id: &addon.manifest.id,
                        name: &addon.manifest.name,
                    },
                },
                request: &page.request,
                selected: spotlight_requests
                    .iter()
                    .position(|request| *request == page.request),
                default: default_requests.contains(&page.request),
                content: page.content.as_ref().map(|content| match content {
                    Loadable::Ready(_) => {
                        Loadable::Ready(preview(&spotlight_items(std::iter::once(page))))
                    }
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error.to_string()),
                }),
            })
            .collect(),
        spotlight: preview(&spotlight_items(spotlight_requests.iter().filter_map(
            |request| {
                board_catalogs
                    .catalogs
                    .iter()
                    .find(|page| page.request == *request)
            },
        ))),
    })
    .expect("JsValue from BoardCatalogs")
}
