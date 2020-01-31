use crate::state_types::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, ResourceAction,
    ResourceContent, ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::ctx::user::Settings as UserSettings;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionLoad, ActionPlayer, Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest};
use crate::types::{MetaDetail, Stream, SubtitlesSource, Video};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    stream: Stream,
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
                            addons: &ctx.user.content().addons,
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
                    &ctx.user.content().settings,
                );
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
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
                    &ctx.user.content().settings,
                );
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::TimeChanged { time, duration })) => {
                match &self.selected {
                    Some(Selected {
                        meta_resource_request: Some(meta_resource_request),
                        video_id: Some(video_id),
                        ..
                    }) => match ctx.library.get_item(&meta_resource_request.path.id) {
                        Some(lib_item) => {
                            let mut lib_item = lib_item.to_owned();
                            lib_item.mtime = Env::now();
                            lib_item.state.time_offset = time.to_owned();
                            lib_item.state.duration = duration.to_owned();
                            lib_item.state.video_id = Some(video_id.to_owned());
                            Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(lib_item)))
                        }
                        _ => Effects::none().unchanged(),
                    },
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
                    &ctx.user.content().settings,
                );
                meta_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn next_video_update(
    video: &mut Option<Video>,
    meta_resource: &Option<ResourceLoadable<MetaDetail>>,
    video_id: &Option<String>,
    settings: &UserSettings,
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
