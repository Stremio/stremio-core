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
use crate::types::addons::ResourceRef;
//use lazysort::SortedBy;

// Cinemeta/Channels are curently limited to that many
// but in general, it's healthy to have some sort of a limit
const MAX_PER_REQUEST: usize = 50;
// The name of the extra property
const LAST_VID_IDS: &str = "lastVideoIds";

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

                let groups = ctx
                    .content
                    .addons
                    .iter()
                    .flat_map(|addon| {
                        addon
                            .manifest
                            .catalogs
                            .iter()
                            // The catalog supports this property
                            .filter(|cat| cat.extra_iter().any(|e| e.name == LAST_VID_IDS))
                            .flat_map(move |cat| {
                                let relevant_items = lib
                                    .items
                                    .values()
                                    // The item must be eligible for notifications,
                                    // but also meta about it must be provided by the given add-on
                                    .filter(|item|
                                            !item.state.no_notif && !item.removed
                                            && addon.manifest.is_supported(&ResourceRef::without_extra("meta", &item.type_name, &item.id))
                                    )
                                    .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                                    .collect::<Vec<_>>();
                                relevant_items
                                    .chunks(MAX_PER_REQUEST)
                                    // @TODO proper group and side effect; unzip at the end
                                    .map(|items_page| items_page.to_vec())
                                    .collect::<Vec<_>>()
                            })
                    })
                    .collect::<Vec<_>>();

                dbg!(&groups);
                
                /*
                let (groups, effects) = addon_aggr_new::<Env, _>(
                    &ctx.content.addons,
                    &AggrRequest::AllOfCatalog { extra },
                );
                self.groups = groups;
                effects
                */
                // every catalog which has lastVideoIDs
                // expected response: MetaDetailed
                // so, for every catalog that has lastVideoIds required, we will check if the
                // manifest is_supports a ResourceRef to the meta
                /*
                let ids = vec!["tt7366338".to_string(), "tt2306299".to_string()];
                let path = ResourceRef::with_extra("catalog", "series", "last-videos", &[( "lastVideosIds".into(), ids.join(",") )]);
                dbg!(path.to_string());
                */
    
                // @TODO fetch groups
                Effects::none()
            },
            _ => Effects::none().unchanged(),
        }
    }
}
