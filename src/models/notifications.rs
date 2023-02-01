use crate::{
    constants::{CATALOG_RESOURCE_NAME, URI_COMPONENT_ENCODE_SET},
    models::{
        common::{resources_update, Loadable, ResourceLoadable, ResourcesAction},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effect, EffectFuture, Effects, Env, UpdateWithCtx, EnvFutureExt,
    },
    types::{
        addon::{ExtraValue, ResourcePath, ResourceRequest},
        resource::MetaItem,
    },
};

use futures::FutureExt;
use lazysort::SortedBy;
use percent_encoding::utf8_percent_encode;
use serde::Serialize;

use super::common::eq_update;

/// Cinemeta/Channels are currently limited to that many
/// but in general, it's healthy to have some sort of a limit
const MAX_PER_REQUEST: usize = 50;

// The name of the extra property
const EXTRA_LAST_VIDEOS_IDS: &str = "lastVideosIds";

/// The last videos catalog id should be `last-videos`
///
/// See [ManifestCatalog.id](crate::types::addon::ManifestCatalog.id)
const LIST_VIDEOS_CATALOG_ID: &str = "last-videos";

/// Notifications for new video for [`LibraryItem`]s with videos
/// (i.e. movie series with new episodes).
///
/// [`LibraryItem`]: crate::types::library::LibraryItem
#[derive(Default, Serialize)]
pub struct Notifications {
    /// each addon has it's own group
    /// These groups are ordered based on the addon indices which
    /// always ordered by installation order.
    ///
    /// `Cinemeta` is installed by default so it's always the first index in the user's addon list.
    pub groups: Vec<ResourceLoadable<Vec<MetaItem>>>,
}

// const itemTime = item.last_watched_time || newest_episode.released
// sort by itemTime
impl<E: Env + 'static> UpdateWithCtx<E> for Notifications {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Notifications)) => {
                let library = &ctx.library;

                let (groups, effects): (Vec<_>, Vec<_>) = ctx
                    .profile
                    .addons
                    .iter()
                    .filter(|addon| {
                        // skip the addon if it does not support `new_episode_notifications`
                        addon.manifest.behavior_hints.new_episode_notifications
                    })
                    .flat_map(|addon| {
                        // The catalogs that support the `lastVideosIds` property
                        let viable_catalogs = addon.manifest.catalogs.iter().filter(|cat| {
                            
                            cat.extra.iter().any(|e| e.name == EXTRA_LAST_VIDEOS_IDS)
                        });

                        viable_catalogs.flat_map(move |cat| {
                            let relevant_items = library
                                .items
                                .values()
                                // The item must be eligible for notifications,
                                // but also meta about it must be provided by the given add-on
                                .filter(|item| {
                                    !item.state.no_notif
                                        && !item.removed
                                        && cat.r#type == item.r#type // for example `series`
                                        && addon.manifest.is_resource_supported(
                                            &ResourcePath::without_extra(
                                                CATALOG_RESOURCE_NAME,
                                                &item.r#type,
                                                &LIST_VIDEOS_CATALOG_ID,
                                            ),
                                        )
                                })
                                .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
                                .collect::<Vec<_>>();

                            // .chunks will also make sure that if relevant_items is empty,
                            // we get no chunks (so no group)
                            relevant_items
                                .chunks(MAX_PER_REQUEST)
                                .map(|items_page| -> (_, Effect) {
                                    let ordered_ids = {
                                        let mut ids = items_page
                                            .iter()
                                            .map(|x| x.id.as_str())
                                            .collect::<Vec<_>>();
                                            // sort the ids alphabetically
                                        ids.sort_unstable();
                                        ids
                                    };
                                    let extra_props = [ExtraValue {
                                        name: EXTRA_LAST_VIDEOS_IDS.into(),
                                        value: utf8_percent_encode(
                                            &ordered_ids.join(","),
                                            URI_COMPONENT_ENCODE_SET,
                                        )
                                        .to_string(),
                                    }];

                                    let path = ResourcePath::with_extra(
                                        CATALOG_RESOURCE_NAME,
                                        &cat.r#type,
                                        &cat.id,
                                        &extra_props,
                                    );
                                    let addon_req =
                                        ResourceRequest::new(addon.transport_url.to_owned(), path);

                                    (
                                        ResourceLoadable {
                                            request: addon_req.to_owned(),
                                            content: Some(Loadable::Loading),
                                        },
                                        EffectFuture::Concurrent(
                                            E::addon_transport(&addon_req.base)
                                                .resource(&addon_req.path)
                                                .map(move |result| {
                                                    Msg::Internal(Internal::ResourceRequestResult(
                                                        addon_req,
                                                        Box::new(result),
                                                    ))
                                                })
                                                .boxed_env(),
                                        )
                                        .into(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .unzip();


                resources_update_with_vector_content::<E, _>(
                    &mut notifications.groups,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllOfResource(resource_path.to_owned()),
                        addons: &profile.addons,
                    },
                );

                eq_update(&mut notifications.groups, groups)
            }
            Msg::Internal(Internal::ResourceRequestResult(req, result)) => {
                if let Some(idx) = self.groups.iter().position(|g| g.request == *req) {
                    resources_update::<E, _>(
                        &mut self.groups,
                        ResourcesAction::ResourceRequestResult {
                            request: req,
                            result,
                        },
                    );
                    // Modify all the items so that only the new videos are left
                    if let Some(Loadable::Ready(ref mut meta_items)) = self.groups[idx].content {
                        for item in meta_items {
                            if let Some(library_item) = ctx.library.items.get(&item.preview.id) {
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
