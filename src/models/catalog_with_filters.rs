use crate::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME, TYPE_PRIORITIES};
use crate::models::common::{
    compare_with_priorities, eq_update, resource_update_with_vector_content, Loadable,
    ResourceAction, ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{
    DescriptorPreview, ExtraProp, ExtraValue, Manifest, ManifestCatalog, ResourceRef,
    ResourceRequest, ResourceResponse,
};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub enum SelectablePriority {
    Type,
    Catalog,
}

pub trait CatalogResourceAdapter {
    fn resource_name() -> &'static str;
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog];
    fn selectable_priority() -> SelectablePriority;
    fn catalog_page_size() -> Option<usize>;
}

impl CatalogResourceAdapter for MetaItemPreview {
    fn resource_name() -> &'static str {
        "catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.catalogs
    }
    fn selectable_priority() -> SelectablePriority {
        SelectablePriority::Type
    }
    fn catalog_page_size() -> Option<usize> {
        Some(CATALOG_PAGE_SIZE)
    }
}

impl CatalogResourceAdapter for DescriptorPreview {
    fn resource_name() -> &'static str {
        "addon_catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.addon_catalogs
    }
    fn selectable_priority() -> SelectablePriority {
        SelectablePriority::Catalog
    }
    fn catalog_page_size() -> Option<usize> {
        None
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub request: ResourceRequest,
}

#[derive(Clone, PartialEq, Serialize)]
pub struct SelectableCatalog {
    pub name: String,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableType {
    pub name: String,
    pub request: ResourceRequest,
}

#[derive(Default, PartialEq, Serialize)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub catalogs: Vec<SelectableCatalog>,
    pub extra: Vec<ExtraProp>,
    pub has_prev_page: bool,
    pub has_next_page: bool,
}

#[derive(Default, Serialize)]
pub struct CatalogWithFilters<T> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Option<ResourceLoadable<Vec<T>>>,
}

impl<T: CatalogResourceAdapter> CatalogWithFilters<T> {
    pub fn new(profile: &Profile) -> (Self, Effects) {
        let catalog = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update::<T>(&mut selectable, &catalog, &profile);
        (
            CatalogWithFilters {
                selectable,
                catalog,
                selected: None,
            },
            effects.unchanged(),
        )
    }
}

impl<E, T> UpdateWithCtx<Ctx<E>> for CatalogWithFilters<T>
where
    E: Env + 'static,
    T: CatalogResourceAdapter + PartialEq,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let catalog_effects = resource_update_with_vector_content::<E, _>(
                    &mut self.catalog,
                    ResourceAction::ResourceRequested {
                        request: &selected.request,
                    },
                );
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.catalog, &ctx.profile);
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalog_effects = eq_update(&mut self.catalog, None);
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.catalog, &ctx.profile);
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let catalog_effects = resource_update_with_vector_content::<E, _>(
                    &mut self.catalog,
                    ResourceAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &T::catalog_page_size(),
                    },
                );
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.catalog, &ctx.profile);
                catalog_effects.join(selectable_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                selectable_update(&mut self.selectable, &self.catalog, &ctx.profile)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn selectable_update<T: CatalogResourceAdapter>(
    selectable: &mut Selectable,
    catalog: &Option<ResourceLoadable<Vec<T>>>,
    profile: &Profile,
) -> Effects {
    let selectable_catalogs = profile
        .addons
        .iter()
        .flat_map(|addon| {
            T::catalogs_from_manifest(&addon.manifest)
                .iter()
                .map(move |manifest_catalog| (addon, manifest_catalog))
        })
        .filter_map(|(addon, manifest_catalog)| {
            manifest_catalog
                .default_required_extra()
                .map(|default_required_extra| SelectableCatalog {
                    name: manifest_catalog
                        .name
                        .as_ref()
                        .unwrap_or(&manifest_catalog.id)
                        .to_owned(),
                    request: ResourceRequest {
                        base: addon.transport_url.to_owned(),
                        path: ResourceRef::with_extra(
                            T::resource_name(),
                            &manifest_catalog.type_,
                            &manifest_catalog.id,
                            &default_required_extra,
                        ),
                    },
                })
        })
        .dedup_by(|a, b| a.request == b.request)
        .collect::<Vec<_>>();
    let (selectable_types, selectable_catalogs) = match T::selectable_priority() {
        SelectablePriority::Type => {
            let selectable_types = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| &selectable_catalog.request.path.type_)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.request.path.type_.to_owned(),
                    request: selectable_catalog.request.to_owned(),
                })
                .collect::<Vec<_>>();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match catalog {
                    Some(catalog) => {
                        selectable_catalog.request.path.type_ == catalog.request.path.type_
                    }
                    None => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            (selectable_types, selectable_catalogs)
        }
        SelectablePriority::Catalog => {
            let selectable_types = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match catalog {
                    Some(catalog) => {
                        selectable_catalog.request.path.id == catalog.request.path.id
                            && selectable_catalog.request.base == catalog.request.base
                    }
                    _ => true,
                })
                .unique_by(|selectable_catalog| &selectable_catalog.request.path.type_)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.request.path.type_.to_owned(),
                    request: selectable_catalog.request.to_owned(),
                })
                .collect::<Vec<_>>();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| {
                    (
                        &selectable_catalog.request.base,
                        &selectable_catalog.request.path.id,
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            (selectable_types, selectable_catalogs)
        }
    };
    let selectable_types = selectable_types
        .into_iter()
        .sorted_by(|a, b| {
            compare_with_priorities(a.name.as_ref(), b.name.as_ref(), &*TYPE_PRIORITIES)
        })
        .rev()
        .collect::<Vec<_>>();
    let (selectable_extra, has_prev_page, has_next_page) = match catalog {
        Some(catalog) => profile
            .addons
            .iter()
            .find(|addon| addon.transport_url == catalog.request.base)
            .iter()
            .flat_map(|addon| T::catalogs_from_manifest(&addon.manifest))
            .find(|ManifestCatalog { id, type_, .. }| {
                *id == catalog.request.path.id && *type_ == catalog.request.path.type_
            })
            .map(|manifest_catalog| {
                let selectable_extra = manifest_catalog
                    .extra
                    .iter()
                    .filter(|extra| extra.options.iter().flatten().next().is_some())
                    .map(|extra| extra.into_owned())
                    .collect();
                let skip_supported = manifest_catalog
                    .extra
                    .iter()
                    .any(|extra| extra.name == SKIP_EXTRA_NAME);
                let first_page_requested = catalog
                    .request
                    .path
                    .get_extra_first_value(SKIP_EXTRA_NAME)
                    .and_then(|value| value.parse::<u32>().ok())
                    .map(|skip| skip == 0)
                    .unwrap_or(true);
                let last_page_requested = match &catalog.content {
                    Loadable::Ready(content) => match T::catalog_page_size() {
                        Some(catalog_page_size) => content.len() < catalog_page_size,
                        None => true,
                    },
                    Loadable::Err(_) => true,
                    Loadable::Loading => true,
                };
                let has_prev_page = skip_supported && !first_page_requested;
                let has_next_page = skip_supported && !last_page_requested;
                (selectable_extra, has_prev_page, has_next_page)
            })
            .unwrap_or_default(),
        _ => Default::default(),
    };
    let next_selectable = Selectable {
        types: selectable_types,
        catalogs: selectable_catalogs,
        extra: selectable_extra,
        has_prev_page,
        has_next_page,
    };
    if *selectable != next_selectable {
        *selectable = next_selectable;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
