use super::common::{
    resource_update_with_vector_content, ResourceAction, ResourceContent, ResourceLoadable,
};
use crate::state_types::messages::*;
use crate::state_types::models::*;
use crate::state_types::*;
use crate::types::addons::*;
use crate::types::MetaPreview;
use itertools::*;
use serde_derive::*;
use std::convert::TryFrom;
use std::marker::PhantomData;

const CATALOG_PAGE_SIZE: usize = 100;
const SKIP: &str = "skip";

pub trait ResourceAdapter {
    fn resource() -> &'static str;
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog];
}

impl ResourceAdapter for MetaPreview {
    fn resource() -> &'static str {
        "catalog"
    }
    fn catalogs_from_manifest(manifest: &Manifest) -> &[ManifestCatalog] {
        &manifest.catalogs
    }
}

impl ResourceAdapter for DescriptorPreview {
    fn resource() -> &'static str {
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
    T: Clone + ResourceAdapter,
    Vec<T>: TryFrom<ResourceResponse>,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        let selectable_update = match &self.selectable_priority {
            SelectablePriority::Catalog => selectable_update_with_catalog_priority::<T>,
            SelectablePriority::Type => selectable_update_with_type_priority::<T>,
        };
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogFiltered(request))) => {
                let request = request_with_valid_extra(request);
                let catalog_effects = resource_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resource,
                    ResourceAction::ResourceRequested {
                        request: &request,
                        env: PhantomData,
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
                    None => Effects::none().unchanged(),
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
                    None => Effects::none().unchanged(),
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

fn selectable_update_with_catalog_priority<T: ResourceAdapter>(
    selectable: &mut Selectable,
    action: SelectableAction<T>,
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

fn selectable_update_with_type_priority<T: ResourceAdapter>(
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
            let selectable_extra = extra_from_requested_catalog(addons, &resource.request);
            let (has_prev_page, has_next_page) =
                pagination_from_resource(&selectable_extra, resource);
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

fn request_with_valid_extra(request: &ResourceRequest) -> ResourceRequest {
    let extra = request
        .path
        .extra
        .iter()
        .cloned()
        .fold::<Vec<ExtraProp>, _>(vec![], |mut extra, (key, value)| {
            if key.eq(SKIP) && extra.iter().all(|(key, _)| key.ne(SKIP)) {
                if let Ok(value) = value.parse::<u32>() {
                    let value = (value / CATALOG_PAGE_SIZE as u32) * CATALOG_PAGE_SIZE as u32;
                    extra.push((key, value.to_string()));
                };
            } else {
                extra.push((key, value));
            };

            extra
        });
    ResourceRequest {
        base: request.base.to_owned(),
        path: ResourceRef {
            resource: request.path.resource.to_owned(),
            type_name: request.path.type_name.to_owned(),
            id: request.path.id.to_owned(),
            extra,
        },
    }
}

fn catalogs_from_addons<T: ResourceAdapter>(addons: &[Descriptor]) -> Vec<SelectableCatalog> {
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
                            T::resource(),
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

fn extra_from_requested_catalog<'a>(
    addons: &'a [Descriptor],
    request: &ResourceRequest,
) -> Vec<ManifestExtraProp> {
    addons
        .iter()
        .find(|addon| addon.transport_url.eq(&request.base))
        .iter()
        .flat_map(|addon| &addon.manifest.catalogs)
        .find(|catalog| {
            catalog.type_name.eq(&request.path.type_name) && catalog.id.eq(&request.path.id)
        })
        .map(|catalog| {
            catalog
                .extra_iter()
                .filter(|extra| extra.options.iter().flatten().next().is_some())
                .map(|extra| extra.into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn pagination_from_resource<T>(
    selectable_extra: &[ManifestExtraProp],
    resource: &ResourceLoadable<Vec<T>>,
) -> (bool, bool) {
    let skip_supported = selectable_extra.iter().any(|extra| extra.name.eq(SKIP));
    let skip_requested = resource
        .request
        .path
        .get_extra_first_val(SKIP)
        .and_then(|value| value.parse::<u32>().ok())
        .is_some();
    let last_page_requested = match &resource.content {
        ResourceContent::Ready(content) => content.len() < CATALOG_PAGE_SIZE,
        ResourceContent::Err(_) => true,
        ResourceContent::Loading => false,
    };
    let has_prev_page = skip_supported && skip_requested;
    let has_next_page = skip_supported && !last_page_requested;
    (has_prev_page, has_next_page)
}
