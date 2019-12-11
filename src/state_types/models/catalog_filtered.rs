use crate::constants::{META_CATALOG_PAGE_SIZE, SKIP_EXTRA_NAME};
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
    fn resource_name() -> &'static str;
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog];
    fn catalog_page_size() -> Option<usize>;
}

impl CatalogResourceAdapter for MetaPreview {
    fn resource_name() -> &'static str {
        "catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.catalogs
    }
    fn catalog_page_size() -> Option<usize> {
        Some(META_CATALOG_PAGE_SIZE)
    }
}

impl CatalogResourceAdapter for DescriptorPreview {
    fn resource_name() -> &'static str {
        "addon_catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.addon_catalogs
    }
    fn catalog_page_size() -> Option<usize> {
        None
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
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered(request))) => {
                let extra = validate_extra(&request.path.extra, T::catalog_page_size());
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
                            selectable_priority: &self.selectable_priority,
                        },
                    ),
                    _ => Effects::none().unchanged(),
                };
                selectable_effects.join(catalog_effects)
            }
            Msg::Action(Action::Unload) => resource_update_with_vector_content::<_, Env>(
                &mut self.catalog_resource,
                ResourceAction::ResourceReplaced { resource: None },
            ),
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                let catalog_effects = resource_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: T::catalog_page_size(),
                    },
                );
                let selectable_effects = match &self.catalog_resource {
                    Some(catalog_resource) => selectable_update(
                        &mut self.selectable,
                        SelectableAction::ResourceChanged {
                            resource: catalog_resource,
                            addons: &ctx.content.addons,
                            selectable_priority: &self.selectable_priority,
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
                        selectable_priority: &self.selectable_priority,
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
        selectable_priority: &'a SelectablePriority,
    },
    AddonsChanged {
        addons: &'a [Descriptor],
        resource: &'a Option<ResourceLoadable<Vec<T>>>,
        selectable_priority: &'a SelectablePriority,
    },
}

fn selectable_update<T: CatalogResourceAdapter>(
    selectable: &mut Selectable,
    action: SelectableAction<T>,
) -> Effects {
    let (resource, addons, selectable_priority) = match action {
        SelectableAction::ResourceChanged {
            resource,
            addons,
            selectable_priority,
        }
        | SelectableAction::AddonsChanged {
            addons,
            resource: Some(resource),
            selectable_priority,
        } => (Some(resource), addons, selectable_priority),
        SelectableAction::AddonsChanged {
            addons,
            resource: None,
            selectable_priority,
        } => (None, addons, selectable_priority),
    };
    let selectable_catalogs = addons
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
                .map(|default_required_extra| SelectableCatalog {
                    name: catalog.name.as_ref().unwrap_or(&catalog.id).to_owned(),
                    addon_name: addon.manifest.name.to_owned(),
                    load_request: ResourceRequest {
                        base: addon.transport_url.to_owned(),
                        path: ResourceRef::with_extra(
                            T::resource_name(),
                            &catalog.type_name,
                            &catalog.id,
                            &default_required_extra,
                        ),
                    },
                })
        })
        .collect::<Vec<_>>()
        .iter()
        .unique_by(|selectable_catalog| &selectable_catalog.load_request)
        .cloned()
        .collect::<Vec<_>>();
    let (selectable_catalogs, selectable_types) = match selectable_priority {
        SelectablePriority::Catalog => {
            let selectable_types = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match resource {
                    Some(resource) => selectable_catalog
                        .load_request
                        .path
                        .id
                        .eq(&resource.request.path.id),
                    None => true,
                })
                .unique_by(|selectable_catalog| &selectable_catalog.load_request.path.type_name)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.load_request.path.type_name.to_owned(),
                    load_request: selectable_catalog.load_request.to_owned(),
                })
                .collect::<Vec<_>>();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| &selectable_catalog.load_request.path.id)
                .cloned()
                .collect();
            (selectable_catalogs, selectable_types)
        }
        SelectablePriority::Type => {
            let selectable_types = selectable_catalogs
                .iter()
                .unique_by(|selectable_catalog| &selectable_catalog.load_request.path.type_name)
                .map(|selectable_catalog| SelectableType {
                    name: selectable_catalog.load_request.path.type_name.to_owned(),
                    load_request: selectable_catalog.load_request.to_owned(),
                })
                .collect();
            let selectable_catalogs = selectable_catalogs
                .iter()
                .filter(|selectable_catalog| match resource {
                    Some(resource) => selectable_catalog
                        .load_request
                        .path
                        .type_name
                        .eq(&resource.request.path.type_name),
                    None => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            (selectable_catalogs, selectable_types)
        }
    };
    let (selectable_extra, has_prev_page, has_next_page) = match action {
        SelectableAction::ResourceChanged {
            resource, addons, ..
        }
        | SelectableAction::AddonsChanged {
            addons,
            resource: Some(resource),
            ..
        } => {
            let requested_catalog = addons
                .iter()
                .find(|addon| addon.transport_url.eq(&resource.request.base))
                .iter()
                .flat_map(|addon| T::catalogs_from_manifest(&addon.manifest))
                .find(|catalog| {
                    catalog.type_name.eq(&resource.request.path.type_name)
                        && catalog.id.eq(&resource.request.path.id)
                });
            match &requested_catalog {
                Some(requested_catalog) => {
                    let selectable_extra = requested_catalog
                        .extra_iter()
                        .filter(|extra| extra.options.iter().flatten().next().is_some())
                        .map(|extra| extra.into_owned())
                        .collect();
                    let skip_supported = requested_catalog
                        .extra_iter()
                        .any(|extra| extra.name.eq(SKIP_EXTRA_NAME));
                    let first_page_requested = resource
                        .request
                        .path
                        .get_extra_first_val(SKIP_EXTRA_NAME)
                        .and_then(|value| value.parse::<u32>().ok())
                        .map(|skip| skip.eq(&0))
                        .unwrap_or(true);
                    let last_page_requested = match &resource.content {
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
                }
                _ => Default::default(),
            }
        }
        _ => Default::default(),
    };
    let next_selectable = Selectable {
        catalogs: selectable_catalogs,
        types: selectable_types,
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
