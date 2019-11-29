use crate::state_types::models::common::{addon_get, Loadable};
use crate::state_types::{Effects, EnvError, Environment};
use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum GroupError {
    EmptyContent,
    UnexpectedResp,
    Other(String),
}

pub type GroupContent<T> = Loadable<T, GroupError>;

#[derive(Debug, Clone, Serialize)]
pub struct Group<T> {
    pub request: ResourceRequest,
    pub content: GroupContent<T>,
}

pub enum GroupAction<'a, T, Env: Environment + 'static> {
    GroupRequested {
        request: &'a ResourceRequest,
        env: PhantomData<Env>,
    },
    GroupReplaced {
        group: Group<T>,
    },
    GroupResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn group_update<T, Env>(group: &mut Group<T>, action: GroupAction<T, Env>) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        GroupAction::GroupRequested { request, .. } => {
            if request.ne(&group.request) {
                *group = Group {
                    request: request.to_owned(),
                    content: GroupContent::Loading,
                };
                Effects::one(addon_get::<Env>(request.to_owned()))
            } else {
                Effects::none().unchanged()
            }
        }
        GroupAction::GroupReplaced { group: next_group } => {
            if next_group.request.ne(&group.request) {
                *group = next_group;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        GroupAction::GroupResponseReceived {
            request, response, ..
        } => {
            if request.eq(&group.request) {
                group.content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(content) => GroupContent::Ready(content),
                        Err(_) => GroupContent::Err(GroupError::UnexpectedResp),
                    },
                    Err(error) => GroupContent::Err(GroupError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

pub fn group_update_with_vector_content<T, Env>(
    group: &mut Group<Vec<T>>,
    action: GroupAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        GroupAction::GroupResponseReceived {
            request,
            response,
            limit,
        } => {
            if request.eq(&group.request) {
                group.content = match response {
                    Ok(response) => match <Vec<T>>::try_from(response.to_owned()) {
                        Ok(ref content) if content.is_empty() => {
                            GroupContent::Err(GroupError::EmptyContent)
                        }
                        Ok(content) => {
                            if let Some(limit) = limit {
                                GroupContent::Ready(content.into_iter().take(limit).collect())
                            } else {
                                GroupContent::Ready(content)
                            }
                        }
                        Err(_) => GroupContent::Err(GroupError::UnexpectedResp),
                    },
                    Err(error) => GroupContent::Err(GroupError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        _ => group_update::<_, Env>(group, action),
    }
}

pub enum GroupsAction<'a, T, Env: Environment + 'static> {
    GroupsRequested {
        addons: &'a [Descriptor],
        request: &'a AggrRequest<'a>,
        env: PhantomData<Env>,
    },
    GroupsReplaced {
        groups: Vec<Group<T>>,
    },
    GroupResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn groups_update<T, Env>(groups: &mut Vec<Group<T>>, action: GroupsAction<T, Env>) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        GroupsAction::GroupsRequested {
            addons, request, ..
        } => {
            let requests = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| request)
                .collect::<Vec<ResourceRequest>>();
            if requests
                .iter()
                .ne(groups.iter().map(|group| &group.request))
            {
                let (next_groups, effects) = requests
                    .iter()
                    .map(|request| {
                        (
                            Group {
                                request: request.to_owned(),
                                content: GroupContent::Loading,
                            },
                            addon_get::<Env>(request.to_owned()),
                        )
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                *groups = next_groups;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        GroupsAction::GroupsReplaced {
            groups: next_groups,
        } => {
            if next_groups
                .iter()
                .map(|group| &group.request)
                .ne(groups.iter().map(|group| &group.request))
            {
                *groups = next_groups;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        GroupsAction::GroupResponseReceived {
            request,
            response,
            limit,
        } => {
            let group_index = groups.iter().position(|group| group.request.eq(request));
            if let Some(group_index) = group_index {
                group_update::<_, Env>(
                    &mut groups[group_index],
                    GroupAction::GroupResponseReceived {
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

pub fn groups_update_with_vector_content<T, Env>(
    groups: &mut Vec<Group<Vec<T>>>,
    action: GroupsAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        GroupsAction::GroupResponseReceived {
            request,
            response,
            limit,
        } => {
            let group_index = groups.iter().position(|group| group.request.eq(request));
            if let Some(group_index) = group_index {
                group_update_with_vector_content::<_, Env>(
                    &mut groups[group_index],
                    GroupAction::GroupResponseReceived {
                        request,
                        response,
                        limit,
                    },
                )
            } else {
                Effects::none().unchanged()
            }
        }
        _ => groups_update::<_, Env>(groups, action),
    }
}
