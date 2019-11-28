use super::common::{items_groups_update, ItemsGroup, ItemsGroupsAction};
use crate::state_types::messages::Internal::*;
use crate::state_types::messages::*;
use crate::state_types::models::*;
use crate::state_types::*;
use crate::types::addons::*;
use crate::types::MetaPreview;
use serde_derive::*;
use std::marker::PhantomData;

#[derive(Debug, Clone, Default, Serialize)]
pub struct CatalogGrouped {
    pub groups: Vec<ItemsGroup<Vec<MetaPreview>>>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogGrouped {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogGrouped { extra })) => {
                items_groups_update::<_, Env>(
                    &mut self.groups,
                    ItemsGroupsAction::GroupsRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllCatalogs { extra },
                        env: PhantomData,
                    },
                )
            }
            Msg::Internal(AddonResponse(request, response)) => items_groups_update::<_, Env>(
                &mut self.groups,
                ItemsGroupsAction::AddonResponse { request, response },
            ),
            _ => Effects::none().unchanged(),
        }
    }
}
