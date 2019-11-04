use super::addons::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaPreview, Stream};
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MetaDetail {
    pub metas: Vec<ItemsGroup<Vec<MetaPreview>>>,
    pub streams: Vec<ItemsGroup<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetail {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetail {
                type_name,
                id,
                video_id,
            })) => {
                let mut detail_effects = Effects::none().unchanged();
                let metas_resource_ref = ResourceRef::without_extra("meta", type_name, id);
                let (metas, metas_effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(metas_resource_ref),
                );
                self.metas = metas;
                detail_effects = detail_effects.join(metas_effects);
                if let Some(video_id) = video_id {
                    let streams_resource_ref =
                        ResourceRef::without_extra("stream", type_name, video_id);
                    let (streams, streams_effects) = addon_aggr_new::<Env, _>(
                        &ctx.content.addons,
                        &AggrRequest::AllOfResource(streams_resource_ref),
                    );
                    self.streams = streams;
                    detail_effects = detail_effects.join(streams_effects);
                }
                detail_effects
            }
            _ => addon_aggr_update(&mut self.metas, msg)
                .join(addon_aggr_update(&mut self.streams, msg)),
        }
    }
}
