use super::common::{addon_get, items_groups_update, ItemsGroup, ItemsGroupsAction, Loadable};
use crate::state_types::models::*;
use crate::state_types::msg::Internal::*;
use crate::state_types::msg::*;
use crate::state_types::*;
use crate::types::addons::{ResourceRef, ResourceRequest};
use crate::types::MetaDetail;
use lazysort::SortedBy;
use serde_derive::*;

// Cinemeta/Channels are curently limited to that many
// but in general, it's healthy to have some sort of a limit
const MAX_PER_REQUEST: usize = 50;
// The name of the extra property
const LAST_VID_IDS: &str = "lastVideosIds";

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
                        *self = Notifications::default();
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
                                        && cat.type_name == item.type_name
                                        && addon.manifest.is_supported(&ResourceRef::without_extra(
                                            "meta",
                                            &item.type_name,
                                            &item.id,
                                        ))
                                })
                                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                                .collect::<Vec<_>>();

                            // .chunks will also make sure that if relevant_items is empty,
                            // we get no chunks (so no group)
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
                                        addon_get::<Env>(addon_req.to_owned()),
                                        ItemsGroup {
                                            request: addon_req,
                                            content: Loadable::Loading,
                                        },
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
                if let Some(idx) = self.groups.iter().position(|g| g.request.eq(req)) {
                    items_groups_update::<_, Env>(
                        &mut self.groups,
                        ItemsGroupsAction::AddonResponse {
                            request: req,
                            response: result,
                        },
                    );
                    // Modify all the items so that only the new videos are left
                    if let Loadable::Ready(ref mut meta_items) = self.groups[idx].content {
                        for item in meta_items {
                            if let Some(lib_item) = ctx.library.get(&item.id) {
                                item.videos
                                    // It's not gonna be a notification if we don't have the
                                    // released date of the last watched video
                                    // NOTE: if we want to show the recent videos in the default
                                    // case, we should unwrap_or(now - THRESHOLD)
                                    // we have to get `now` somehow (environment or through a msg)
                                    // Alternatively, just set that when we add lib items
                                    .retain(|v| {
                                        lib_item
                                            .state
                                            .last_vid_released
                                            .map_or(false, |lvr| v.released > lvr)
                                    });
                            } else {
                                item.videos = vec![];
                            }
                        }
                    }
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            _ => Effects::none().unchanged(),
        }
    }
}
