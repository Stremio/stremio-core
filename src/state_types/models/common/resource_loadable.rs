use super::{get_resource, Loadable};
use crate::state_types::msg::{Internal, Msg, MsgError};
use crate::state_types::{Effect, Effects, Environment};
use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use futures::{future, Future};
use serde::Serialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum ResourceError {
    EmptyContent,
    UnexpectedResponse(String),
    Other(String),
}

pub type ResourceContent<T> = Loadable<T, ResourceError>;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceLoadable<T> {
    pub request: ResourceRequest,
    pub content: ResourceContent<T>,
}

impl<T> PartialEq for ResourceLoadable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.request.eq(&other.request)
    }
}

pub enum ResourceAction<'a> {
    ResourceRequested {
        request: &'a ResourceRequest,
    },
    ResourceRequestResult {
        request: &'a ResourceRequest,
        result: &'a Result<ResourceResponse, MsgError>,
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
        result: &'a Result<ResourceResponse, MsgError>,
        limit: &'a Option<usize>,
    },
}

pub fn resource_update<Env, T>(
    resource: &mut Option<ResourceLoadable<T>>,
    action: ResourceAction,
) -> Effects
where
    Env: Environment + 'static,
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequested { request } => {
            if Some(request).ne(&resource.as_ref().map(|resource| &resource.request)) {
                let request = request.to_owned();
                *resource = Some(ResourceLoadable {
                    request: request.to_owned(),
                    content: ResourceContent::Loading,
                });
                Effects::one(Box::new(get_resource::<Env>(&request).then(
                    move |result| {
                        future::ok(Msg::Internal(Internal::ResourceRequestResult(
                            request,
                            Box::new(result),
                        )))
                    },
                )))
            } else {
                Effects::none().unchanged()
            }
        }
        ResourceAction::ResourceRequestResult {
            request, result, ..
        } => match resource {
            Some(resource) if resource.request.eq(request) => {
                resource.content = resource_content_from_result(result);
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
    }
}

pub fn resource_update_with_vector_content<Env, T>(
    resource: &mut Option<ResourceLoadable<Vec<T>>>,
    action: ResourceAction,
) -> Effects
where
    Env: Environment + 'static,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequestResult {
            request,
            result,
            limit,
        } => match resource {
            Some(resource) if resource.request.eq(request) => {
                resource.content = resource_vector_content_from_result(result, limit);
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        },
        _ => resource_update::<Env, _>(resource, action),
    }
}

pub fn resources_update<Env, T>(
    resources: &mut Vec<ResourceLoadable<T>>,
    action: ResourcesAction,
) -> Effects
where
    Env: Environment + 'static,
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
                    .map(|request| -> (_, Effect) {
                        (
                            ResourceLoadable {
                                request: request.to_owned(),
                                content: ResourceContent::Loading,
                            },
                            Box::new(get_resource::<Env>(&request).then(move |result| {
                                future::ok(Msg::Internal(Internal::ResourceRequestResult(
                                    request,
                                    Box::new(result),
                                )))
                            })),
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
                .position(|resource| resource.request.eq(request))
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

pub fn resources_update_with_vector_content<Env, T>(
    resources: &mut Vec<ResourceLoadable<Vec<T>>>,
    action: ResourcesAction,
) -> Effects
where
    Env: Environment + 'static,
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
                .position(|resource| resource.request.eq(request))
            {
                Some(position) => {
                    resources[position].content =
                        resource_vector_content_from_result(result, limit);
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            }
        }
        _ => resources_update::<Env, _>(resources, action),
    }
}

fn resource_content_from_result<T>(
    result: &Result<ResourceResponse, MsgError>,
) -> ResourceContent<T>
where
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match result {
        Ok(result) => match T::try_from(result.to_owned()) {
            Ok(content) => ResourceContent::Ready(content),
            Err(error) => ResourceContent::Err(ResourceError::UnexpectedResponse(error.to_owned())),
        },
        Err(error) => ResourceContent::Err(ResourceError::Other(error.to_string())),
    }
}

fn resource_vector_content_from_result<T>(
    result: &Result<ResourceResponse, MsgError>,
    limit: &Option<usize>,
) -> ResourceContent<Vec<T>>
where
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match result {
        Ok(result) => match <Vec<T>>::try_from(result.to_owned()) {
            Ok(content) => {
                if content.is_empty() {
                    ResourceContent::Err(ResourceError::EmptyContent)
                } else if let Some(limit) = limit {
                    ResourceContent::Ready(content.into_iter().take(limit.to_owned()).collect())
                } else {
                    ResourceContent::Ready(content)
                }
            }
            Err(error) => ResourceContent::Err(ResourceError::UnexpectedResponse(error.to_owned())),
        },
        Err(error) => ResourceContent::Err(ResourceError::Other(error.to_string())),
    }
}
