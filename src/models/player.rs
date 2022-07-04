use crate::constants::{CREDITS_THRESHOLD_COEF, WATCHED_THRESHOLD_COEF};
use crate::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, Loadable, ResourceAction,
    ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, ActionPlayer, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ResourcePath, ResourceRequest};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::{MetaItem, SeriesInfo, Stream, Subtitles, Video};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::marker::PhantomData;
use stremio_watched_bitfield::WatchedBitField;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub stream: Stream,
    pub stream_request: Option<ResourceRequest>,
    pub meta_request: Option<ResourceRequest>,
    pub subtitles_path: Option<ResourcePath>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub selected: Option<Selected>,
    pub meta_item: Option<ResourceLoadable<MetaItem>>,
    pub subtitles: Vec<ResourceLoadable<Vec<Subtitles>>>,
    pub next_video: Option<Video>,
    pub series_info: Option<SeriesInfo>,
    pub library_item: Option<LibraryItem>,
    #[serde(skip_serializing)]
    pub watched: Option<WatchedBitField>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for Player {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Player(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_item_effects = match &selected.meta_request {
                    Some(meta_request) => match &mut self.meta_item {
                        Some(meta_item) => resource_update::<E, _>(
                            meta_item,
                            ResourceAction::ResourceRequested {
                                request: meta_request,
                            },
                        ),
                        _ => {
                            let mut meta_item = ResourceLoadable {
                                request: meta_request.to_owned(),
                                content: None,
                            };
                            let meta_item_effects = resource_update::<E, _>(
                                &mut meta_item,
                                ResourceAction::ResourceRequested {
                                    request: meta_request,
                                },
                            );
                            self.meta_item = Some(meta_item);
                            meta_item_effects
                        }
                    },
                    _ => eq_update(&mut self.meta_item, None),
                };
                let subtitles_effects = match &selected.subtitles_path {
                    Some(subtitles_path) => resources_update_with_vector_content::<E, _>(
                        &mut self.subtitles,
                        ResourcesAction::ResourcesRequested {
                            request: &AggrRequest::AllOfResource(subtitles_path.to_owned()),
                            addons: &ctx.profile.addons,
                        },
                    ),
                    _ => eq_update(&mut self.subtitles, vec![]),
                };
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let series_info_effects =
                    series_info_update(&mut self.series_info, &self.selected, &self.meta_item);
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_item,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update::<E>(&mut self.watched, &self.meta_item, &self.library_item);
                selected_effects
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_item_effects = eq_update(&mut self.meta_item, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let series_info_effects = eq_update(&mut self.series_info, None);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let watched_effects = eq_update(&mut self.watched, None);
                let library_item_state_effects = match &mut self.library_item {
                    Some(library_item) => {
                        if library_item.state.time_watched as f64
                            > library_item.state.duration as f64 * CREDITS_THRESHOLD_COEF
                        {
                            library_item.state.time_offset = 0;
                            if let Some(next_video) = &self.next_video {
                                library_item.state.video_id = Some(next_video.id.to_owned());
                                library_item.state.overall_time_watched = library_item
                                    .state
                                    .overall_time_watched
                                    .saturating_add(library_item.state.time_watched);
                                library_item.state.time_watched = 0;
                                library_item.state.flagged_watched = 0;
                                library_item.state.time_offset = 1;
                            }
                        }
                        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                            library_item.to_owned(),
                        )))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                };
                selected_effects
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(library_item_state_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::UpdateLibraryItemState {
                time,
                duration,
            })) => match (&self.selected, &mut self.library_item) {
                (
                    Some(Selected {
                        stream_request:
                            Some(ResourceRequest {
                                path: ResourcePath { id: video_id, .. },
                                ..
                            }),
                        ..
                    }),
                    Some(library_item),
                ) => {
                    library_item.state.last_watched = Some(E::now());
                    if library_item.state.video_id != Some(video_id.to_owned()) {
                        library_item.state.video_id = Some(video_id.to_owned());
                        library_item.state.overall_time_watched = library_item
                            .state
                            .overall_time_watched
                            .saturating_add(library_item.state.time_watched);
                        library_item.state.time_watched = 0;
                        library_item.state.flagged_watched = 0;
                    } else {
                        let time_watched =
                            cmp::min(1000, time.saturating_sub(library_item.state.time_offset));
                        library_item.state.time_watched =
                            library_item.state.time_watched.saturating_add(time_watched);
                        library_item.state.overall_time_watched = library_item
                            .state
                            .overall_time_watched
                            .saturating_add(time_watched);
                    };
                    library_item.state.time_offset = time.to_owned();
                    library_item.state.duration = duration.to_owned();
                    if library_item.state.flagged_watched == 0
                        && library_item.state.time_watched as f64
                            > library_item.state.duration as f64 * WATCHED_THRESHOLD_COEF
                    {
                        library_item.state.flagged_watched = 1;
                        library_item.state.times_watched =
                            library_item.state.times_watched.saturating_add(1);
                        if let Some(watched) = &self.watched {
                            let mut watched = watched.to_owned();
                            watched.set_video(video_id, true);
                            library_item.state.watched = Some(watched.to_string());
                        }
                    };
                    if library_item.temp && library_item.state.times_watched == 0 {
                        library_item.removed = true;
                    };
                    if library_item.removed {
                        library_item.temp = true;
                    };
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::Player(ActionPlayer::PushToLibrary)) => match &self.library_item {
                Some(library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                    library_item.to_owned(),
                )))
                .unchanged(),
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let meta_item_effects = match &mut self.meta_item {
                    Some(meta_item) => resource_update::<E, _>(
                        meta_item,
                        ResourceAction::ResourceRequestResult { request, result },
                    ),
                    _ => Effects::none().unchanged(),
                };
                let subtitles_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.subtitles,
                    ResourcesAction::ResourceRequestResult { request, result },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let series_info_effects =
                    series_info_update(&mut self.series_info, &self.selected, &self.meta_item);
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_item,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update::<E>(&mut self.watched, &self.meta_item, &self.library_item);
                meta_item_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn next_video_update(
    video: &mut Option<Video>,
    selected: &Option<Selected>,
    meta_item: &Option<ResourceLoadable<MetaItem>>,
    settings: &ProfileSettings,
) -> Effects {
    let next_video = match (selected, meta_item) {
        (
            Some(Selected {
                stream_request:
                    Some(ResourceRequest {
                        path: ResourcePath { id: video_id, .. },
                        ..
                    }),
                ..
            }),
            Some(ResourceLoadable {
                content: Some(Loadable::Ready(meta_item)),
                ..
            }),
        ) if settings.binge_watching => meta_item
            .videos
            .iter()
            .find_position(|video| video.id == *video_id)
            .and_then(|(position, current_video)| {
                meta_item
                    .videos
                    .get(position + 1)
                    .map(|next_video| (current_video, next_video))
            })
            .filter(|(current_video, next_video)| {
                let current_season = current_video
                    .series_info
                    .as_ref()
                    .map(|info| info.season)
                    .unwrap_or_default();
                let next_season = next_video
                    .series_info
                    .as_ref()
                    .map(|info| info.season)
                    .unwrap_or_default();
                next_season != 0 || current_season == next_season
            })
            .map(|(_, next_video)| next_video)
            .cloned(),
        _ => None,
    };
    eq_update(video, next_video)
}

fn series_info_update(
    series_info: &mut Option<SeriesInfo>,
    selected: &Option<Selected>,
    meta_item: &Option<ResourceLoadable<MetaItem>>,
) -> Effects {
    let next_series_info = match (selected, meta_item) {
        (
            Some(Selected {
                stream_request:
                    Some(ResourceRequest {
                        path: ResourcePath { id: video_id, .. },
                        ..
                    }),
                ..
            }),
            Some(ResourceLoadable {
                content: Some(Loadable::Ready(meta_item)),
                ..
            }),
        ) => meta_item
            .videos
            .iter()
            .find(|video| video.id == *video_id)
            .and_then(|video| video.series_info.as_ref())
            .cloned(),
        _ => None,
    };
    eq_update(series_info, next_series_info)
}

fn library_item_update<E: Env + 'static>(
    library_item: &mut Option<LibraryItem>,
    selected: &Option<Selected>,
    meta_item: &Option<ResourceLoadable<MetaItem>>,
    library: &LibraryBucket,
) -> Effects {
    let next_library_item = match selected {
        Some(Selected {
            meta_request: Some(meta_request),
            ..
        }) => {
            let library_item = library_item
                .as_ref()
                .filter(|library_item| library_item.id == meta_request.path.id)
                .or_else(|| library.items.get(&meta_request.path.id));
            let meta_item = meta_item.as_ref().and_then(|meta_item| match meta_item {
                ResourceLoadable {
                    content: Some(Loadable::Ready(meta_item)),
                    ..
                } => Some(meta_item),
                _ => None,
            });
            match (library_item, meta_item) {
                (Some(library_item), Some(meta_item)) => {
                    Some(LibraryItem::from((&meta_item.preview, library_item)))
                }
                (None, Some(meta_item)) => {
                    Some(LibraryItem::from((&meta_item.preview, PhantomData::<E>)))
                }
                (Some(library_item), None) => Some(library_item.to_owned()),
                _ => None,
            }
        }
        _ => None,
    };
    if *library_item != next_library_item {
        let update_library_item_effects = match library_item {
            Some(library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                library_item.to_owned(),
            )))
            .unchanged(),
            _ => Effects::none().unchanged(),
        };
        *library_item = next_library_item;
        Effects::none().join(update_library_item_effects)
    } else {
        Effects::none().unchanged()
    }
}

fn watched_update<E: Env>(
    watched: &mut Option<WatchedBitField>,
    meta_item: &Option<ResourceLoadable<MetaItem>>,
    library_item: &Option<LibraryItem>,
) -> Effects {
    let next_watched = meta_item
        .as_ref()
        .and_then(|meta_item| match &meta_item.content {
            Some(Loadable::Ready(meta_item)) => Some(meta_item),
            _ => None,
        })
        .and_then(|meta_item| {
            library_item
                .as_ref()
                .map(|library_item| (meta_item, library_item))
        })
        .map(|(meta_item, library_item)| library_item.state.watched_bitfield(&meta_item.videos));
    eq_update(watched, next_watched)
}
