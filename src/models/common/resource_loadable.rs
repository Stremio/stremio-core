use crate::models::common::{eq_update, Loadable};
use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{EffectFuture, Effects, Env, EnvError, EnvFutureExt};
use crate::types::addon::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use enclose::enclose;
use futures::FutureExt;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, PartialEq, Serialize, Debug)]
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
                write!(f, "UnexpectedResponse: {message}")
            }
            ResourceError::Env(error) => write!(f, "Env: {}", error.message()),
        }
    }
}

/// When we want to fetch meta items, streams and catalogs
#[derive(Clone, PartialEq, Serialize, Debug)]
pub struct ResourceLoadable<T> {
    pub request: ResourceRequest,
    pub content: Option<Loadable<T, ResourceError>>,
}

pub enum ResourceAction<'a> {
    ResourceRequested {
        request: &'a ResourceRequest,
    },
    ResourceRequestResult {
        request: &'a ResourceRequest,
        result: &'a Result<ResourceResponse, EnvError>,
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
    },
}

impl<T> ResourceLoadable<T> {
    pub fn update<E>(&mut self, action: ResourceAction) -> Effects
    where
        E: Env + 'static,
        T: TryFrom<ResourceResponse, Error = &'static str>,
    {
        resource_update::<E, T>(self, action)
    }
}

impl<T> ResourceLoadable<Vec<T>> {
    pub fn update_with_vector_content<E>(&mut self, action: ResourceAction) -> Effects
    where
        E: Env + 'static,
        Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
    {
        resource_update_with_vector_content::<E, T>(self, action)
    }
}

pub fn resource_update<E, T>(resource: &mut ResourceLoadable<T>, action: ResourceAction) -> Effects
where
    E: Env + 'static,
    T: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequested { request }
            if resource.request != *request || resource.content.is_none() =>
        {
            resource.request = request.to_owned();
            resource.content = Some(Loadable::Loading);
            Effects::future(EffectFuture::Concurrent(
                E::addon_transport(&request.base)
                    .resource(&request.path)
                    .map(enclose!((request) move |result| {
                        Msg::Internal(Internal::ResourceRequestResult(request, Box::new(result)))
                    }))
                    .boxed_env(),
            ))
        }
        ResourceAction::ResourceRequestResult {
            request, result, ..
        } if resource.request == *request
            && matches!(resource.content, Some(Loadable::Loading)) =>
        {
            resource.content = Some(resource_content_from_result(result));
            Effects::none()
        }
        _ => Effects::none().unchanged(),
    }
}

pub fn resource_update_with_vector_content<E, T>(
    resource: &mut ResourceLoadable<Vec<T>>,
    action: ResourceAction,
) -> Effects
where
    E: Env + 'static,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourceAction::ResourceRequestResult { request, result }
            if resource.request == *request
                && matches!(resource.content, Some(Loadable::Loading)) =>
        {
            resource.content = Some(resource_vector_content_from_result(result));
            Effects::none()
        }
        _ => resource_update::<E, _>(resource, action),
    }
}

pub fn resources_update<E, T>(
    resources: &mut Vec<ResourceLoadable<T>>,
    action: ResourcesAction,
) -> Effects
where
    E: Env + 'static,
    T: TryFrom<ResourceResponse, Error = &'static str> + Clone + PartialEq,
{
    match action {
        ResourcesAction::ResourcesRequested { request, addons } => {
            let (next_resources, effects) = request
                .plan(addons)
                .into_iter()
                .map(|(_, request)| {
                    resources
                        .iter()
                        .find(|resource| resource.request == request && resource.content.is_some())
                        .map(|resource| (resource.to_owned(), None))
                        .unwrap_or_else(|| {
                            (
                                ResourceLoadable {
                                    request: request.to_owned(),
                                    content: Some(Loadable::Loading),
                                },
                                Some(
                                    EffectFuture::Concurrent(
                                        E::addon_transport(&request.base)
                                            .resource(&request.path)
                                            .map(|result| {
                                                Msg::Internal(Internal::ResourceRequestResult(
                                                    request,
                                                    Box::new(result),
                                                ))
                                            })
                                            .boxed_env(),
                                    )
                                    .into(),
                                ),
                            )
                        })
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();
            Effects::many(effects.into_iter().flatten().collect())
                .unchanged()
                .join(eq_update(resources, next_resources))
        }
        ResourcesAction::ResourceRequestResult {
            request, result, ..
        } => {
            match resources.iter_mut().find(|resource| {
                resource.request == *request && matches!(resource.content, Some(Loadable::Loading))
            }) {
                Some(resource) => {
                    resource.content = Some(resource_content_from_result(result));
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
    T: Clone + PartialEq,
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match action {
        ResourcesAction::ResourceRequestResult { request, result } => {
            match resources.iter_mut().find(|resource| {
                resource.request == *request && matches!(resource.content, Some(Loadable::Loading))
            }) {
                Some(resource) => {
                    resource.content = Some(resource_vector_content_from_result(result));
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
) -> Loadable<Vec<T>, ResourceError>
where
    Vec<T>: TryFrom<ResourceResponse, Error = &'static str>,
{
    match result {
        Ok(result) => match <Vec<T>>::try_from(result.to_owned()) {
            Ok(content) => {
                if content.is_empty() {
                    Loadable::Err(ResourceError::EmptyContent)
                } else {
                    Loadable::Ready(content)
                }
            }
            Err(error) => Loadable::Err(ResourceError::UnexpectedResponse(error.to_owned())),
        },
        Err(error) => Loadable::Err(ResourceError::Env(error.to_owned())),
    }
}
