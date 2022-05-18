use crate::constants::{CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME, TYPE_PRIORITIES};
use crate::models::common::{
    compare_with_priorities, eq_update, resource_update_with_vector_content, Loadable,
    ResourceAction, ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{
    DescriptorPreview, ExtraExt, Manifest, ManifestCatalog, ResourcePath, ResourceRequest,
    ResourceResponse,
};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use boolinator::Boolinator;
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(PartialEq)]
pub enum SelectablePriority {
    Type,
    Catalog,
}

pub trait CatalogResourceAdapter {
    fn resource() -> &'static str;
    fn catalogs(manifest: &Manifest) -> &[ManifestCatalog];
    fn selectable_priority() -> SelectablePriority;
    fn page_size() -> Option<usize>;
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
    fn page_size() -> Option<usize> {
        Some(CATALOG_PAGE_SIZE)
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
    fn page_size() -> Option<usize> {
        None
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub request: ResourceRequest,
}

#[derive(PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectableCatalog {
    pub catalog: String,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableType {
    pub r#type: String,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Serialize)]
pub struct SelectableExtraOption {
    pub value: Option<String>,
    pub selected: bool,
    pub request: ResourceRequest,
}

#[derive(PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectableExtra {
    pub name: String,
    pub is_required: bool,
    pub options: Vec<SelectableExtraOption>,
}

#[derive(PartialEq, Serialize)]
pub struct SelectablePage {
    pub request: ResourceRequest,
}

#[derive(Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Selectable {
    pub types: Vec<SelectableType>,
    pub catalogs: Vec<SelectableCatalog>,
    pub extra: Vec<SelectableExtra>,
    pub prev_page: Option<SelectablePage>,
    pub next_page: Option<SelectablePage>,
}

#[derive(Derivative, Serialize)]
#[derivative(Default(bound = ""))]
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
            Self {
                selectable,
                catalog,
                ..Self::default()
            },
            effects.unchanged(),
        )
    }
}

impl<E, T> UpdateWithCtx<E> for CatalogWithFilters<T>
where
    E: Env + 'static,
    T: CatalogResourceAdapter + PartialEq,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
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
                        limit: &T::page_size(),
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
            T::catalogs(&addon.manifest)
                .iter()
                .map(move |manifest_catalog| (addon, manifest_catalog))
        })
        .filter_map(|(addon, manifest_catalog)| {
            manifest_catalog
                .default_required_extra()
                .map(|default_required_extra| SelectableCatalog {
                    catalog: manifest_catalog
                        .name
                        .as_ref()
                        .unwrap_or(&manifest_catalog.id)
                        .to_owned(),
                    selected: catalog
                        .as_ref()
                        .map(|catalog| {
                            catalog.request.base == addon.transport_url
                                && catalog.request.path.id == manifest_catalog.id
                                && catalog.request.path.resource == T::resource()
                        })
                        .unwrap_or_default(),
                    request: ResourceRequest {
                        base: addon.transport_url.to_owned(),
                        path: ResourcePath {
                            resource: T::resource().to_owned(),
                            r#type: manifest_catalog.r#type.to_owned(),
                            id: manifest_catalog.id.to_owned(),
                            extra: default_required_extra,
                        },
                    },
                })
        })
        .dedup_by(|a, b| a.request.eq_no_extra(&b.request))
        .collect::<Vec<_>>();
    let selectable_types = selectable_catalogs
        .iter()
        .map(|selectable_catalog| &selectable_catalog.request)
        .filter(|request| match catalog {
            Some(catalog) if T::selectable_priority() == SelectablePriority::Catalog => {
                request.base == catalog.request.base
                    && request.path.id == catalog.request.path.id
                    && request.path.resource == catalog.request.path.resource
            }
            _ => true,
        })
        .unique_by(|request| &request.path.r#type)
        .cloned()
        .map(|request| SelectableType {
            r#type: request.path.r#type.to_owned(),
            selected: catalog
                .as_ref()
                .map(|catalog| catalog.request.path.r#type == request.path.r#type)
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
            .filter(|selectable_catalog| match catalog {
                Some(catalog) => {
                    selectable_catalog.request.path.r#type == catalog.request.path.r#type
                }
                _ => true,
            })
            .collect::<Vec<_>>(),
        SelectablePriority::Catalog => selectable_catalogs
            .into_iter()
            .dedup_by(|a, b| {
                a.request.base == b.request.base && a.request.path.id == b.request.path.id
            })
            .collect::<Vec<_>>(),
    };
    let (selectable_extra, prev_page, next_page) = match catalog {
        Some(catalog) if catalog.request.path.resource == T::resource() => profile
            .addons
            .iter()
            .find(|addon| addon.transport_url == catalog.request.base)
            .iter()
            .flat_map(|addon| T::catalogs(&addon.manifest))
            .find(|ManifestCatalog { id, r#type, .. }| {
                *id == catalog.request.path.id && *r#type == catalog.request.path.r#type
            })
            .map(|manifest_catalog| {
                let selectable_extra = manifest_catalog
                    .extra
                    .iter()
                    .filter_map(|extra_prop| {
                        extra_prop
                            .options
                            .as_ref()
                            .filter(|options| !options.is_empty())
                            .map(|options| {
                                let none_option =
                                    (!extra_prop.is_required).as_option().map(|_| {
                                        SelectableExtraOption {
                                            value: None,
                                            selected: catalog.request.path.extra.iter().all(
                                                |extra_value| extra_value.name != extra_prop.name,
                                            ),
                                            request: ResourceRequest {
                                                base: catalog.request.base.to_owned(),
                                                path: ResourcePath {
                                                    resource: T::resource().to_owned(),
                                                    r#type: manifest_catalog.r#type.to_owned(),
                                                    id: manifest_catalog.id.to_owned(),
                                                    extra: catalog
                                                        .request
                                                        .path
                                                        .extra
                                                        .to_owned()
                                                        .extend_one(&extra_prop, None),
                                                },
                                            },
                                        }
                                    });
                                let options = options
                                    .iter()
                                    .map(|value| SelectableExtraOption {
                                        value: Some(value.to_owned()),
                                        selected: catalog.request.path.extra.iter().any(
                                            |extra_value| {
                                                extra_value.name == extra_prop.name
                                                    && extra_value.value == *value
                                            },
                                        ),
                                        request: ResourceRequest {
                                            base: catalog.request.base.to_owned(),
                                            path: ResourcePath {
                                                resource: T::resource().to_owned(),
                                                r#type: manifest_catalog.r#type.to_owned(),
                                                id: manifest_catalog.id.to_owned(),
                                                extra: catalog
                                                    .request
                                                    .path
                                                    .extra
                                                    .to_owned()
                                                    .extend_one(
                                                        &extra_prop,
                                                        Some(value.to_owned()),
                                                    ),
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
                    })
                    .collect();
                let (prev_page, next_page) = manifest_catalog
                    .extra
                    .iter()
                    .find(|extra_prop| extra_prop.name == SKIP_EXTRA_NAME)
                    .and_then(|extra_prop| T::page_size().map(|page_size| (extra_prop, page_size)))
                    .map(|(extra_prop, page_size)| {
                        let skip = catalog
                            .request
                            .path
                            .get_extra_first_value(SKIP_EXTRA_NAME)
                            .and_then(|value| value.parse::<u32>().ok())
                            .unwrap_or(0);
                        let prev_page = (skip > 0).as_option().map(|_| SelectablePage {
                            request: ResourceRequest {
                                base: catalog.request.base.to_owned(),
                                path: ResourcePath {
                                    resource: T::resource().to_owned(),
                                    r#type: manifest_catalog.r#type.to_owned(),
                                    id: manifest_catalog.id.to_owned(),
                                    extra: catalog.request.path.extra.to_owned().extend_one(
                                        &extra_prop,
                                        Some(
                                            ((skip.saturating_sub(page_size as u32)
                                                / page_size as u32)
                                                * page_size as u32)
                                                .to_string(),
                                        ),
                                    ),
                                },
                            },
                        });
                        let next_page = match &catalog.content {
                            Some(Loadable::Ready(content)) if content.len() == page_size => {
                                Some(SelectablePage {
                                    request: ResourceRequest {
                                        base: catalog.request.base.to_owned(),
                                        path: ResourcePath {
                                            resource: T::resource().to_owned(),
                                            r#type: manifest_catalog.r#type.to_owned(),
                                            id: manifest_catalog.id.to_owned(),
                                            extra: catalog
                                                .request
                                                .path
                                                .extra
                                                .to_owned()
                                                .extend_one(
                                                    &extra_prop,
                                                    Some(
                                                        ((skip.saturating_add(page_size as u32)
                                                            / page_size as u32)
                                                            * page_size as u32)
                                                            .to_string(),
                                                    ),
                                                ),
                                        },
                                    },
                                })
                            }
                            _ => None,
                        };
                        (prev_page, next_page)
                    })
                    .unwrap_or_default();
                (selectable_extra, prev_page, next_page)
            })
            .unwrap_or_default(),
        _ => Default::default(),
    };
    let next_selectable = Selectable {
        types: selectable_types,
        catalogs: selectable_catalogs,
        extra: selectable_extra,
        prev_page,
        next_page,
    };
    eq_update(selectable, next_selectable)
}
