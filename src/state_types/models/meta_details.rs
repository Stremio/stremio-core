use super::addons::*;
use crate::state_types::models::Loadable;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest, ResourceResponse};
use crate::types::{MetaDetail, Stream};
use serde_derive::*;
use std::convert::TryFrom;

type Selected = (Option<ResourceRef>, Option<ResourceRef>);
type MetaGroups = Vec<ItemsGroup<MetaDetail>>;
type StreamsGroups = Vec<ItemsGroup<Vec<Stream>>>;

#[derive(Debug, Clone, Default, Serialize)]
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
                let (selected, selected_effects) = reduce(
                    &self.selected,
                    SelectedAction::Load {
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
                let (meta_groups, meta_effects) = reduce(
                    &self.meta_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &meta_groups,
                    },
                    items_groups_reducer,
                    meta_effects,
                );
                let (streams_groups, streams_effects) = if let Some(video_id) = video_id {
                    if let Some(streams_groups) =
                        streams_groups_from_meta_groups(&meta_groups, video_id)
                    {
                        (streams_groups, Effects::none())
                    } else {
                        let (streams_groups, streams_effects) = addon_aggr_new::<Env, _>(
                            &ctx.content.addons,
                            &AggrRequest::AllOfResource(ResourceRef::without_extra(
                                "stream", type_name, video_id,
                            )),
                        );
                        (streams_groups, streams_effects)
                    }
                } else {
                    (Vec::new(), Effects::none())
                };
                let (streams_groups, streams_effects) = reduce(
                    &self.streams_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &streams_groups,
                    },
                    items_groups_reducer,
                    streams_effects,
                );
                self.selected = selected;
                self.meta_groups = meta_groups;
                self.streams_groups = streams_groups;
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, response)) if request.path.resource.eq("meta") => {
                let (meta_groups, meta_effects) = reduce(
                    &self.meta_groups,
                    ItemsGroupsAction::AddonResponse { request, response },
                    items_groups_reducer,
                    Effects::none(),
                );
                let streams_groups = match &self.selected {
                    (Some(_), Some(streams_resource_ref)) => {
                        if let Some(streams_groups) =
                            streams_groups_from_meta_groups(&meta_groups, &streams_resource_ref.id)
                        {
                            streams_groups
                        } else {
                            self.streams_groups.to_owned()
                        }
                    }
                    _ => self.streams_groups.to_owned(),
                };
                let (streams_groups, streams_effects) = reduce(
                    &self.streams_groups,
                    ItemsGroupsAction::GroupsChanged {
                        items_groups: &streams_groups,
                    },
                    items_groups_reducer,
                    Effects::none(),
                );
                self.meta_groups = meta_groups;
                self.streams_groups = streams_groups;
                meta_effects.join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, response))
                if request.path.resource.eq("stream") =>
            {
                let (streams_groups, streams_effects) = reduce(
                    &self.streams_groups,
                    ItemsGroupsAction::AddonResponse { request, response },
                    items_groups_reducer,
                    Effects::none(),
                );
                self.streams_groups = streams_groups;
                streams_effects
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Load {
        type_name: &'a String,
        id: &'a String,
        video_id: &'a Option<String>,
    },
}
fn selected_reducer(prev: &Selected, action: SelectedAction) -> (Selected, bool) {
    let next = match action {
        SelectedAction::Load {
            type_name,
            id,
            video_id,
        } => {
            if let Some(video_id) = video_id {
                (
                    Some(ResourceRef::without_extra("meta", type_name, id)),
                    Some(ResourceRef::without_extra("stream", type_name, video_id)),
                )
            } else {
                (
                    Some(ResourceRef::without_extra("meta", type_name, id)),
                    None,
                )
            }
        }
    };
    let changed = prev.eq(&next);
    (next, changed)
}

type ItemsGroups<T> = Vec<ItemsGroup<T>>;
enum ItemsGroupsAction<'a, T> {
    GroupsChanged {
        items_groups: &'a ItemsGroups<T>,
    },
    AddonResponse {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
    },
}
#[allow(clippy::ptr_arg)]
fn items_groups_reducer<T: Clone + TryFrom<ResourceResponse>>(
    prev: &ItemsGroups<T>,
    action: ItemsGroupsAction<T>,
) -> (ItemsGroups<T>, bool) {
    match action {
        ItemsGroupsAction::GroupsChanged { items_groups } => {
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
                let next = &mut prev.clone();
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

fn streams_groups_from_meta_groups(
    meta_groups: &[ItemsGroup<MetaDetail>],
    video_id: &str,
) -> Option<Vec<ItemsGroup<Vec<Stream>>>> {
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
        .map(|(req, streams)| {
            vec![ItemsGroup {
                req: req.to_owned(),
                content: Loadable::Ready(streams.to_owned()),
            }]
        })
}
