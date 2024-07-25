use crate::model::deep_links_ext::DeepLinksExt;
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use stremio_core::deep_links::AddonsDeepLinks;
use stremio_core::models::installed_addons_with_filters::{
    InstalledAddonsRequest, InstalledAddonsWithFilters, Selected,
};
use wasm_bindgen::JsValue;

mod model {
    use stremio_core::types::addon::{DescriptorFlags, ManifestPreview};
    use url::Url;

    use super::*;

    /// Descriptor Preview serializing the [`ManifestPreview`] and
    /// [`DescriptorFlags`] of an addon.
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview {
        pub manifest: ManifestPreview,
        pub transport_url: Url,
        #[serde(default)]
        pub flags: DescriptorFlags,
        /// All addons in this model are installed by default!
        pub installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableType<'a> {
        pub r#type: &'a Option<String>,
        pub selected: &'a bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableCatalog {
        pub name: String,
        pub selected: bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub catalogs: Vec<SelectableCatalog>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InstalledAddonsWithFilters<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Vec<DescriptorPreview>,
    }
}

pub fn serialize_installed_addons(installed_addons: &InstalledAddonsWithFilters) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&model::InstalledAddonsWithFilters {
        selected: &installed_addons.selected,
        selectable: model::Selectable {
            types: installed_addons
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
            catalogs: vec![model::SelectableCatalog {
                name: "Installed".to_owned(),
                selected: installed_addons.selected.is_some(),
                deep_links: AddonsDeepLinks::from(&InstalledAddonsRequest { r#type: None })
                    .into_web_deep_links(),
            }],
        },
        catalog: installed_addons
            .catalog
            .iter()
            .map(|addon| model::DescriptorPreview {
                manifest: (&addon.manifest).into(),
                transport_url: addon.transport_url.clone(),
                flags: addon.flags.clone(),
                installed: true,
            })
            .collect(),
    })
    .expect("JsValue from model::InstalledAddonsWithFilters")
}
