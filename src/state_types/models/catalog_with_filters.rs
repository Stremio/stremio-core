use crate::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME};
use crate::state_types::models::common::{
    eq_update, resource_update_with_vector_content, validate_extra, ResourceAction,
    ResourceContent, ResourceLoadable,
};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{
    Descriptor, DescriptorPreview, Manifest, ManifestCatalog, ManifestExtraProp, ResourceRef,
    ResourceRequest, ResourceResponse,
};
use crate::types::MetaPreview;
use derivative::Derivative;
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

impl CatalogResourceAdapter for MetaPreview {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub request: ResourceRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectableCatalog {
    pub name: String,
    pub addon_name: String,
    pub request: ResourceRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectableType {
    pub name: String,
    pub request: ResourceRequest,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub catalogs: Vec<SelectableCatalog>,
    pub extra: Vec<ManifestExtraProp>,
    pub has_prev_page: bool,
    pub has_next_page: bool,
}

#[derive(Derivative, Debug, Clone, Serialize)]
#[derivative(Default(bound = ""))]
pub struct CatalogWithFilters<T> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog_resource: Option<ResourceLoadable<Vec<T>>>,
}

impl<Env, T> UpdateWithCtx<Ctx<Env>> for CatalogWithFilters<T>
where
    Env: Environment + 'static,
    T: CatalogResourceAdapter,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(selected))) => {
                let selected = Selected {
                    request: ResourceRequest {
                        base: selected.request.base.to_owned(),
                        path: ResourceRef {
                            resource: selected.request.path.resource.to_owned(),
                            type_name: selected.request.path.type_name.to_owned(),
                            id: selected.request.path.id.to_owned(),
                            extra: validate_extra(
                                &selected.request.path.extra,
                                &T::catalog_page_size(),
                            ),
                        },
                    },
                };
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let catalog_effects = resource_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceRequested {
                        request: &selected.request,
                    },
                );
                let selectable_effects = selectable_update(
                    &mut self.selectable,
                    &self.catalog_resource,
                    &ctx.profile.content().addons,
                );
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalog_effects = eq_update(&mut self.catalog_resource, None);
                let selectable_effects = selectable_update(
                    &mut self.selectable,
                    &self.catalog_resource,
                    &ctx.profile.content().addons,
                );
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let catalog_effects = resource_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &T::catalog_page_size(),
                    },
                );
                let selectable_effects = selectable_update(
                    &mut self.selectable,
                    &self.catalog_resource,
                    &ctx.profile.content().addons,
                );
                catalog_effects.join(selectable_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => selectable_update(
                &mut self.selectable,
                &self.catalog_resource,
                &ctx.profile.content().addons,
            ),
            _ => Effects::none().unchanged(),
        }
    }
}

fn selectable_update<T: CatalogResourceAdapter>(
    selectable: &mut Selectable,
    catalog_resource: &Option<ResourceLoadable<Vec<T>>>,
    addons: &[Descriptor],
) -> Effects {
    let selectable_catalogs = addons
        .iter()
        .flat_map(|addon| {
            T::catalogs_from_manifest(&addon.manifest)
                .iter()
                .map(move |manifest_catalog| (addon, manifest_catalog))
        })
        .filter_map(|(addon, manifest_catalog)| {
            manifest_catalog
                .extra_iter()
                .filter(|extra| extra.is_required)
                .map(|extra| {
                    extra
                        .options
                        .as_ref()
                        .and_then(|options| options.first())
                        .map(|first_option| (extra.name.to_owned(), first_option.to_owned()))
                })
                .collect::<Option<Vec<_>>>()
                .map(|default_required_extra| SelectableCatalog {
                    name: manifest_catalog
                        .name
                        .as_ref()
                        .unwrap_or(&manifest_catalog.id)
                        .to_owned(),
                    addon_name: addon.manifest.name.to_owned(),
                    request: ResourceRequest {
                        base: addon.transport_url.to_owned(),
                        path: ResourceRef::with_extra(
                            T::resource_name(),
                            &manifest_catalog.type_name,
                            &manifest_catalog.id,
                            &default_required_extra,
                        ),
                    },
                })
        }) // TODO this .collect.iter should be removed
        // .cloned()
        // .unique_by(|selectable_catalog| &selectable_catalog.request)
        .collect::<Vec<SelectableCatalog>>();
    let (selectable_types, selectable_catalogs) = match T::selectable_priority() {
        SelectablePriority::Type => {
            let selectable_types = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| &selectable_catalog.request.path.type_name)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.request.path.type_name.to_owned(),
                    request: selectable_catalog.request.to_owned(),
                })
                .collect::<Vec<_>>();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match catalog_resource {
                    Some(catalog_resource) => selectable_catalog
                        .request
                        .path
                        .type_name
                        .eq(&catalog_resource.request.path.type_name),
                    None => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            (selectable_types, selectable_catalogs)
        }
        SelectablePriority::Catalog => {
            let selectable_types = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match catalog_resource {
                    Some(catalog_resource) => selectable_catalog
                        .request
                        .path
                        .id
                        .eq(&catalog_resource.request.path.id),
                    None => true,
                })
                .unique_by(|selectable_catalog| &selectable_catalog.request.path.type_name)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.request.path.type_name.to_owned(),
                    request: selectable_catalog.request.to_owned(),
                })
                .collect::<Vec<_>>();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| &selectable_catalog.request.path.id)
                .cloned()
                .collect::<Vec<_>>();
            (selectable_types, selectable_catalogs)
        }
    };
    let (selectable_extra, has_prev_page, has_next_page) = match catalog_resource {
        Some(catalog_resource) => addons
            .iter()
            .find(|addon| addon.transport_url.eq(&catalog_resource.request.base))
            .iter()
            .flat_map(|addon| T::catalogs_from_manifest(&addon.manifest))
            .find(|ManifestCatalog { id, type_name, .. }| {
                type_name.eq(&catalog_resource.request.path.type_name)
                    && id.eq(&catalog_resource.request.path.id)
            })
            .map(|manifest_catalog| {
                let selectable_extra = manifest_catalog
                    .extra_iter()
                    .filter(|extra| extra.options.iter().flatten().next().is_some())
                    .map(|extra| extra.into_owned())
                    .collect();
                let skip_supported = manifest_catalog
                    .extra_iter()
                    .any(|extra| extra.name.eq(SKIP_EXTRA_NAME));
                let first_page_requested = catalog_resource
                    .request
                    .path
                    .get_extra_first_val(SKIP_EXTRA_NAME)
                    .and_then(|value| value.parse::<u32>().ok())
                    .map(|skip| skip.eq(&0))
                    .unwrap_or(true);
                let last_page_requested = match &catalog_resource.content {
                    ResourceContent::Ready(content) => match T::catalog_page_size() {
                        Some(catalog_page_size) => content.len() < catalog_page_size,
                        None => true,
                    },
                    ResourceContent::Err(_) => true,
                    ResourceContent::Loading => true,
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
    if next_selectable.ne(selectable) {
        *selectable = next_selectable;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
