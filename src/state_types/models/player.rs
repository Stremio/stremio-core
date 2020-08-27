use crate::constants::WATCHED_THRESHOLD_COEF;
use crate::state_types::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, ResourceAction,
    ResourceContent, ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, ActionPlayer, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ResourceRef, ResourceRequest};
use crate::types::library::{LibBucket, LibItem, LibItemState};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::{MetaItem, Stream, Subtitles, Video};
use serde::{Deserialize, Serialize};
use std::cmp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub stream: Stream,
    #[serde(default)]
    pub stream_resource_request: Option<ResourceRequest>,
    #[serde(default)]
    pub meta_resource_request: Option<ResourceRequest>,
    #[serde(default)]
    pub subtitles_resource_ref: Option<ResourceRef>,
    #[serde(default)]
    pub video_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Player {
    pub selected: Option<Selected>,
    pub meta_resource: Option<ResourceLoadable<MetaItem>>,
    pub subtitles_resources: Vec<ResourceLoadable<Vec<Subtitles>>>,
    pub next_video: Option<Video>,
    pub lib_item: Option<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for Player {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Player(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_effects = match &selected.meta_resource_request {
                    Some(meta_resource_request) => resource_update::<Env, _>(
                        &mut self.meta_resource,
                        ResourceAction::ResourceRequested {
                            request: meta_resource_request,
                        },
                    ),
                    _ => eq_update(&mut self.meta_resource, None),
                };
                let subtitles_effects = match &selected.subtitles_resource_ref {
                    Some(subtitles_resource_ref) => resources_update_with_vector_content::<Env, _>(
                        &mut self.subtitles_resources,
                        ResourcesAction::ResourcesRequested {
                            request: &AggrRequest::AllOfResource(subtitles_resource_ref.to_owned()),
                            addons: &ctx.profile.addons,
                        },
                    ),
                    _ => eq_update(&mut self.subtitles_resources, vec![]),
                };
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.meta_resource,
                    &self
                        .selected
                        .as_ref()
                        .and_then(|selected| selected.video_id.to_owned()),
                    &ctx.profile.settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library);
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(lib_item_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_effects = eq_update(&mut self.meta_resource, None);
                let subtitles_effects = eq_update(&mut self.subtitles_resources, vec![]);
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.meta_resource,
                    &self
                        .selected
                        .as_ref()
                        .and_then(|selected| selected.video_id.to_owned()),
                    &ctx.profile.settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library);
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(lib_item_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::UpdateLibraryItemState {
                time,
                duration,
            })) => {
                if let (
                    Some(Selected {
                        video_id: Some(video_id),
                        ..
                    }),
                    Some(lib_item),
                ) = (&self.selected, &mut self.lib_item)
                {
                    lib_item.state.last_watched = Some(Env::now());
                    if lib_item.state.video_id != Some(video_id.to_owned()) {
                        lib_item.state.video_id = Some(video_id.to_owned());
                        lib_item.state.overall_time_watched = lib_item
                            .state
                            .overall_time_watched
                            .saturating_add(lib_item.state.time_watched);
                        lib_item.state.time_watched = 0;
                        lib_item.state.flagged_watched = 0;
                    } else {
                        lib_item.state.time_watched = lib_item.state.time_watched.saturating_add(
                            cmp::min(1000, cmp::max(0, time - lib_item.state.time_offset)),
                        );
                    };
                    lib_item.state.time_offset = time.to_owned();
                    lib_item.state.duration = duration.to_owned();
                    if lib_item.state.flagged_watched == 0
                        && lib_item.state.time_watched as f64
                            > lib_item.state.duration as f64 * WATCHED_THRESHOLD_COEF
                    {
                        lib_item.state.flagged_watched = 1;
                        lib_item.state.times_watched =
                            lib_item.state.times_watched.saturating_add(1);
                    };
                    if lib_item.temp && lib_item.state.times_watched == 0 {
                        lib_item.removed = true;
                    };
                    if lib_item.removed {
                        lib_item.temp = true;
                    };
                };
                Effects::none()
            }
            Msg::Action(Action::Player(ActionPlayer::PushToLibrary)) => match &self.lib_item {
                Some(lib_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                    lib_item.to_owned(),
                )))
                .unchanged(),
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                let meta_effects = resource_update::<Env, _>(
                    &mut self.meta_resource,
                    ResourceAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let subtitles_effects = resources_update_with_vector_content::<Env, _>(
                    &mut self.subtitles_resources,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.meta_resource,
                    &self
                        .selected
                        .as_ref()
                        .and_then(|selected| selected.video_id.to_owned()),
                    &ctx.profile.settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library);
                meta_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(lib_item_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn next_video_update(
    video: &mut Option<Video>,
    meta_resource: &Option<ResourceLoadable<MetaItem>>,
    video_id: &Option<String>,
    settings: &ProfileSettings,
) -> Effects {
    let next_video = match (meta_resource, video_id) {
        (
            Some(ResourceLoadable {
                content: ResourceContent::Ready(meta_detail),
                ..
            }),
            Some(video_id),
        ) if settings.binge_watching => meta_detail
            .videos
            .iter()
            .position(|video| video_id == &video.id)
            .and_then(|position| meta_detail.videos.get(position + 1))
            .cloned(),
        _ => None,
    };
    if video != &next_video {
        *video = next_video;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn lib_item_update<Env: Environment>(
    lib_item: &mut Option<LibItem>,
    meta_resource: &Option<ResourceLoadable<MetaItem>>,
    library: &LibBucket,
) -> Effects {
    let next_lib_item = match meta_resource {
        Some(meta_resource) => {
            let meta_item = match meta_resource {
                ResourceLoadable {
                    content: ResourceContent::Ready(meta_item),
                    ..
                } => Some(meta_item.to_owned()),
                _ => None,
            };
            let lib_item = match lib_item {
                Some(LibItem { id, .. }) if id == &meta_resource.request.path.id => {
                    lib_item.to_owned()
                }
                _ => library.items.get(&meta_resource.request.path.id).cloned(),
            };
            match (meta_item, lib_item) {
                (Some(meta_item), Some(lib_item)) => Some(LibItem {
                    id: lib_item.id.to_owned(),
                    removed: lib_item.removed.to_owned(),
                    temp: lib_item.temp.to_owned(),
                    ctime: lib_item.ctime.to_owned(),
                    mtime: lib_item.mtime.to_owned(),
                    state: lib_item.state,
                    name: meta_item.name.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    behavior_hints: meta_item.behavior_hints,
                }),
                (Some(meta_item), None) => Some(LibItem {
                    id: meta_item.id.to_owned(),
                    removed: true,
                    temp: true,
                    ctime: Some(Env::now()),
                    mtime: Env::now(),
                    state: LibItemState::default(),
                    name: meta_item.name.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    behavior_hints: meta_item.behavior_hints,
                }),
                (None, Some(lib_item)) => Some(lib_item),
                _ => None,
            }
        }
        _ => None,
    };
    if lib_item != &next_lib_item {
        let update_library_item_effects = match lib_item {
            Some(lib_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                lib_item.to_owned(),
            )))
            .unchanged(),
            _ => Effects::none().unchanged(),
        };
        *lib_item = next_lib_item;
        Effects::none().join(update_library_item_effects)
    } else {
        Effects::none().unchanged()
    }
}
