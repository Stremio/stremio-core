use super::{Ctx, LibraryLoadable};
use crate::state_types::*;
use crate::types::LibItem;
use itertools::Itertools;
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibItems {
    pub items: Vec<LibItem>,
    pub types: Vec<String>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibItems {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::UserOp(ActionUser::LibItemsByType(item_type))) => {
                if let LibraryLoadable::Ready(l) = &ctx.library {
                    self.items = l
                        .items
                        .values()
                        .filter(|x| x.type_name == *item_type && !x.removed)
                        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                        .cloned()
                        .collect();
                    self.types = l
                        .items
                        .values()
                        .filter(|x| !x.removed)
                        .map(|x| x.type_name.to_owned())
                        .unique()
                        .collect();
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
