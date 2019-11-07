use super::addons::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest};
use crate::types::{MetaDetail, Stream};
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MetaDetails {
    pub selected: Option<(ResourceRef, Option<ResourceRef>)>,
    pub metas: Vec<ItemsGroup<MetaDetail>>,
    pub streams: Vec<ItemsGroup<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails {
                type_name,
                id,
                video_id,
            })) => {
                let metas_resource_ref = ResourceRef::without_extra("meta", type_name, id);
                let (metas, metas_effects) = addon_aggr_new::<Env, ItemsGroup<MetaDetail>>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(metas_resource_ref.to_owned()),
                );
                let metas_changed = !metas
                    .iter()
                    .map(|group| &group.req.path)
                    .eq(self.metas.iter().map(|group| &group.req.path));
                let metas_effects = if metas_changed {
                    self.metas = metas;
                    metas_effects
                } else {
                    Effects::none().unchanged()
                };
                if let Some(video_id) = video_id {
                    let streams_resource_ref =
                        ResourceRef::without_extra("stream", type_name, video_id);
                    let (streams, streams_effects) = addon_aggr_new::<Env, ItemsGroup<Vec<Stream>>>(
                        &ctx.content.addons,
                        &AggrRequest::AllOfResource(streams_resource_ref.clone()),
                    );
                    let streams_changed = !streams
                        .iter()
                        .map(|group| &group.req.path)
                        .eq(self.streams.iter().map(|group| &group.req.path));
                    self.selected = Some((metas_resource_ref, Some(streams_resource_ref)));
                    if streams_changed {
                        self.streams = streams;
                        metas_effects.join(streams_effects)
                    } else {
                        metas_effects
                    }
                } else {
                    self.selected = Some((metas_resource_ref, None));
                    self.streams = Vec::new();
                    metas_effects
                }
            }
            Msg::Internal(AddonResponse(_, _)) => {
                let metas_effects = addon_aggr_update(&mut self.metas, msg);
                let streams_effects = match &self.selected {
                    Some((_, Some(streams_resource_ref))) => {
                        let streams_from_meta = self
                            .metas
                            .iter()
                            .find_map(|group| match &group.content {
                                Loadable::Ready(meta_item) => Some((&group.req, meta_item)),
                                _ => None,
                            })
                            .map_or(None, |(req, meta_item)| {
                                meta_item
                                    .videos
                                    .iter()
                                    .find(|video| {
                                        video.id == streams_resource_ref.id
                                            && !video.streams.is_empty()
                                    })
                                    .map_or(None, |video| Some((req, &video.streams)))
                            })
                            .map_or(None, |(req, streams)| {
                                Some(vec![ItemsGroup {
                                    req: ResourceRequest {
                                        base: req.base.to_owned(),
                                        path: streams_resource_ref.to_owned(),
                                    },
                                    content: Loadable::Ready(streams.to_owned()),
                                }])
                            });
                        if let Some(streams_from_meta) = streams_from_meta {
                            self.streams = streams_from_meta;
                            Effects::none()
                        } else {
                            addon_aggr_update(&mut self.streams, msg)
                        }
                    }
                    _ => Effects::none().unchanged(),
                };

                metas_effects.join(streams_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}
