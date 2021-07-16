use crate::models::common::Loadable;
use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{Effects, Env, EnvError};
use crate::types::addon::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use futures::FutureExt;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum ResourceError {
    EmptyContent,
    UnexpectedResponse(String),
    Env(EnvError),
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ResourceError::EmptyContent => write!(f, "EmptyContent"),
            ResourceError::UnexpectedResponse(message) => {
                write!(f, "UnexpectedResponse: {}", message)
            }
            ResourceError::Env(error) => write!(f, "Env: {}", error.message()),
        }
    }
}

#[derive(PartialEq, Serialize)]
pub struct ResourceLoadable<T> {
    pub request: ResourceRequest,
    pub content: Loadable<T, ResourceError>,
}

pub enum ResourceAction<'a> {
    ResourceRequested {
        request: &'a ResourceRequest,
    },
    ResourceRequestResult {
        request: &'a ResourceRequest,
        result: &'a Result<ResourceResponse, EnvError>,
        limit: &'a Option<usize>,
    },
}

pub enum ResourcesAction<'a> {
    ResourcesRequested {
        request: &'a AggrRequest<'a>,
        addons: &'a [Descriptor],
    },
    ResourceRequestResult {
        request: &'a ResourceRequest,
        result: &'a Result<ResourceResponse, EnvError>,
        limit: &'a Option<usize>,
    },
}

pub fn resource_update<E, T>(
    resource: &mut Option<ResourceLoadable<T>>,
    action: ResourceAction,
) -> Effects
where
    E: Env + 'static,
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequested { request } => {
            if resource.as_ref().map(|resource| &resource.request) != Some(request) {
                let request = request.to_owned();
                *resource = Some(ResourceLoadable {
                    request: request.to_owned(),
                    content: Loadable::Loading,
                });
                Effects::future(
                    E::addon_transport(&request.base)
                        .resource(&request.path)
                        .map(move |result| {
                            Msg::Internal(Internal::ResourceRequestResult(
                                request,
                                Box::new(result),
                            ))
                        })
                        .boxed_local(),
                )
            } else {
                Effects::none().unchanged()
            }
        }
        ResourceAction::ResourceRequestResult {
            request, result, ..
        } => match resource {
            Some(resource) if resource.request == *request => {
                resource.content = resource_content_from_result(result);
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
    }
}

pub fn resource_update_with_vector_content<E, T>(
    resource: &mut Option<ResourceLoadable<Vec<T>>>,
    action: ResourceAction,
) -> Effects
where
    E: Env + 'static,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequestResult {
            request,
            result,
            limit,
        } => match resource {
            Some(resource) if resource.request == *request => {
                resource.content = resource_vector_content_from_result(result, limit);
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
        _ => resource_update::<E, _>(resource, action),
    }
}

pub fn resources_update<E, T>(
    resources: &mut Vec<ResourceLoadable<T>>,
    action: ResourcesAction,
) -> Effects
where
    E: Env + 'static,
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourcesAction::ResourcesRequested { request, addons } => {
            let requests = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| request)
                .collect::<Vec<_>>();
            if requests
                .iter()
                .ne(resources.iter().map(|resource| &resource.request))
            {
                let (next_resources, effects) = requests
                    .iter()
                    .cloned()
                    .map(|request| {
                        (
                            ResourceLoadable {
                                request: request.to_owned(),
                                content: Loadable::Loading,
                            },
                            E::addon_transport(&request.base)
                                .resource(&request.path)
                                .map(move |result| {
                                    Msg::Internal(Internal::ResourceRequestResult(
                                        request,
                                        Box::new(result),
                                    ))
                                })
                                .boxed_local()
                                .into(),
                        )
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                *resources = next_resources;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        ResourcesAction::ResourceRequestResult {
            request, result, ..
        } => {
            match resources
                .iter()
                .position(|resource| resource.request == *request)
            {
                Some(position) => {
                    resources[position].content = resource_content_from_result(result);
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            }
        }
    }
}

pub fn resources_update_with_vector_content<E, T>(
    resources: &mut Vec<ResourceLoadable<Vec<T>>>,
    action: ResourcesAction,
) -> Effects
where
    E: Env + 'static,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourcesAction::ResourceRequestResult {
            request,
            result,
            limit,
        } => {
            match resources
                .iter()
                .position(|resource| resource.request == *request)
            {
                Some(position) => {
                    resources[position].content =
                        resource_vector_content_from_result(result, limit);
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            }
        }
        _ => resources_update::<E, _>(resources, action),
    }
}

fn resource_content_from_result<T>(
    result: &Result<ResourceResponse, EnvError>,
) -> Loadable<T, ResourceError>
where
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match result {
        Ok(result) => match T::try_from(result.to_owned()) {
            Ok(content) => Loadable::Ready(content),
            Err(error) => Loadable::Err(ResourceError::UnexpectedResponse(error.to_owned())),
        },
        Err(error) => Loadable::Err(ResourceError::Env(error.to_owned())),
    }
}

fn resource_vector_content_from_result<T>(
    result: &Result<ResourceResponse, EnvError>,
    limit: &Option<usize>,
) -> Loadable<Vec<T>, ResourceError>
where
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match result {
        Ok(result) => match <Vec<T>>::try_from(result.to_owned()) {
            Ok(content) => {
                if content.is_empty() {
                    Loadable::Err(ResourceError::EmptyContent)
                } else if let Some(limit) = limit {
                    Loadable::Ready(content.into_iter().take(limit.to_owned()).collect())
                } else {
                    Loadable::Ready(content)
                }
            }
            Err(error) => Loadable::Err(ResourceError::UnexpectedResponse(error.to_owned())),
        },
        Err(error) => Loadable::Err(ResourceError::Env(error.to_owned())),
    }
}
