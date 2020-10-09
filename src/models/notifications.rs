use crate::models::common::{resources_update, Loadable, ResourceLoadable, ResourcesAction};
use crate::models::ctx::Ctx;
use crate::runtime::msg::Internal::*;
use crate::runtime::msg::*;
use crate::runtime::*;
use crate::types::addon::{ResourceRef, ResourceRequest};
use crate::types::resource::MetaItem;
use futures::FutureExt;
use lazysort::SortedBy;
use serde::*;

// Cinemeta/Channels are curently limited to that many
// but in general, it's healthy to have some sort of a limit
const MAX_PER_REQUEST: usize = 50;
// The name of the extra property
const LAST_VID_IDS: &str = "lastVideosIds";

#[derive(Default, Serialize)]
pub struct Notifications {
    pub groups: Vec<ResourceLoadable<Vec<MetaItem>>>,
}
impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for Notifications {
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Notifications)) => {
                let library = &ctx.library;

                let (groups, effects): (Vec<_>, Vec<_>) = ctx
                    .profile
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
                            let relevant_items = library
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
                                .map(|items_page| -> (_, Effect) {
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
                                        ResourceRequest::new(addon.transport_url.to_owned(), path);

                                    (
                                        ResourceLoadable {
                                            request: addon_req.to_owned(),
                                            content: Loadable::Loading,
                                        },
                                        E::addon_transport(&addon_req.base)
                                            .resource(&addon_req.path)
                                            .map(move |result| {
                                                Msg::Internal(Internal::ResourceRequestResult(
                                                    addon_req,
                                                    Box::new(result),
                                                ))
                                            })
                                            .boxed_local()
                                            .into(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .unzip();

                self.groups = groups;
                Effects::many(effects)
            }
            Msg::Internal(ResourceRequestResult(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| g.request == *req) {
                    resources_update::<E, _>(
                        &mut self.groups,
                        ResourcesAction::ResourceRequestResult {
                            request: req,
                            result,
                            limit: &None,
                        },
                    );
                    // Modify all the items so that only the new videos are left
                    if let Loadable::Ready(ref mut meta_items) = self.groups[idx].content {
                        for item in meta_items {
                            if let Some(library_item) = ctx.library.items.get(&item.id) {
                                item.videos
                                    // It's not gonna be a notification if we don't have the
                                    // released date of the last watched video
                                    // NOTE: if we want to show the recent videos in the default
                                    // case, we should unwrap_or(now - THRESHOLD)
                                    // we have to get `now` somehow (environment or through a msg)
                                    // Alternatively, just set that when we add library items
                                    .retain(|v| {
                                        library_item.state.last_vid_released.map_or(false, |lvr| {
                                            v.released.map_or(false, |vr| vr > lvr)
                                        })
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
