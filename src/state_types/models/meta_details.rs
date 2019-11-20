use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde_derive::*;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MetaDetailsSelected {
    Meta {
        meta_resource_ref: ResourceRef,
    },
    MetaAndStreams {
        meta_resource_ref: ResourceRef,
        streams_resource_ref: ResourceRef,
    },
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MetaDetails {
    pub selected: Option<MetaDetailsSelected>,
    pub meta_groups: Vec<ItemsGroup<MetaDetail>>,
    pub streams_groups: Vec<ItemsGroup<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails {
                type_name,
                id,
                video_id,
            })) => {
                let meta_resource_ref = ResourceRef::without_extra("meta", type_name, id);
                let (meta_groups, meta_effects) = addon_aggr_new::<Env, ItemsGroup<MetaDetail>>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(meta_resource_ref.to_owned()),
                );
                let (meta_groups, meta_effects) =
                    next_groups_with_effects(&self.meta_groups, &meta_groups, meta_effects);
                let (streams_resource_ref, streams_groups, streams_effects) =
                    if let Some(video_id) = video_id {
                        let streams_resource_ref =
                            ResourceRef::without_extra("stream", type_name, video_id);
                        if let Some(streams_groups) =
                            streams_groups_from_meta_groups(&meta_groups, &video_id)
                        {
                            (Some(streams_resource_ref), streams_groups, Effects::none())
                        } else {
                            let (streams_groups, streams_effects) =
                                addon_aggr_new::<Env, ItemsGroup<Vec<Stream>>>(
                                    &ctx.content.addons,
                                    &AggrRequest::AllOfResource(streams_resource_ref.to_owned()),
                                );
                            (Some(streams_resource_ref), streams_groups, streams_effects)
                        }
                    } else {
                        (Option::None, Vec::new(), Effects::none())
                    };
                let (streams_groups, streams_effects) = next_groups_with_effects(
                    &self.streams_groups,
                    &streams_groups,
                    streams_effects,
                );
                self.selected = if let Some(streams_resource_ref) = streams_resource_ref {
                    Some(MetaDetailsSelected::MetaAndStreams {
                        meta_resource_ref,
                        streams_resource_ref,
                    })
                } else {
                    Some(MetaDetailsSelected::Meta { meta_resource_ref })
                };
                self.meta_groups = meta_groups;
                self.streams_groups = streams_groups;
                meta_effects.join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, _)) if request.path.resource.eq("meta") => {
                let meta_effects = addon_aggr_update(&mut self.meta_groups, msg);
                let streams_effects = match &self.selected {
                    Some(MetaDetailsSelected::MetaAndStreams {
                        streams_resource_ref,
                        ..
                    }) => {
                        if let Some(streams_groups) = streams_groups_from_meta_groups(
                            &self.meta_groups,
                            &streams_resource_ref.id,
                        ) {
                            self.streams_groups = streams_groups;
                            Effects::none()
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_effects.join(streams_effects)
            }
            Msg::Internal(AddonResponse(request, _)) if request.path.resource.eq("stream") => {
                addon_aggr_update(&mut self.streams_groups, msg)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn next_groups_with_effects<T: Clone>(
    prev_groups: &Vec<ItemsGroup<T>>,
    next_groups: &Vec<ItemsGroup<T>>,
    next_groups_effects: Effects,
) -> (Vec<ItemsGroup<T>>, Effects) {
    if prev_groups
        .iter()
        .map(|group| &group.req)
        .eq(next_groups.iter().map(|group| &group.req))
    {
        (prev_groups.to_owned(), Effects::none().unchanged())
    } else {
        (next_groups.to_owned(), next_groups_effects)
    }
}

fn streams_groups_from_meta_groups(
    meta_groups: &Vec<ItemsGroup<MetaDetail>>,
    video_id: &String,
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
