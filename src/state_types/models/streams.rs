use super::addons::*;
use crate::state_types::*;
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::Stream;
use serde_derive::*;

// @TODO this will become Detail

#[derive(Debug, Clone, Default, Serialize)]
pub struct Streams {
    pub groups: Vec<ItemsGroup<Vec<Stream>>>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for Streams {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Streams { type_name, id })) => {
                let resource_ref = ResourceRef::without_extra("stream", type_name, id);
                let (groups, effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfResource(resource_ref),
                );
                self.groups = groups;
                effects
            }
            _ => addon_aggr_update(&mut self.groups, msg),
        }
    }
}
