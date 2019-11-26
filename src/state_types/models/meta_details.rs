use super::addons::*;
use crate::state_types::models::Loadable;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest, ResourceResponse};
use crate::types::{MetaDetail, Stream};
use serde_derive::*;
use std::convert::TryFrom;
use std::marker::PhantomData;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    meta_resource_ref: Option<ResourceRef>,
    streams_resource_ref: Option<ResourceRef>,
}
pub type MetaGroups = Vec<ItemsGroup<MetaDetail>>;
pub type StreamsGroups = Vec<ItemsGroup<Vec<Stream>>>;

#[derive(Default, Debug, Clone, Serialize)]
pub struct MetaDetails {
    pub selected: Selected,
    pub meta_groups: MetaGroups,
    pub streams_groups: StreamsGroups,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails {
                type_name,
                id,
                video_id,
            })) => {
                let selected_effects = update(
                    &mut self.selected,
                    SelectedAction::Select {
                        type_name,
                        id,
                        video_id,
                    },
                    selected_reducer,
                    Effects::none(),
                );
                let (meta_groups, meta_effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(ResourceRef::without_extra("meta", type_name, id)),
                );
                let meta_effects = update(
                    &mut self.meta_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &meta_groups,
                        env: PhantomData,
                    },
                    items_groups_reducer::<_, Env>,
                    meta_effects,
                );
                let (streams_groups, streams_effects) = if let Some(video_id) = video_id {
                    if let Some(streams_group) =
                        streams_group_from_meta_groups(&meta_groups, video_id)
                    {
                        (vec![streams_group], Effects::none())
                    } else {
                        addon_aggr_new::<Env, _>(
                            &ctx.content.addons,
                            &AggrRequest::AllOfResource(ResourceRef::without_extra(
                                "stream", type_name, video_id,
                            )),
                        )
                    }
                } else {
                    (vec![], Effects::none())
                };
                let streams_effects = update(
                    &mut self.streams_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &streams_groups,
                        env: PhantomData,
                    },
                    items_groups_reducer::<_, Env>,
                    streams_effects,
                );
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, response)) if request.path.resource.eq("meta") => {
                let meta_effects = update(
                    &mut self.meta_groups,
                    ItemsGroupsAction::AddonResponse { request, response },
                    items_groups_reducer::<_, Env>,
                    Effects::none(),
                );
                let streams_groups = match &self.selected {
                    Selected {
                        streams_resource_ref: Some(streams_resource_ref),
                        ..
                    } => {
                        if let Some(streams_group) = streams_group_from_meta_groups(
                            &self.meta_groups,
                            &streams_resource_ref.id,
                        ) {
                            vec![streams_group]
                        } else {
                            self.streams_groups.to_owned()
                        }
                    }
                    _ => self.streams_groups.to_owned(),
                };
                let streams_effects = update(
                    &mut self.streams_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &streams_groups,
                        env: PhantomData,
                    },
                    items_groups_reducer::<_, Env>,
                    Effects::none(),
                );
                meta_effects.join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, response))
                if request.path.resource.eq("stream") =>
            {
                let streams_effects = update(
                    &mut self.streams_groups,
                    ItemsGroupsAction::AddonResponse { request, response },
                    items_groups_reducer::<_, Env>,
                    Effects::none(),
                );
                streams_effects
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Select {
        type_name: &'a String,
        id: &'a String,
        video_id: &'a Option<String>,
    },
}
fn selected_reducer(prev: &Selected, action: SelectedAction) -> (Selected, bool) {
    let next = match action {
        SelectedAction::Select {
            type_name,
            id,
            video_id: Some(video_id),
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra("meta", type_name, id)),
            streams_resource_ref: Some(ResourceRef::without_extra("stream", type_name, video_id)),
        },
        SelectedAction::Select {
            type_name,
            id,
            video_id: None,
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra("meta", type_name, id)),
            streams_resource_ref: None,
        },
    };
    let changed = prev.ne(&next);
    (next, changed)
}

type ItemsGroups<T> = Vec<ItemsGroup<T>>;
enum ItemsGroupsAction<'a, T, Env: Environment + 'static> {
    GroupsChanged {
        items_groups: &'a ItemsGroups<T>,
        env: PhantomData<Env>,
    },
    AddonResponse {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
    },
}
#[allow(clippy::ptr_arg)]
fn items_groups_reducer<T: Clone + TryFrom<ResourceResponse>, Env: Environment + 'static>(
    prev: &ItemsGroups<T>,
    action: ItemsGroupsAction<T, Env>,
) -> (ItemsGroups<T>, bool) {
    match action {
        ItemsGroupsAction::GroupsChanged { items_groups, .. } => {
            let changed = prev
                .iter()
                .map(|group| &group.req)
                .ne(items_groups.iter().map(|group| &group.req));
            let next = if changed {
                items_groups.to_owned()
            } else {
                prev.to_owned()
            };
            (next, changed)
        }
        ItemsGroupsAction::AddonResponse { request, response } => {
            let group_index = prev.iter().position(|group| group.req.eq(request));
            if let Some(group_index) = group_index {
                let group_content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(items) => Loadable::Ready(items),
                        Err(_) => Loadable::Err(CatalogError::UnexpectedResp),
                    },
                    Err(error) => Loadable::Err(CatalogError::Other(error.to_string())),
                };
                let next = &mut prev.to_owned();
                next[group_index] = ItemsGroup {
                    req: request.to_owned(),
                    content: group_content,
                };
                (next.to_owned(), true)
            } else {
                (prev.to_owned(), false)
            }
        }
    }
}

fn streams_group_from_meta_groups(
    meta_groups: &[ItemsGroup<MetaDetail>],
    video_id: &str,
) -> Option<ItemsGroup<Vec<Stream>>> {
    meta_groups
        .iter()
        .find_map(|meta_group| match &meta_group.content {
            Loadable::Ready(meta_detail) => Some((&meta_group.req, meta_detail)),
            _ => None,
        })
        .and_then(|(req, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id.eq(video_id) && !video.streams.is_empty())
                .map(|video| (req, &video.streams))
        })
        .map(|(req, streams)| ItemsGroup {
            req: req.to_owned(),
            content: Loadable::Ready(streams.to_owned()),
        })
}
