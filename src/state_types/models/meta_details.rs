use super::addons::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MetaDetails {
    pub metas: Vec<ItemsGroup<Vec<MetaDetail>>>,
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
                let (metas, metas_effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(metas_resource_ref),
                );
                self.metas = metas;
                if let Some(video_id) = video_id {
                    let streams_resource_ref =
                        ResourceRef::without_extra("stream", type_name, video_id);
                    let (streams, streams_effects) = addon_aggr_new::<Env, _>(
                        &ctx.content.addons,
                        &AggrRequest::AllOfResource(streams_resource_ref),
                    );
                    self.streams = streams;
                    metas_effects.join(streams_effects)
                } else {
                    self.streams = Vec::new();
                    metas_effects
                }
            }
            _ => addon_aggr_update(&mut self.metas, msg)
                .join(addon_aggr_update(&mut self.streams, msg)),
        }
    }
}
