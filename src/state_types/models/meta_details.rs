use crate::constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME};
use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{
    resources_update, resources_update_with_vector_content, ResourceContent, ResourceLoadable,
    ResourcesAction,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    meta_resource_ref: Option<ResourceRef>,
    streams_resource_ref: Option<ResourceRef>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct MetaDetails {
    pub selected: Selected,
    pub meta_resources: Vec<ResourceLoadable<MetaDetail>>,
    pub streams_resources: Vec<ResourceLoadable<Vec<Stream>>>,
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
                let meta_effects = resources_update::<_, Env>(
                    &mut self.meta_resources,
                    ResourcesAction::ResourcesRequested {
                        aggr_request: &AggrRequest::AllOfResource(ResourceRef::without_extra(
                            META_RESOURCE_NAME,
                            type_name,
                            id,
                        )),
                        addons: &ctx.content.addons,
                    },
                );
                let streams_effects = match video_id {
                    Some(video_id) => {
                        if let Some(streams_resource) =
                            streams_resource_from_meta_resources(&self.meta_resources, video_id)
                        {
                            resources_update_with_vector_content::<_, Env>(
                                &mut self.streams_resources,
                                ResourcesAction::ResourcesReplaced {
                                    resources: vec![streams_resource],
                                },
                            )
                        } else {
                            resources_update_with_vector_content::<_, Env>(
                                &mut self.streams_resources,
                                ResourcesAction::ResourcesRequested {
                                    aggr_request: &AggrRequest::AllOfResource(
                                        ResourceRef::without_extra(
                                            STREAM_RESOURCE_NAME,
                                            type_name,
                                            video_id,
                                        ),
                                    ),
                                    addons: &ctx.content.addons,
                                },
                            )
                        }
                    }
                    None => resources_update_with_vector_content::<_, Env>(
                        &mut self.streams_resources,
                        ResourcesAction::ResourcesReplaced { resources: vec![] },
                    ),
                };
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response))
                if request.path.resource.eq(META_RESOURCE_NAME) =>
            {
                let meta_effects = resources_update::<_, Env>(
                    &mut self.meta_resources,
                    ResourcesAction::ResourceResponseReceived {
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
                        if let Some(streams_resource) = streams_resource_from_meta_resources(
                            &self.meta_resources,
                            &streams_resource_ref.id,
                        ) {
                            resources_update_with_vector_content::<_, Env>(
                                &mut self.streams_resources,
                                ResourcesAction::ResourcesReplaced {
                                    resources: vec![streams_resource],
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
                resources_update_with_vector_content::<_, Env>(
                    &mut self.streams_resources,
                    ResourcesAction::ResourceResponseReceived {
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

fn streams_resource_from_meta_resources(
    meta_resources: &[ResourceLoadable<MetaDetail>],
    video_id: &str,
) -> Option<ResourceLoadable<Vec<Stream>>> {
    meta_resources
        .iter()
        .find_map(|resource| match &resource.content {
            ResourceContent::Ready(meta_detail) => {
                Some((&resource.addon_name, &resource.request, meta_detail))
            }
            _ => None,
        })
        .and_then(|(addon_name, meta_request, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id.eq(video_id) && !video.streams.is_empty())
                .map(|video| (addon_name, meta_request, &video.streams))
        })
        .map(|(addon_name, meta_request, streams)| ResourceLoadable {
            addon_name: addon_name.to_owned(),
            request: meta_request.to_owned(),
            content: ResourceContent::Ready(streams.to_owned()),
        })
}
