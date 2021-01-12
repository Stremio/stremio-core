use crate::constants::WATCHED_THRESHOLD_COEF;
use crate::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, Loadable, ResourceAction,
    ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, ActionPlayer, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ResourcePath, ResourceRequest};
use crate::types::library::{
    LibraryBucket, LibraryItem, LibraryItemBehaviorHints, LibraryItemState,
};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::{MetaItem, Stream, Subtitles, Video};
use serde::{Deserialize, Serialize};
use std::cmp;

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
    pub library_item: Option<LibraryItem>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for Player {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Player(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_item_effects = match &selected.meta_request {
                    Some(meta_request) => resource_update::<E, _>(
                        &mut self.meta_item,
                        ResourceAction::ResourceRequested {
                            request: meta_request,
                        },
                    ),
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
                let library_item_effects =
                    library_item_update::<E>(&mut self.library_item, &self.meta_item, &ctx.library);
                selected_effects
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(library_item_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_item_effects = eq_update(&mut self.meta_item, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let library_item_effects =
                    library_item_update::<E>(&mut self.library_item, &self.meta_item, &ctx.library);
                selected_effects
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(library_item_effects)
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
                            cmp::min(1000, cmp::max(0, time - library_item.state.time_offset));
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
                let meta_item_effects = resource_update::<E, _>(
                    &mut self.meta_item,
                    ResourceAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let subtitles_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.subtitles,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let library_item_effects =
                    library_item_update::<E>(&mut self.library_item, &self.meta_item, &ctx.library);
                meta_item_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(library_item_effects)
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
                content: Loadable::Ready(meta_item),
                ..
            }),
        ) if settings.binge_watching => meta_item
            .videos
            .iter()
            .position(|video| video.id == *video_id)
            .and_then(|position| meta_item.videos.get(position + 1))
            .cloned(),
        _ => None,
    };
    eq_update(video, next_video)
}

fn library_item_update<E: Env>(
    library_item: &mut Option<LibraryItem>,
    meta_item: &Option<ResourceLoadable<MetaItem>>,
    library: &LibraryBucket,
) -> Effects {
    let next_library_item = match meta_item {
        Some(meta_item) => {
            let library_item = library_item
                .as_ref()
                .filter(|library_item| library_item.id == meta_item.request.path.id)
                .or_else(|| library.items.get(&meta_item.request.path.id));
            let meta_item = match meta_item {
                ResourceLoadable {
                    content: Loadable::Ready(meta_item),
                    ..
                } => Some(meta_item),
                _ => None,
            };
            match (library_item, meta_item) {
                (Some(library_item), Some(meta_item)) => Some(LibraryItem {
                    id: library_item.id.to_owned(),
                    removed: library_item.removed.to_owned(),
                    temp: library_item.temp.to_owned(),
                    ctime: library_item.ctime.to_owned(),
                    mtime: library_item.mtime.to_owned(),
                    state: library_item.state.to_owned(),
                    name: meta_item.name.to_owned(),
                    r#type: meta_item.r#type.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    behavior_hints: LibraryItemBehaviorHints {
                        default_video_id: meta_item.behavior_hints.default_video_id.to_owned(),
                    },
                }),
                (None, Some(meta_item)) => Some(LibraryItem {
                    id: meta_item.id.to_owned(),
                    removed: true,
                    temp: true,
                    ctime: Some(E::now()),
                    mtime: E::now(),
                    state: LibraryItemState::default(),
                    name: meta_item.name.to_owned(),
                    r#type: meta_item.r#type.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    behavior_hints: LibraryItemBehaviorHints {
                        default_video_id: meta_item.behavior_hints.default_video_id.to_owned(),
                    },
                }),
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
