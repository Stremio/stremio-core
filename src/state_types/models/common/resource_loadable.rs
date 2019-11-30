use crate::state_types::models::common::{addon_get, Loadable};
use crate::state_types::{Effects, EnvError, Environment};
use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum ResourceError {
    EmptyContent,
    UnexpectedResp,
    Other(String),
}

pub type ResourceContent<T> = Loadable<T, ResourceError>;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceLoadable<T> {
    pub request: ResourceRequest,
    pub content: ResourceContent<T>,
}

pub enum ResourceAction<'a, T, Env: Environment + 'static> {
    ResourceRequested {
        request: &'a ResourceRequest,
        env: PhantomData<Env>,
    },
    ResourceReplaced {
        resource: ResourceLoadable<T>,
    },
    ResourceResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn resource_update<T, Env>(
    resource: &mut ResourceLoadable<T>,
    action: ResourceAction<T, Env>,
) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        ResourceAction::ResourceRequested { request, .. } => {
            if request.ne(&resource.request) {
                *resource = ResourceLoadable {
                    request: request.to_owned(),
                    content: ResourceContent::Loading,
                };
                Effects::one(addon_get::<Env>(request.to_owned()))
            } else {
                Effects::none().unchanged()
            }
        }
        ResourceAction::ResourceReplaced {
            resource: next_resource,
        } => {
            if next_resource.request.ne(&resource.request) {
                *resource = next_resource;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        ResourceAction::ResourceResponseReceived {
            request, response, ..
        } => {
            if request.eq(&resource.request) {
                resource.content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(content) => ResourceContent::Ready(content),
                        Err(_) => ResourceContent::Err(ResourceError::UnexpectedResp),
                    },
                    Err(error) => ResourceContent::Err(ResourceError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

pub fn resource_update_with_vector_content<T, Env>(
    resource: &mut ResourceLoadable<Vec<T>>,
    action: ResourceAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        ResourceAction::ResourceResponseReceived {
            request,
            response,
            limit,
        } => {
            if request.eq(&resource.request) {
                resource.content = match response {
                    Ok(response) => match <Vec<T>>::try_from(response.to_owned()) {
                        Ok(ref content) if content.is_empty() => {
                            ResourceContent::Err(ResourceError::EmptyContent)
                        }
                        Ok(content) => {
                            if let Some(limit) = limit {
                                ResourceContent::Ready(content.into_iter().take(limit).collect())
                            } else {
                                ResourceContent::Ready(content)
                            }
                        }
                        Err(_) => ResourceContent::Err(ResourceError::UnexpectedResp),
                    },
                    Err(error) => ResourceContent::Err(ResourceError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        _ => resource_update::<_, Env>(resource, action),
    }
}

pub enum ResourcesAction<'a, T, Env: Environment + 'static> {
    ResourcesRequested {
        addons: &'a [Descriptor],
        request: &'a AggrRequest<'a>,
        env: PhantomData<Env>,
    },
    ResourcesReplaced {
        resources: Vec<ResourceLoadable<T>>,
    },
    ResourceResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn resources_update<T, Env>(
    resources: &mut Vec<ResourceLoadable<T>>,
    action: ResourcesAction<T, Env>,
) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        ResourcesAction::ResourcesRequested {
            addons, request, ..
        } => {
            let requests = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| request)
                .collect::<Vec<ResourceRequest>>();
            if requests
                .iter()
                .ne(resources.iter().map(|resource| &resource.request))
            {
                let (next_resources, effects) = requests
                    .iter()
                    .map(|request| {
                        (
                            ResourceLoadable {
                                request: request.to_owned(),
                                content: ResourceContent::Loading,
                            },
                            addon_get::<Env>(request.to_owned()),
                        )
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                *resources = next_resources;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        ResourcesAction::ResourcesReplaced {
            resources: next_resources,
        } => {
            if next_resources
                .iter()
                .map(|resource| &resource.request)
                .ne(resources.iter().map(|resource| &resource.request))
            {
                *resources = next_resources;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        ResourcesAction::ResourceResponseReceived {
            request,
            response,
            limit,
        } => {
            let resource_index = resources
                .iter()
                .position(|resource| resource.request.eq(request));
            if let Some(resource_index) = resource_index {
                resource_update::<_, Env>(
                    &mut resources[resource_index],
                    ResourceAction::ResourceResponseReceived {
                        request,
                        response,
                        limit,
                    },
                )
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

pub fn resources_update_with_vector_content<T, Env>(
    resources: &mut Vec<ResourceLoadable<Vec<T>>>,
    action: ResourcesAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        ResourcesAction::ResourceResponseReceived {
            request,
            response,
            limit,
        } => {
            let resource_index = resources
                .iter()
                .position(|resource| resource.request.eq(request));
            if let Some(resource_index) = resource_index {
                resource_update_with_vector_content::<_, Env>(
                    &mut resources[resource_index],
                    ResourceAction::ResourceResponseReceived {
                        request,
                        response,
                        limit,
                    },
                )
            } else {
                Effects::none().unchanged()
            }
        }
        _ => resources_update::<_, Env>(resources, action),
    }
}
