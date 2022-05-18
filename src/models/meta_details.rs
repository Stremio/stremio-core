use crate::constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME};
use crate::models::common::{
    eq_update, resources_update, resources_update_with_vector_content, Loadable, ResourceLoadable,
    ResourcesAction, ResourcesRequestRange,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ResourcePath};
use crate::types::resource::{MetaItem, Stream};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub meta_path: ResourcePath,
    pub stream_path: Option<ResourcePath>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaDetails {
    pub selected: Option<Selected>,
    pub meta_items: Vec<ResourceLoadable<MetaItem>>,
    pub streams: Vec<ResourceLoadable<Vec<Stream>>>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for MetaDetails {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_items_effects = resources_update::<E, _>(
                    &mut self.meta_items,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllOfResource(selected.meta_path.to_owned()),
                        addons: &ctx.profile.addons,
                        range: &Some(ResourcesRequestRange::All),
                    },
                );
                let streams_effects = match &selected.stream_path {
                    Some(stream_path) => {
                        if let Some(streams) =
                            streams_from_meta_items(&self.meta_items, &stream_path.id)
                        {
                            eq_update(&mut self.streams, vec![streams])
                        } else {
                            resources_update_with_vector_content::<E, _>(
                                &mut self.streams,
                                ResourcesAction::ResourcesRequested {
                                    request: &AggrRequest::AllOfResource(stream_path.to_owned()),
                                    addons: &ctx.profile.addons,
                                    range: &Some(ResourcesRequestRange::All),
                                },
                            )
                        }
                    }
                    None => eq_update(&mut self.streams, vec![]),
                };
                selected_effects
                    .join(meta_items_effects)
                    .join(streams_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_items_effects = eq_update(&mut self.meta_items, vec![]);
                let streams_effects = eq_update(&mut self.streams, vec![]);
                selected_effects
                    .join(meta_items_effects)
                    .join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == META_RESOURCE_NAME =>
            {
                let meta_items_effects = resources_update::<E, _>(
                    &mut self.meta_items,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let streams_effects = match &self.selected {
                    Some(Selected {
                        stream_path: Some(stream_path),
                        ..
                    }) => {
                        if let Some(streams) =
                            streams_from_meta_items(&self.meta_items, &stream_path.id)
                        {
                            eq_update(&mut self.streams, vec![streams])
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_items_effects.join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == STREAM_RESOURCE_NAME =>
            {
                resources_update_with_vector_content::<E, _>(
                    &mut self.streams,
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

fn streams_from_meta_items(
    meta_items: &[ResourceLoadable<MetaItem>],
    video_id: &str,
) -> Option<ResourceLoadable<Vec<Stream>>> {
    meta_items
        .iter()
        .find_map(|meta_item| match meta_item {
            ResourceLoadable {
                request,
                content: Some(Loadable::Ready(meta_item)),
            } => Some((request, meta_item)),
            _ => None,
        })
        .and_then(|(request, meta_item)| {
            meta_item
                .videos
                .iter()
                .find(|video| video.id == video_id && !video.streams.is_empty())
                .map(|video| (request, &video.streams))
        })
        .map(|(request, streams)| ResourceLoadable {
            request: request.to_owned(),
            content: Some(Loadable::Ready(streams.to_owned())),
        })
}
