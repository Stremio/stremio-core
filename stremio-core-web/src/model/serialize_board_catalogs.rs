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
        pub selected: bool,
        pub default: bool,
        pub content: Option<Loadable<Option<Preview<'a>>, String>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BoardCatalogs<'a> {
        pub selected: &'a Option<Selected>,
        pub catalogs: Vec<BoardCatalog<'a>>,
    }
}

#[cfg(feature = "wasm")]
pub fn serialize_board_catalogs(
    board_catalogs: &stremio_core::models::board_catalogs::BoardCatalogs,
    ctx: &stremio_core::models::ctx::Ctx,
) -> JsValue {
    use stremio_core::models::{
        catalogs_with_extra::{
            board_catalogs as eligible_catalogs, board_spotlight_request, spotlight_items,
        },
        common::Loadable,
    };

    let spotlight_request = board_spotlight_request(&ctx.profile);
    let catalogs = eligible_catalogs(&ctx.profile.addons);
    let default_request = catalogs.first().map(|(_, _, request)| request.to_owned());
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
                selected: spotlight_request.as_ref() == Some(&page.request),
                default: default_request.as_ref() == Some(&page.request),
                content: page.content.as_ref().map(|content| match content {
                    Loadable::Ready(items) => {
                        let items = spotlight_items(items).collect::<Vec<_>>();
                        Loadable::Ready(items.first().and_then(|item| {
                            item.background.as_ref().map(|background| model::Preview {
                                background,
                                logo: item.logo.as_ref(),
                                name: &item.name,
                                count: items.len(),
                            })
                        }))
                    }
                    Loadable::Loading => Loadable::Loading,
                    Loadable::Err(error) => Loadable::Err(error.to_string()),
                }),
            })
            .collect(),
    })
    .expect("JsValue from BoardCatalogs")
}
