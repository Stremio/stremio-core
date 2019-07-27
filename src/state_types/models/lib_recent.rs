use super::{Ctx, LibraryLoadable};
use crate::state_types::*;
use crate::types::LibItem;
use lazysort::SortedBy;
use serde_derive::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct LibRecent {
    pub recent: Vec<LibItem>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibRecent {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Event(Event::CtxChanged) | Msg::Internal(Internal::LibLoaded(_)) | Msg::Event(Event::LibPersisted) => {
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
            _ => Effects::none().unchanged(),
        }
    }
}

use crate::types::Video;
//use lazysort::SortedBy;
#[derive(Debug, Clone, Default, Serialize)]
pub struct Notifications {
    pub groups: Vec<ItemsGroup<Vec<Video>>>,
}
impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for Notifications {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Notifications)) => {
                let lib = match &ctx.library {
                    LibraryLoadable::Ready(l) => l,
                    _ => {
                        self.groups = vec![];
                        return Effects::none();
                    }
                };

                // @TODO group into groups of 50, by type and maybe ID prefix?
                let relevant_items = &lib
                    .items
                    .values()
                    .filter(|item| item.type_name == "series" && !item.state.no_notif && !item.removed)
                    .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                    .take(50)
                    .collect::<Vec<_>>();
                /*
                let (groups, effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfCatalog { extra },
                );
                self.groups = groups;
                effects
                */
                /*
                let ids = vec!["tt7366338".to_string(), "tt2306299".to_string()];
                let path = ResourceRef::with_extra("catalog", "series", "last-videos", &[( "lastVideosIds".into(), ids.join(",") )]);
                dbg!(path.to_string());
                */
    
                dbg!(&relevant_items);
                // @TODO fetch groups
                Effects::none()
            },
            _ => Effects::none().unchanged(),
        }
    }
}
