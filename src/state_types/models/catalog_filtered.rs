use crate::constants::{CATALOG_PAGE_SIZE, SKIP};
use crate::state_types::messages::{Action, ActionLoad, Event, Internal, Msg};
use crate::state_types::models::common::{
    resource_update_with_vector_content, validate_extra, ResourceAction, ResourceContent,
    ResourceLoadable,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{
    Descriptor, DescriptorPreview, ExtraProp, Manifest, ManifestCatalog, ManifestExtraProp,
    ResourceRef, ResourceRequest, ResourceResponse,
};
use crate::types::MetaPreview;
use itertools::Itertools;
use serde_derive::Serialize;
use std::convert::TryFrom;

pub trait CatalogResourceAdapter {
    fn catalog_resource() -> &'static str;
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog];
}

impl CatalogResourceAdapter for MetaPreview {
    fn catalog_resource() -> &'static str {
        "catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.catalogs
    }
}

impl CatalogResourceAdapter for DescriptorPreview {
    fn catalog_resource() -> &'static str {
        "addon_catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.addon_catalogs
    }
}

#[derive(Debug, Clone)]
pub enum SelectablePriority {
    Catalog,
    Type,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectableCatalog {
    pub name: String,
    pub addon_name: String,
    pub load_request: ResourceRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectableType {
    pub name: String,
    pub load_request: ResourceRequest,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selectable {
    pub catalogs: Vec<SelectableCatalog>,
    pub types: Vec<SelectableType>,
    pub extra: Vec<ManifestExtraProp>,
    pub has_prev_page: bool,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogFiltered<T> {
    pub selectable: Selectable,
    pub catalog_resource: Option<ResourceLoadable<Vec<T>>>,
    #[serde(skip)]
    pub selectable_priority: SelectablePriority,
}

impl<Env, T> UpdateWithCtx<Ctx<Env>> for CatalogFiltered<T>
where
    Env: Environment + 'static,
    T: CatalogResourceAdapter,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        let selectable_update = match &self.selectable_priority {
            SelectablePriority::Catalog => selectable_update_with_catalog_priority::<T>,
            SelectablePriority::Type => selectable_update_with_type_priority::<T>,
        };
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered(request))) => {
                let extra = validate_extra(&request.path.extra);
                let request = ResourceRequest {
                    base: request.base.to_owned(),
                    path: ResourceRef {
                        resource: request.path.resource.to_owned(),
                        type_name: request.path.type_name.to_owned(),
                        id: request.path.id.to_owned(),
                        extra,
                    },
                };
                let catalog_effects = resource_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceRequested { request: &request },
                );
                let selectable_effects = match &self.catalog_resource {
                    Some(catalog_resource) => selectable_update(
                        &mut self.selectable,
                        SelectableAction::ResourceChanged {
                            resource: catalog_resource,
                            addons: &ctx.content.addons,
                        },
                    ),
                    _ => Effects::none().unchanged(),
                };
                selectable_effects.join(catalog_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                let catalog_effects = resource_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: Some(CATALOG_PAGE_SIZE),
                    },
                );
                let selectable_effects = match &self.catalog_resource {
                    Some(catalog_resource) => selectable_update(
                        &mut self.selectable,
                        SelectableAction::ResourceChanged {
                            resource: catalog_resource,
                            addons: &ctx.content.addons,
                        },
                    ),
                    _ => Effects::none().unchanged(),
                };
                catalog_effects.join(selectable_effects)
            }
            Msg::Internal(Internal::CtxLoaded(_)) | Msg::Event(Event::CtxChanged) => {
                selectable_update(
                    &mut self.selectable,
                    SelectableAction::AddonsChanged {
                        resource: &self.catalog_resource,
                        addons: &ctx.content.addons,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectableAction<'a, T> {
    ResourceChanged {
        resource: &'a ResourceLoadable<Vec<T>>,
        addons: &'a [Descriptor],
    },
    AddonsChanged {
        resource: &'a Option<ResourceLoadable<Vec<T>>>,
        addons: &'a [Descriptor],
    },
}

fn selectable_update_with_catalog_priority<T: CatalogResourceAdapter>(
    _selectable: &mut Selectable,
    _action: SelectableAction<T>,
) -> Effects {
    // match action {
    //     SelectableAction::Select {
    //         request, addons, ..
    //     } => {
    //         let selectable_catalogs = catalogs_from_addons::<T>(addons);
    //         let selectable_types = types_from_catalogs(&selectable_catalogs);
    //         // TODO fix this
    //         let types = selectable_types.iter().filter(|selectable_type| {
    //             selectable_type.load_request.path.id.eq(&request.path.id)
    //         });
    //     }
    // };
    Effects::none().unchanged()
}

fn selectable_update_with_type_priority<T: CatalogResourceAdapter>(
    selectable: &mut Selectable,
    action: SelectableAction<T>,
) -> Effects {
    let next_selectable = match action {
        SelectableAction::ResourceChanged { resource, addons }
        | SelectableAction::AddonsChanged {
            resource: Some(resource),
            addons,
        } => {
            let selectable_catalogs = catalogs_from_addons::<T>(addons);
            let selectable_types = types_from_catalogs(&selectable_catalogs);
            let selectable_catalogs = selectable_catalogs
                .iter()
                .filter(|catalog| {
                    catalog
                        .load_request
                        .path
                        .type_name
                        .eq(&resource.request.path.type_name)
                })
                .cloned()
                .collect::<Vec<_>>();
            let requested_catalog = requested_catalog_from_addons(addons, &resource.request);
            let (selectable_extra, has_prev_page, has_next_page) = match &requested_catalog {
                Some(requested_catalog) => {
                    let selectable_extra = extra_from_requested_catalog(&requested_catalog);
                    let (has_prev_page, has_next_page) =
                        pagination_from_requested_catalog(&requested_catalog, resource);
                    (selectable_extra, has_prev_page, has_next_page)
                }
                _ => (vec![], false, false),
            };
            Selectable {
                catalogs: selectable_catalogs,
                types: selectable_types,
                extra: selectable_extra,
                has_prev_page,
                has_next_page,
            }
        }
        SelectableAction::AddonsChanged {
            resource: None,
            addons,
        } => {
            let selectable_catalogs = catalogs_from_addons::<T>(addons);
            let selectable_types = types_from_catalogs(&selectable_catalogs);
            Selectable {
                catalogs: selectable_catalogs,
                types: selectable_types,
                extra: vec![],
                has_prev_page: false,
                has_next_page: false,
            }
        }
    };
    if next_selectable.ne(selectable) {
        *selectable = next_selectable;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn catalogs_from_addons<T: CatalogResourceAdapter>(
    addons: &[Descriptor],
) -> Vec<SelectableCatalog> {
    addons
        .iter()
        .flat_map(|addon| {
            T::catalogs_from_manifest(&addon.manifest)
                .iter()
                .map(move |catalog| (addon, catalog))
        })
        .filter_map(|(addon, catalog)| {
            catalog
                .extra_iter()
                .filter(|extra| extra.is_required)
                .map(|extra| {
                    extra
                        .options
                        .as_ref()
                        .and_then(|options| options.first())
                        .map(|first_option| (extra.name.to_owned(), first_option.to_owned()))
                })
                .collect::<Option<Vec<ExtraProp>>>()
                .map(|default_required_extra| {
                    let load_request = ResourceRequest {
                        base: addon.transport_url.to_owned(),
                        path: ResourceRef::with_extra(
                            T::catalog_resource(),
                            &catalog.type_name,
                            &catalog.id,
                            &default_required_extra,
                        ),
                    };
                    SelectableCatalog {
                        name: catalog.name.as_ref().unwrap_or(&catalog.id).to_owned(),
                        addon_name: addon.manifest.name.to_owned(),
                        load_request,
                    }
                })
        })
        .collect::<Vec<SelectableCatalog>>()
        .iter()
        .unique_by(|selectable_catalog| &selectable_catalog.load_request)
        .cloned()
        .collect()
}

fn types_from_catalogs(selectable_catalogs: &[SelectableCatalog]) -> Vec<SelectableType> {
    selectable_catalogs
        .iter()
        .unique_by(|selectable_catalog| &selectable_catalog.load_request.path.type_name)
        .map(|selectable_catalog| SelectableType {
            name: selectable_catalog.load_request.path.type_name.to_owned(),
            load_request: selectable_catalog.load_request.to_owned(),
        })
        .collect()
}

fn requested_catalog_from_addons(
    addons: &[Descriptor],
    request: &ResourceRequest,
) -> Option<ManifestCatalog> {
    addons
        .iter()
        .find(|addon| addon.transport_url.eq(&request.base))
        .iter()
        .flat_map(|addon| &addon.manifest.catalogs)
        .find(|catalog| {
            catalog.type_name.eq(&request.path.type_name) && catalog.id.eq(&request.path.id)
        })
        .cloned()
}

fn extra_from_requested_catalog(catalog: &ManifestCatalog) -> Vec<ManifestExtraProp> {
    catalog
        .extra_iter()
        .filter(|extra| extra.options.iter().flatten().next().is_some())
        .map(|extra| extra.into_owned())
        .collect()
}

fn pagination_from_requested_catalog<T>(
    catalog: &ManifestCatalog,
    resource: &ResourceLoadable<Vec<T>>,
) -> (bool, bool) {
    let skip_supported = catalog.extra_iter().any(|extra| extra.name.eq(SKIP));
    let first_page_requested = resource
        .request
        .path
        .get_extra_first_val(SKIP)
        .and_then(|value| value.parse::<u32>().ok())
        .map(|skip| skip.eq(&0))
        .unwrap_or(true);
    let last_page_requested = match &resource.content {
        ResourceContent::Ready(content) => content.len() < CATALOG_PAGE_SIZE,
        ResourceContent::Err(_) => true,
        ResourceContent::Loading => false,
    };
    let has_prev_page = skip_supported && !first_page_requested;
    let has_next_page = skip_supported && !last_page_requested;
    (has_prev_page, has_next_page)
}
