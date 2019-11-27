use crate::state_types::models::common::{addon_get, CatalogError, Loadable};
use crate::state_types::{Effects, EnvError, Environment};
use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize)]
pub struct ItemsGroup<T> {
    pub request: ResourceRequest,
    pub content: Loadable<T, CatalogError>,
}

pub enum ItemsGroupsAction<'a, T, Env: Environment + 'static> {
    GroupsRequested {
        addons: &'a [Descriptor],
        request: &'a AggrRequest<'a>,
        env: PhantomData<Env>,
    },
    GroupsReplaced {
        items_groups: Vec<ItemsGroup<T>>,
    },
    AddonResponse {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
    },
}

pub fn items_groups_update<T, Env>(
    items_groups: &mut Vec<ItemsGroup<T>>,
    action: ItemsGroupsAction<T, Env>,
) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        ItemsGroupsAction::GroupsRequested {
            addons, request, ..
        } => {
            let (next_item_groups, effects) = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| {
                    (
                        ItemsGroup {
                            request: request.to_owned(),
                            content: Loadable::Loading,
                        },
                        addon_get::<Env>(request),
                    )
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();
            if groups_requests_changed(items_groups, &next_item_groups) {
                *items_groups = next_item_groups;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        ItemsGroupsAction::GroupsReplaced {
            items_groups: next_item_groups,
        } => {
            if groups_requests_changed(items_groups, &next_item_groups) {
                *items_groups = next_item_groups;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        ItemsGroupsAction::AddonResponse { request, response } => {
            let group_index = items_groups
                .iter()
                .position(|group| group.request.eq(request));
            if let Some(group_index) = group_index {
                let group_content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(items) => Loadable::Ready(items),
                        Err(_) => Loadable::Err(CatalogError::UnexpectedResp),
                    },
                    Err(error) => Loadable::Err(CatalogError::Other(error.to_string())),
                };
                items_groups[group_index] = ItemsGroup {
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

fn groups_requests_changed<T>(g1: &[ItemsGroup<T>], g2: &[ItemsGroup<T>]) -> bool {
    g1.iter()
        .map(|group| &group.request)
        .ne(g2.iter().map(|group| &group.request))
}
