use crate::state_types::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, ResourceAction,
    ResourceContent, ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, ActionPlayer, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::{LibBucket, LibItem, LibItemState, MetaDetail, Stream, SubtitlesSource, Video};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    stream: Stream,
    #[serde(default)]
    stream_resource_request: Option<ResourceRequest>,
    #[serde(default)]
    meta_resource_request: Option<ResourceRequest>,
    #[serde(default)]
    subtitles_resource_ref: Option<ResourceRef>,
    #[serde(default)]
    video_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Player {
    pub selected: Option<Selected>,
    pub meta_resource: Option<ResourceLoadable<MetaDetail>>,
    pub subtitles_resources: Vec<ResourceLoadable<Vec<SubtitlesSource>>>,
    pub next_video: Option<Video>,
    #[serde(skip)]
    lib_item: Option<LibItem>,
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
                            addons: &ctx.profile().addons,
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
                    &ctx.profile().settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library());
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
                    &ctx.profile().settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library());
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(lib_item_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::TimeChanged { time, duration })) => {
                match (&self.selected, &mut self.lib_item) {
                    (
                        Some(Selected {
                            video_id: Some(video_id),
                            ..
                        }),
                        Some(lib_item),
                    ) => {
                        lib_item.state.time_offset = time.to_owned();
                        lib_item.state.duration = duration.to_owned();
                        lib_item.state.video_id = Some(video_id.to_owned());
                        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                            lib_item.to_owned(),
                        )))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Player(ActionPlayer::Ended)) => {
                // TODO update times_watched
                Effects::none().unchanged()
            }
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
                    &ctx.profile().settings,
                );
                let lib_item_effects =
                    lib_item_update::<Env>(&mut self.lib_item, &self.meta_resource, &ctx.library());
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
    meta_resource: &Option<ResourceLoadable<MetaDetail>>,
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
            .position(|video| video.id.eq(video_id))
            .and_then(|position| meta_detail.videos.get(position + 1))
            .cloned(),
        _ => None,
    };
    if next_video.ne(video) {
        *video = next_video;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn lib_item_update<Env: Environment>(
    lib_item: &mut Option<LibItem>,
    meta_resource: &Option<ResourceLoadable<MetaDetail>>,
    library: &LibBucket,
) -> Effects {
    let next_lib_item = match meta_resource {
        Some(meta_resource) => match lib_item {
            Some(LibItem { id, .. }) if id == meta_resource.request.path.id => lib_item.to_owned(),
            _ => library
                .items
                .get(&meta_resource.request.path.id)
                .cloned()
                .or_else(|| match meta_resource {
                    ResourceLoadable {
                        content: ResourceContent::Ready(meta_detail),
                        ..
                    } => Some(LibItem {
                        id: meta_detail.id.to_owned(),
                        type_name: meta_detail.type_name.to_owned(),
                        name: meta_detail.name.to_owned(),
                        poster: meta_detail.poster.to_owned(),
                        background: None,
                        logo: meta_detail.logo.to_owned(),
                        poster_shape: meta_detail.poster_shape.to_owned(),
                        year: if let Some(released) = &meta_detail.released {
                            Some(released.year().to_string())
                        } else if let Some(release_info) = &meta_detail.release_info {
                            Some(release_info.to_owned())
                        } else {
                            None
                        },
                        removed: true,
                        temp: true,
                        ctime: Some(Env::now()),
                        mtime: Env::now(),
                        state: LibItemState::default(),
                    }),
                    _ => None,
                }),
        },
        _ => None,
    };
    if next_lib_item.ne(lib_item) {
        *lib_item = next_lib_item;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
