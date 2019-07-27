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
            Msg::Event(Event::CtxChanged)
            | Msg::Internal(Internal::LibLoaded(_))
            | Msg::Event(Event::LibPersisted) => {
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

use crate::state_types::msg::Internal::*;
use crate::types::addons::{ResourceRef, ResourceRequest};
use crate::types::MetaDetail;
//use lazysort::SortedBy;

// Cinemeta/Channels are curently limited to that many
// but in general, it's healthy to have some sort of a limit
const MAX_PER_REQUEST: usize = 50;
// The name of the extra property
const LAST_VID_IDS: &str = "lastVideoIds";

#[derive(Debug, Clone, Default, Serialize)]
pub struct Notifications {
    pub groups: Vec<ItemsGroup<Vec<MetaDetail>>>,
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

                let (effects, groups): (Vec<_>, Vec<_>) = ctx
                    .content
                    .addons
                    .iter()
                    .flat_map(|addon| {
                        // The catalog supports this property
                        let viable_catalogs = addon
                            .manifest
                            .catalogs
                            .iter()
                            .filter(|cat| cat.extra_iter().any(|e| e.name == LAST_VID_IDS));

                        viable_catalogs.flat_map(move |cat| {
                            let relevant_items = lib
                                .items
                                .values()
                                // The item must be eligible for notifications,
                                // but also meta about it must be provided by the given add-on
                                .filter(|item| {
                                    !item.state.no_notif
                                        && !item.removed
                                        && addon.manifest.is_supported(&ResourceRef::without_extra(
                                            "meta",
                                            &item.type_name,
                                            &item.id,
                                        ))
                                })
                                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                                .collect::<Vec<_>>();
                            relevant_items
                                .chunks(MAX_PER_REQUEST)
                                .map(|items_page| {
                                    let ids =
                                        items_page.iter().map(|x| x.id.clone()).collect::<Vec<_>>();
                                    let extra_props = [(LAST_VID_IDS.into(), ids.join(","))];
                                    let path = ResourceRef::with_extra(
                                        "catalog",
                                        &cat.type_name,
                                        &cat.id,
                                        &extra_props,
                                    );
                                    let addon_req =
                                        ResourceRequest::new(&addon.transport_url, path);

                                    (
                                        addon_get::<Env>(&addon_req),
                                        ItemsGroup::new(addon, addon_req),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .unzip();

                self.groups = groups;
                Effects::many(effects)
            }
            Msg::Internal(AddonResponse(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| g.addon_req() == req) {
                    self.groups[idx].update(result);
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
