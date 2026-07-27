use crate::constants::{SKIP_EXTRA_PROP, TYPE_PRIORITIES};
use crate::models::common::{
    compare_with_priorities, eq_update, resource_update_with_vector_content, ResourceAction,
    ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCatalogWithFilters, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{
    Descriptor, DescriptorPreview, ExtraExt, Manifest, ManifestCatalog, ResourcePath,
    ResourceRequest, ResourceResponse,
};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use boolinator::Boolinator;
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(PartialEq, Eq)]
pub enum SelectablePriority {
    Type,
    Catalog,
}

pub trait CatalogResourceAdapter {
    fn resource() -> &'static str;
    fn catalogs(manifest: &Manifest) -> &[ManifestCatalog];
    fn selectable_priority() -> SelectablePriority;
}

impl CatalogResourceAdapter for MetaItemPreview {
    fn resource() -> &'static str {
        "catalog"
    }
    fn catalogs(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.catalogs
    }
    fn selectable_priority() -> SelectablePriority {
        SelectablePriority::Type
    }
}

impl CatalogResourceAdapter for DescriptorPreview {
    fn resource() -> &'static str {
        "addon_catalog"
    }
    fn catalogs(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.addon_catalogs
    }
    fn selectable_priority() -> SelectablePriority {
        SelectablePriority::Catalog
    }
}

impl CatalogResourceAdapter for Descriptor {
    fn resource() -> &'static str {
        "addon_catalog"
    }
    fn catalogs(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.addon_catalogs
    }
    fn selectable_priority() -> SelectablePriority {
        SelectablePriority::Catalog
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub request: ResourceRequest,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectableCatalog {
    pub catalog: String,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct SelectableType {
    pub r#type: String,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct SelectableExtraOption {
    pub value: Option<String>,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectableExtra {
    pub name: String,
    pub is_required: bool,
    pub options: Vec<SelectableExtraOption>,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct SelectablePage {
    pub request: ResourceRequest,
}

#[derive(Default, PartialEq, Eq, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub catalogs: Vec<SelectableCatalog>,
    pub extra: Vec<SelectableExtra>,
    pub next_page: Option<SelectablePage>,
}

pub enum CatalogPageRequest {
    First,
    Next,
}

pub type CatalogPage<T> = ResourceLoadable<Vec<T>>;

pub type Catalog<T> = Vec<CatalogPage<T>>;

#[derive(Derivative, Serialize, Clone, Debug)]
#[derivative(Default(bound = ""))]
pub struct CatalogWithFilters<T> {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Catalog<T>,
}

impl<T: CatalogResourceAdapter> CatalogWithFilters<T> {
    pub fn new(profile: &Profile) -> (Self, Effects) {
        let mut model = CatalogWithFilters::<T>::default();
        let selectable_effects = selectable_update(
            &mut model.selectable,
            &model.selected,
            &model.catalog,
            profile,
        )
        .unchanged();
        (model, selectable_effects)
    }
}

impl<E, T> UpdateWithCtx<E> for CatalogWithFilters<T>
where
    E: Env + 'static,
    T: CatalogResourceAdapter + PartialEq,
    Vec<T>: TryFrom<ResourceResponse, Error = derive_more::TryIntoError<ResourceResponse>>,
{
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogWithFilters(selected))) => {
                let selected_effects =
                    selected_update::<T>(&mut self.selected, &self.selectable, selected);
                let catalog_effects = match self.selected.as_ref() {
                    Some(selected) => catalog_update::<E, _>(
                        &mut self.catalog,
                        CatalogPageRequest::First,
                        &selected.request,
                    ),
                    _ => Effects::none().unchanged(),
                };
                let selectable_effects = selectable_update(
                    &mut self.selectable,
                    &self.selected,
                    &self.catalog,
                    &ctx.profile,
                );
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalog_effects = eq_update(&mut self.catalog, vec![]);
                let selectable_effects = selectable_update(
                    &mut self.selectable,
                    &self.selected,
                    &self.catalog,
                    &ctx.profile,
                );
                selected_effects
                    .join(catalog_effects)
                    .join(selectable_effects)
            }
            Msg::Action(Action::CatalogWithFilters(ActionCatalogWithFilters::LoadNextPage)) => {
                match self.selectable.next_page.as_ref() {
                    Some(next_page) => {
                        let catalog_effects = catalog_update::<E, _>(
                            &mut self.catalog,
                            CatalogPageRequest::Next,
                            &next_page.request,
                        );
                        let selectable_effects = selectable_update(
                            &mut self.selectable,
                            &self.selected,
                            &self.catalog,
                            &ctx.profile,
                        );
                        catalog_effects.join(selectable_effects)
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => self
                .catalog
                .iter_mut()
                .find(|page| page.request == *request)
                .map(|page| {
                    resource_update_with_vector_content::<E, _>(
                        page,
                        ResourceAction::ResourceRequestResult { request, result },
                    )
                })
                .map(|catalog_effects| {
                    let selectable_effects = selectable_update(
                        &mut self.selectable,
                        &self.selected,
                        &self.catalog,
                        &ctx.profile,
                    );
                    catalog_effects.join(selectable_effects)
                })
                .unwrap_or_else(|| Effects::none().unchanged()),
            Msg::Internal(Internal::ProfileChanged) => selectable_update(
                &mut self.selectable,
                &self.selected,
                &self.catalog,
                &ctx.profile,
            ),
            Msg::Internal(Internal::LibraryChanged(_)) => Effects::none(),
            _ => Effects::none().unchanged(),
        }
    }
}

fn selected_update<T: CatalogResourceAdapter>(
    selected: &mut Option<Selected>,
    selectable: &Selectable,
    next_selected: &Option<Selected>,
) -> Effects {
    let next_selected = next_selected
        .as_ref()
        .map(|next_selected| {
            let mut next_selected = next_selected.to_owned();
            next_selected.request.path.extra = next_selected
                .request
                .path
                .extra
                .remove_all(&SKIP_EXTRA_PROP);
            next_selected
        })
        .or_else(|| match T::selectable_priority() {
            SelectablePriority::Type => selectable.types.first().map(|selectable_type| Selected {
                request: selectable_type.request.to_owned(),
            }),
            SelectablePriority::Catalog => {
                selectable
                    .catalogs
                    .first()
                    .map(|selectable_catalog| Selected {
                        request: selectable_catalog.request.to_owned(),
                    })
            }
        });
    eq_update(selected, next_selected)
}

fn catalog_update<E, T>(
    catalog: &mut Catalog<T>,
    page_request: CatalogPageRequest,
    request: &ResourceRequest,
) -> Effects
where
    E: Env + 'static,
    T: CatalogResourceAdapter + PartialEq,
    Vec<T>: TryFrom<ResourceResponse, Error = derive_more::TryIntoError<ResourceResponse>>,
{
    let mut page = ResourceLoadable {
        request: request.to_owned(),
        content: None,
    };
    let effects = resource_update_with_vector_content::<E, _>(
        &mut page,
        ResourceAction::ResourceRequested { request },
    );
    match page_request {
        CatalogPageRequest::First => *catalog = vec![page],
        CatalogPageRequest::Next => catalog.extend(vec![page]),
    };
    effects
}

fn selectable_update<T: CatalogResourceAdapter>(
    selectable: &mut Selectable,
    selected: &Option<Selected>,
    catalog: &Catalog<T>,
    profile: &Profile,
) -> Effects {
    let selectable_catalogs = profile
        .addons
        .iter()
        .flat_map(|addon| {
            T::catalogs(&addon.manifest)
                .iter()
                .map(move |manifest_catalog| (addon, manifest_catalog))
        })
        .filter_map(|(addon, manifest_catalog)| {
            manifest_catalog.default_required_extra().map(|extra| {
                let request = ResourceRequest {
                    base: addon.transport_url.to_owned(),
                    path: ResourcePath {
                        id: manifest_catalog.id.to_owned(),
                        r#type: manifest_catalog.r#type.to_owned(),
                        resource: T::resource().to_owned(),
                        extra,
                    },
                };
                (manifest_catalog, request)
            })
        })
        .map(|(manifest_catalog, request)| SelectableCatalog {
            catalog: manifest_catalog
                .name
                .as_ref()
                .unwrap_or(&manifest_catalog.id)
                .to_owned(),
            selected: selected
                .as_ref()
                .map(|selected| {
                    selected.request.base == request.base
                        && selected.request.path.id == request.path.id
                        && selected.request.path.resource == request.path.resource
                })
                .unwrap_or_default(),
            request,
        })
        .collect::<Vec<_>>();
    let selectable_types = selectable_catalogs
        .iter()
        .map(|selectable_catalog| &selectable_catalog.request)
        .filter(|request| match selected {
            Some(selected) if T::selectable_priority() == SelectablePriority::Catalog => {
                request.base == selected.request.base
                    && request.path.id == selected.request.path.id
                    && request.path.resource == selected.request.path.resource
            }
            _ => true,
        })
        .unique_by(|request| &request.path.r#type)
        .cloned()
        .map(|request| SelectableType {
            r#type: request.path.r#type.to_owned(),
            selected: selected
                .as_ref()
                .map(|selected| {
                    selected.request.path.r#type == request.path.r#type
                        && selected.request.path.resource == request.path.resource
                })
                .unwrap_or_default(),
            request,
        })
        .sorted_by(|a, b| {
            compare_with_priorities(a.r#type.as_str(), b.r#type.as_str(), &*TYPE_PRIORITIES)
        })
        .rev()
        .collect::<Vec<_>>();
    let selectable_catalogs = match T::selectable_priority() {
        SelectablePriority::Type => selectable_catalogs
            .into_iter()
            .filter(|selectable_catalog| match selected {
                Some(selected) => {
                    selectable_catalog.request.path.r#type == selected.request.path.r#type
                }
                _ => true,
            })
            .collect::<Vec<_>>(),
        SelectablePriority::Catalog => selectable_catalogs
            .into_iter()
            .unique_by(|selectable_catalog| {
                (
                    selectable_catalog.request.base.to_owned(),
                    selectable_catalog.request.path.id.to_owned(),
                )
            })
            .collect::<Vec<_>>(),
    };
    let (selectable_extra, next_page) = selected
        .as_ref()
        .filter(|selected| selected.request.path.resource == T::resource())
        .and_then(|selected| {
            profile
                .addons
                .iter()
                .find(|addon| addon.transport_url == selected.request.base)
                .map(|addon| (selected, addon))
        })
        .and_then(|(selected, addon)| {
            T::catalogs(&addon.manifest)
                .iter()
                .find(|manifest_catalog| {
                    manifest_catalog.id == selected.request.path.id
                        && manifest_catalog.r#type == selected.request.path.r#type
                })
                .map(|manifest_catalog| (selected, manifest_catalog))
        })
        .map(|(selected, manifest_catalog)| {
            let selectable_extra = manifest_catalog
                .extra
                .iter()
                .filter(|extra_prop| {
                    extra_prop.name != SKIP_EXTRA_PROP.name && !extra_prop.options.is_empty()
                })
                .map(|extra_prop| {
                    let none_option =
                        (!extra_prop.is_required)
                            .as_option()
                            .map(|_| SelectableExtraOption {
                                value: None,
                                selected: selected
                                    .request
                                    .path
                                    .extra
                                    .iter()
                                    .all(|extra_value| extra_value.name != extra_prop.name),
                                request: ResourceRequest {
                                    base: selected.request.base.to_owned(),
                                    path: ResourcePath {
                                        id: manifest_catalog.id.to_owned(),
                                        r#type: manifest_catalog.r#type.to_owned(),
                                        resource: selected.request.path.resource.to_owned(),
                                        extra: selected
                                            .request
                                            .path
                                            .extra
                                            .to_owned()
                                            .extend_one(&extra_prop, None),
                                    },
                                },
                            });
                    let options = extra_prop
                        .options
                        .iter()
                        .map(|value| SelectableExtraOption {
                            value: Some(value.to_owned()),
                            selected: selected.request.path.extra.iter().any(|extra_value| {
                                extra_value.name == extra_prop.name && extra_value.value == *value
                            }),
                            request: ResourceRequest {
                                base: selected.request.base.to_owned(),
                                path: ResourcePath {
                                    id: manifest_catalog.id.to_owned(),
                                    r#type: manifest_catalog.r#type.to_owned(),
                                    resource: selected.request.path.resource.to_owned(),
                                    extra: selected
                                        .request
                                        .path
                                        .extra
                                        .to_owned()
                                        .extend_one(&extra_prop, Some(value.to_owned())),
                                },
                            },
                        })
                        .collect::<Vec<_>>();
                    SelectableExtra {
                        name: extra_prop.name.to_owned(),
                        is_required: extra_prop.is_required.to_owned(),
                        options: none_option.into_iter().chain(options).collect(),
                    }
                })
                .collect();
            let next_page = manifest_catalog
                .extra
                .iter()
                .find(|extra_prop| extra_prop.name == SKIP_EXTRA_PROP.name)
                .and_then(|_| {
                    catalog
                        .iter()
                        .map(|page| {
                            page.content
                                .as_ref()
                                .and_then(|content| content.ready())
                                .filter(|content| !content.is_empty())
                                .map(|content| content.len())
                        })
                        .collect::<Option<Vec<_>>>()
                        .map(|page_sizes| page_sizes.into_iter().fold(0, Add::add))
                })
                .map(|skip| SelectablePage {
                    request: ResourceRequest {
                        base: selected.request.base.to_owned(),
                        path: ResourcePath {
                            id: manifest_catalog.id.to_owned(),
                            r#type: manifest_catalog.r#type.to_owned(),
                            resource: selected.request.path.resource.to_owned(),
                            extra: selected
                                .request
                                .path
                                .extra
                                .to_owned()
                                .extend_one(&SKIP_EXTRA_PROP, Some(skip.to_string())),
                        },
                    },
                });
            (selectable_extra, next_page)
        })
        .unwrap_or_default();
    let next_selectable = Selectable {
        types: selectable_types,
        catalogs: selectable_catalogs,
        extra: selectable_extra,
        next_page,
    };
    eq_update(selectable, next_selectable)
}
