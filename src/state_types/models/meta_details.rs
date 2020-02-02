use crate::constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME};
use crate::state_types::models::common::{
    eq_update, resources_update, resources_update_with_vector_content, ResourceContent,
    ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    meta_resource_ref: ResourceRef,
    streams_resource_ref: Option<ResourceRef>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct MetaDetails {
    pub selected: Option<Selected>,
    pub meta_resources: Vec<ResourceLoadable<MetaDetail>>,
    pub streams_resources: Vec<ResourceLoadable<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_effects = resources_update::<Env, _>(
                    &mut self.meta_resources,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllOfResource(selected.meta_resource_ref.to_owned()),
                        addons: &ctx.profile.content().addons,
                    },
                );
                let streams_effects = match &selected.streams_resource_ref {
                    Some(streams_resource_ref) => {
                        if let Some(streams_resource) = streams_resource_from_meta_resources(
                            &self.meta_resources,
                            &streams_resource_ref.id,
                        ) {
                            eq_update(&mut self.streams_resources, vec![streams_resource])
                        } else {
                            resources_update_with_vector_content::<Env, _>(
                                &mut self.streams_resources,
                                ResourcesAction::ResourcesRequested {
                                    request: &AggrRequest::AllOfResource(
                                        streams_resource_ref.to_owned(),
                                    ),
                                    addons: &ctx.profile.content().addons,
                                },
                            )
                        }
                    }
                    None => eq_update(&mut self.streams_resources, vec![]),
                };
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_effects = eq_update(&mut self.meta_resources, vec![]);
                let streams_effects = eq_update(&mut self.streams_resources, vec![]);
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource.eq(META_RESOURCE_NAME) =>
            {
                let meta_effects = resources_update::<Env, _>(
                    &mut self.meta_resources,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let streams_effects = match &self.selected {
                    Some(Selected {
                        streams_resource_ref: Some(streams_resource_ref),
                        ..
                    }) => {
                        if let Some(streams_resource) = streams_resource_from_meta_resources(
                            &self.meta_resources,
                            &streams_resource_ref.id,
                        ) {
                            eq_update(&mut self.streams_resources, vec![streams_resource])
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_effects.join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource.eq(STREAM_RESOURCE_NAME) =>
            {
                resources_update_with_vector_content::<Env, _>(
                    &mut self.streams_resources,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn streams_resource_from_meta_resources(
    meta_resources: &[ResourceLoadable<MetaDetail>],
    video_id: &str,
) -> Option<ResourceLoadable<Vec<Stream>>> {
    meta_resources
        .iter()
        .find_map(|resource| match &resource.content {
            ResourceContent::Ready(meta_detail) => Some((&resource.request, meta_detail)),
            _ => None,
        })
        .and_then(|(meta_request, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id.eq(video_id) && !video.streams.is_empty())
                .map(|video| (meta_request, &video.streams))
        })
        .map(|(meta_request, streams)| ResourceLoadable {
            request: meta_request.to_owned(),
            content: ResourceContent::Ready(streams.to_owned()),
        })
}
