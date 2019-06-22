use crate::types::LibItem;
use serde_derive::*;
use crate::state_types::*;
use super::{Ctx, LibraryLoadable};
use lazysort::SortedBy;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibRecent {
    pub recent: Vec<LibItem>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibRecent {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::LibLoaded(_)) | Msg::Event(Event::LibPersisted) => {
                if let LibraryLoadable::Ready(l) = &ctx.library {
                    self.recent = l
                        .items
                        .values()
                        .filter(|x| x.is_in_continue_watching())
                        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                        .take(20)
                        .cloned()
                        .collect();
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged()
        }
    }
}
