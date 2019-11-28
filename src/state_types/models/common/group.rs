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

pub enum GroupsAction<'a, T, Env: Environment + 'static> {
    GroupsRequested {
        addons: &'a [Descriptor],
        request: &'a AggrRequest<'a>,
        env: PhantomData<Env>,
    },
    GroupsReplaced {
        groups: Vec<Group<T>>,
    },
    AddonResponse {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
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
            let (next_groups, effects) = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| {
                    (
                        Group {
                            request: request.to_owned(),
                            content: GroupContent::Loading,
                        },
                        addon_get::<Env>(request),
                    )
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();
            if groups_requests_changed(groups, &next_groups) {
                *groups = next_groups;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        GroupsAction::GroupsReplaced {
            groups: next_groups,
        } => {
            if groups_requests_changed(groups, &next_groups) {
                *groups = next_groups;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        GroupsAction::AddonResponse { request, response } => {
            let group_index = groups.iter().position(|group| group.request.eq(request));
            if let Some(group_index) = group_index {
                let group_content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(content) => GroupContent::Ready(content),
                        Err(_) => GroupContent::Err(GroupError::UnexpectedResp),
                    },
                    Err(error) => GroupContent::Err(GroupError::Other(error.to_string())),
                };
                groups[group_index] = Group {
                    request: request.to_owned(),
                    content: group_content,
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

fn groups_requests_changed<T>(g1: &[Group<T>], g2: &[Group<T>]) -> bool {
    g1.iter()
        .map(|group| &group.request)
        .ne(g2.iter().map(|group| &group.request))
}
