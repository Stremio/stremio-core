use crate::constants::{META_RESOURCE_NAME, SUBTITLES_RESOURCE_NAME};
use crate::state_types::messages::{Action, ActionLoad, ActionPlayer, ActionUser, Internal, Msg};
use crate::state_types::models::common::{
    resource_update, resources_update_with_vector_content, ResourceAction, ResourceContent,
    ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::{Ctx, Settings};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef, ResourceRequest};
use crate::types::{MetaDetail, SubtitlesSource, Video};
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    transport_url: String,
    type_name: String,
    id: String,
    video_id: String,
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
            Msg::Action(Action::Load(ActionLoad::Player {
                transport_url,
                type_name,
                id,
                video_id,
            })) => {
                let selected_effects = selected_update(
                    &mut self.selected,
                    SelectedAction::Select {
                        transport_url,
                        type_name,
                        id,
                        video_id,
                    },
                );
                let meta_effects = resource_update::<_, Env>(
                    &mut self.meta_resource,
                    ResourceAction::ResourceRequested {
                        request: &ResourceRequest {
                            base: transport_url.to_owned(),
                            path: ResourceRef::without_extra(META_RESOURCE_NAME, type_name, id),
                        },
                    },
                );
                let subtitles_effects = resources_update_with_vector_content::<_, Env>(
                    &mut self.subtitles_resources,
                    ResourcesAction::ResourcesRequested {
                        aggr_request: &AggrRequest::AllOfResource(ResourceRef::without_extra(
                            SUBTITLES_RESOURCE_NAME,
                            type_name,
                            id,
                        )),
                        addons: &ctx.content.addons,
                    },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    NextVideoAction::MetaChanged {
                        meta_resource: &self.meta_resource,
                        video_id: &Some(video_id.to_owned()),
                        settings: &ctx.content.settings,
                    },
                );
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = selected_update(&mut self.selected, SelectedAction::Clear);
                let meta_effects = resource_update::<_, Env>(
                    &mut self.meta_resource,
                    ResourceAction::ResourceReplaced { resource: None },
                );
                let subtitles_effects = resources_update_with_vector_content::<_, Env>(
                    &mut self.subtitles_resources,
                    ResourcesAction::ResourcesReplaced { resources: vec![] },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    NextVideoAction::MetaChanged {
                        meta_resource: &None,
                        video_id: &None,
                        settings: &ctx.content.settings,
                    },
                );
                selected_effects
                    .join(meta_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
            }
            Msg::Action(Action::PlayerOp(ActionPlayer::TimeChanged { time, duration })) => {
                match &self.selected {
                    Some(Selected { id, video_id, .. }) => match ctx.library.get(id) {
                        Some(lib_item) => {
                            let mut lib_item = lib_item.to_owned();
                            lib_item.mtime = Env::now();
                            lib_item.state.time_offset = time.to_owned();
                            lib_item.state.duration = duration.to_owned();
                            lib_item.state.video_id = Some(video_id.to_owned());
                            Effects::msg(Msg::Action(Action::UserOp(ActionUser::LibUpdate(
                                lib_item.to_owned(),
                            ))))
                        }
                        _ => Effects::none().unchanged(),
                    },
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::PlayerOp(ActionPlayer::Ended)) => {
                // TODO update times_watched
                Effects::none().unchanged()
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                let meta_effects = resource_update::<_, Env>(
                    &mut self.meta_resource,
                    ResourceAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                );
                let subtitles_effects = resources_update_with_vector_content::<_, Env>(
                    &mut self.subtitles_resources,
                    ResourcesAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    NextVideoAction::MetaChanged {
                        meta_resource: &self.meta_resource,
                        video_id: &self
                            .selected
                            .as_ref()
                            .map(|selected| selected.video_id.to_owned()),
                        settings: &ctx.content.settings,
                    },
                );
                meta_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Select {
        transport_url: &'a String,
        type_name: &'a String,
        id: &'a String,
        video_id: &'a String,
    },
    Clear,
}

fn selected_update(selected: &mut Option<Selected>, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select {
            transport_url,
            type_name,
            id,
            video_id,
        } => Some(Selected {
            transport_url: transport_url.to_owned(),
            type_name: type_name.to_owned(),
            id: id.to_owned(),
            video_id: video_id.to_owned(),
        }),
        SelectedAction::Clear => None,
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

enum NextVideoAction<'a> {
    MetaChanged {
        meta_resource: &'a Option<ResourceLoadable<MetaDetail>>,
        video_id: &'a Option<String>,
        settings: &'a Settings,
    },
}

fn next_video_update(video: &mut Option<Video>, action: NextVideoAction) -> Effects {
    let next_video = match action {
        NextVideoAction::MetaChanged {
            meta_resource:
                Some(ResourceLoadable {
                    content: ResourceContent::Ready(meta_detail),
                    ..
                }),
            video_id: Some(video_id),
            settings,
        } if settings.autoplay_next_vid.eq("true") => meta_detail
            .videos
            .iter()
            .enumerate()
            .find(|(index, video)| video.id.eq(video_id) && index + 1 < meta_detail.videos.len())
            .map(|(.., video)| video.to_owned()),
        _ => None,
    };
    if next_video.ne(video) {
        *video = next_video;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
