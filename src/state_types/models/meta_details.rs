use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{
    groups_update, groups_update_with_vector_content, Group, GroupContent, GroupsAction,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde_derive::Serialize;
use std::marker::PhantomData;

const META_RESOURCE_NAME: &str = "meta";
const STREAM_RESOURCE_NAME: &str = "stream";

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    meta_resource_ref: Option<ResourceRef>,
    streams_resource_ref: Option<ResourceRef>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct MetaDetails {
    pub selected: Selected,
    pub meta_groups: Vec<Group<MetaDetail>>,
    pub streams_groups: Vec<Group<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails {
                type_name,
                id,
                video_id,
            })) => {
                let selected_effects = selected_update(
                    &mut self.selected,
                    SelectedAction::Select {
                        type_name,
                        id,
                        video_id,
                    },
                );
                let meta_effects = groups_update::<_, Env>(
                    &mut self.meta_groups,
                    GroupsAction::GroupsRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllOfResource(ResourceRef::without_extra(
                            META_RESOURCE_NAME,
                            type_name,
                            id,
                        )),
                        env: PhantomData,
                    },
                );
                let streams_effects = match video_id {
                    Some(video_id) => {
                        if let Some(streams_group) =
                            streams_group_from_meta_groups(&self.meta_groups, video_id)
                        {
                            groups_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                GroupsAction::GroupsReplaced {
                                    groups: vec![streams_group],
                                },
                            )
                        } else {
                            groups_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                GroupsAction::GroupsRequested {
                                    addons: &ctx.content.addons,
                                    request: &AggrRequest::AllOfResource(
                                        ResourceRef::without_extra(
                                            STREAM_RESOURCE_NAME,
                                            type_name,
                                            video_id,
                                        ),
                                    ),
                                    env: PhantomData,
                                },
                            )
                        }
                    }
                    None => groups_update_with_vector_content::<_, Env>(
                        &mut self.streams_groups,
                        GroupsAction::GroupsReplaced { groups: vec![] },
                    ),
                };
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response))
                if request.path.resource.eq(META_RESOURCE_NAME) =>
            {
                let meta_effects = groups_update::<_, Env>(
                    &mut self.meta_groups,
                    GroupsAction::GroupResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                );
                let streams_effects = match &self.selected {
                    Selected {
                        streams_resource_ref: Some(streams_resource_ref),
                        ..
                    } => {
                        if let Some(streams_group) = streams_group_from_meta_groups(
                            &self.meta_groups,
                            &streams_resource_ref.id,
                        ) {
                            groups_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                GroupsAction::GroupsReplaced {
                                    groups: vec![streams_group],
                                },
                            )
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_effects.join(streams_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response))
                if request.path.resource.eq(STREAM_RESOURCE_NAME) =>
            {
                groups_update_with_vector_content::<_, Env>(
                    &mut self.streams_groups,
                    GroupsAction::GroupResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                )
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

fn selected_update(selected: &mut Selected, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select {
            type_name,
            id,
            video_id: Some(video_id),
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra(
                META_RESOURCE_NAME,
                type_name,
                id,
            )),
            streams_resource_ref: Some(ResourceRef::without_extra(
                STREAM_RESOURCE_NAME,
                type_name,
                video_id,
            )),
        },
        SelectedAction::Select {
            type_name,
            id,
            video_id: None,
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra(
                META_RESOURCE_NAME,
                type_name,
                id,
            )),
            streams_resource_ref: None,
        },
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn streams_group_from_meta_groups(
    meta_groups: &[Group<MetaDetail>],
    video_id: &str,
) -> Option<Group<Vec<Stream>>> {
    meta_groups
        .iter()
        .find_map(|meta_group| match &meta_group.content {
            GroupContent::Ready(meta_detail) => Some((&meta_group.request, meta_detail)),
            _ => None,
        })
        .and_then(|(request, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id.eq(video_id) && !video.streams.is_empty())
                .map(|video| (request, &video.streams))
        })
        .map(|(request, streams)| Group {
            request: request.to_owned(),
            content: GroupContent::Ready(streams.to_owned()),
        })
}
