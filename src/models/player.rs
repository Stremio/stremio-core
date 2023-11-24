use std::marker::PhantomData;

use base64::Engine;
use futures::{future, FutureExt, TryFutureExt};

use crate::constants::{
    BASE64, CREDITS_THRESHOLD_COEF, PLAYER_IGNORE_SEEK_AFTER, VIDEO_HASH_EXTRA_PROP,
    VIDEO_SIZE_EXTRA_PROP, WATCHED_THRESHOLD_COEF,
};
use crate::models::common::{
    eq_update, resource_update, resource_update_with_vector_content,
    resources_update_with_vector_content, Loadable, ResourceAction, ResourceLoadable,
    ResourcesAction,
};
use crate::models::ctx::{Ctx, CtxError};
use crate::runtime::msg::{Action, ActionLoad, ActionPlayer, Event, Internal, Msg};
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::{AggrRequest, Descriptor, ExtraExt, ResourcePath, ResourceRequest};
use crate::types::api::{fetch_api, APIRequest, APIResult, SeekLogRequest, SuccessResponse};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::{MetaItem, SeriesInfo, Stream, StreamSource, Subtitles, Video};

use stremio_watched_bitfield::WatchedBitField;

use chrono::{DateTime, Duration, TimeZone, Utc};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

lazy_static! {
    /// The duration that must have passed in order for a library item to be updated.
    pub static ref PUSH_TO_LIBRARY_EVERY: Duration = Duration::seconds(30);
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsContext {
    #[serde(rename = "libItemID")]
    pub id: Option<String>,
    #[serde(rename = "libItemType")]
    pub r#type: Option<String>,
    #[serde(rename = "libItemName")]
    pub name: Option<String>,
    #[serde(rename = "libItemVideoID")]
    pub video_id: Option<String>,
    #[serde(rename = "libItemTimeOffset")]
    pub time: Option<u64>,
    #[serde(rename = "libItemTimeDuration")]
    pub duration: Option<u64>,
    pub device_type: Option<String>,
    pub device_name: Option<String>,
    pub player_duration: Option<u64>,
    pub player_video_width: u64,
    pub player_video_height: u64,
    pub has_trakt: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoParams {
    /// Opensubtitles hash usually retrieved from a streaming server endpoint.
    ///
    /// It's used for requesting subtitles from Opensubtitles.
    pub hash: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub stream: Stream,
    pub stream_request: Option<ResourceRequest>,
    /// A request to fetch the selected [`MetaItem`].
    pub meta_request: Option<ResourceRequest>,
    pub subtitles_path: Option<ResourcePath>,
}

#[derive(Clone, Derivative, Serialize, Debug)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub selected: Option<Selected>,
    pub video_params: Option<VideoParams>,
    pub meta_item: Option<ResourceLoadable<MetaItem>>,
    pub subtitles: Vec<ResourceLoadable<Vec<Subtitles>>>,
    pub next_video: Option<Video>,
    pub next_streams: Option<ResourceLoadable<Vec<Stream>>>,
    pub next_stream: Option<Stream>,
    pub series_info: Option<SeriesInfo>,
    pub library_item: Option<LibraryItem>,
    #[serde(skip_serializing)]
    pub watched: Option<WatchedBitField>,
    #[serde(skip_serializing)]
    pub analytics_context: Option<AnalyticsContext>,
    #[serde(skip_serializing)]
    pub load_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    #[derivative(Default(value = "Utc.timestamp_opt(0, 0).unwrap()"))]
    pub push_library_item_time: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub loaded: bool,
    #[serde(skip_serializing)]
    pub ended: bool,
    #[serde(skip_serializing)]
    pub paused: Option<bool>,
    #[serde(skip_serializing)]
    pub seek_history: Vec<SeekLog>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeekLog {
    /// in milliseconds
    pub from: u64,
    /// in milliseconds
    pub to: u64,
}

impl<E: Env + 'static> UpdateWithCtx<E> for Player {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Player(selected))) => {
                let switch_to_next_video_effects = if self
                    .selected
                    .as_ref()
                    .and_then(|selected| selected.meta_request.as_ref())
                    .map(|meta_request| &meta_request.path.id)
                    != selected
                        .meta_request
                        .as_ref()
                        .map(|meta_request| &meta_request.path.id)
                {
                    switch_to_next_video(&mut self.library_item, &self.next_video)
                } else {
                    Effects::none().unchanged()
                };
                let update_streams_effects = if self.selected.as_ref().map(|selected| {
                    (
                        &selected.stream,
                        &selected.stream_request,
                        &selected.meta_request,
                    )
                }) != Some((
                    &selected.stream,
                    &selected.stream_request,
                    &selected.meta_request,
                )) {
                    Effects::msg(Msg::Internal(Internal::StreamLoaded {
                        stream: selected.stream.to_owned(),
                        stream_request: selected.stream_request.to_owned(),
                        meta_request: selected.meta_request.to_owned(),
                    }))
                    .unchanged()
                } else {
                    Effects::none().unchanged()
                };
                let selected_effects = eq_update(&mut self.selected, Some(*selected.to_owned()));
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
                let video_params_effects = eq_update(&mut self.video_params, None);
                let subtitles_effects = subtitles_update::<E>(
                    &mut self.subtitles,
                    &self.selected,
                    &self.video_params,
                    &ctx.profile.addons,
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.next_stream,
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let next_streams_effects = next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
                );
                let next_stream_effects = next_stream_update(
                    &mut self.next_stream,
                    &self.next_streams,
                    &self.selected,
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
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                // dismiss LibraryItem notification if we have a LibraryItem to begin with
                let notification_effects = match &self.library_item {
                    Some(library_item) => Effects::msg(Msg::Internal(
                        Internal::DismissNotificationItem(library_item.id.to_owned()),
                    ))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let (id, r#type, name, video_id, time, duration) = self
                    .library_item
                    .as_ref()
                    .map(|library_item| {
                        (
                            Some(library_item.id.to_owned()),
                            Some(library_item.r#type.to_owned()),
                            Some(library_item.name.to_owned()),
                            library_item.state.video_id.to_owned(),
                            Some(library_item.state.time_offset),
                            Some(library_item.state.duration),
                        )
                    })
                    .unwrap_or_default();
                self.analytics_context = Some(AnalyticsContext {
                    id,
                    r#type,
                    name,
                    video_id,
                    time,
                    duration,
                    has_trakt: ctx.profile.has_trakt::<E>(),
                    ..Default::default()
                });
                self.load_time = Some(E::now());
                self.loaded = false;
                self.ended = false;
                self.paused = None;
                switch_to_next_video_effects
                    .join(update_streams_effects)
                    .join(selected_effects)
                    .join(meta_item_effects)
                    .join(video_params_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(notification_effects)
            }
            Msg::Action(Action::Unload) => {
                let ended_effects = if !self.ended && self.selected.is_some() {
                    Effects::msg(Msg::Event(Event::PlayerStopped {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                } else {
                    Effects::none().unchanged()
                };
                let seek_history_effects = seek_update::<E>(
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.seek_history,
                    // we do not have information whether the user is currently
                    // skipping the outro by Unloading the player.
                    None,
                );

                let switch_to_next_video_effects =
                    switch_to_next_video(&mut self.library_item, &self.next_video);
                let push_to_library_effects = match &self.library_item {
                    Some(library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                        library_item.to_owned(),
                    )))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let selected_effects = eq_update(&mut self.selected, None);
                let video_params_effects = eq_update(&mut self.video_params, None);
                let meta_item_effects = eq_update(&mut self.meta_item, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let next_streams_effects = eq_update(&mut self.next_streams, None);
                let next_stream_effects = eq_update(&mut self.next_stream, None);
                let series_info_effects = eq_update(&mut self.series_info, None);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let watched_effects = eq_update(&mut self.watched, None);
                self.analytics_context = None;
                self.load_time = None;
                self.loaded = false;
                self.ended = false;
                self.paused = None;

                seek_history_effects
                    .join(switch_to_next_video_effects)
                    .join(push_to_library_effects)
                    .join(selected_effects)
                    .join(video_params_effects)
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(ended_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::VideoParamsChanged { video_params })) => {
                let video_params_effects =
                    eq_update(&mut self.video_params, video_params.to_owned());
                let subtitles_effects = subtitles_update::<E>(
                    &mut self.subtitles,
                    &self.selected,
                    &self.video_params,
                    &ctx.profile.addons,
                );
                video_params_effects.join(subtitles_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::TimeChanged {
                time,
                duration,
                device,
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
                    let seeking = library_item.state.time_offset.abs_diff(*time) > 1000;

                    // seek logging
                    if seeking
                        && library_item.r#type == "series"
                        && time < &PLAYER_IGNORE_SEEK_AFTER
                    {
                        self.seek_history.push(SeekLog {
                            from: library_item.state.time_offset,
                            to: *time,
                        });
                    }

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
                            1000.min(time.saturating_sub(library_item.state.time_offset));
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
                        if let Some(watched_bit_field) = &self.watched {
                            let mut watched_bit_field = watched_bit_field.to_owned();
                            watched_bit_field.set_video(video_id, true);
                            library_item.state.watched = Some(watched_bit_field.into());
                        }
                    };
                    if library_item.temp && library_item.state.times_watched == 0 {
                        library_item.removed = true;
                    };
                    if library_item.removed {
                        library_item.temp = true;
                    };
                    if let Some(analytics_context) = &mut self.analytics_context {
                        analytics_context.video_id = library_item.state.video_id.to_owned();
                        analytics_context.time = Some(library_item.state.time_offset);
                        analytics_context.duration = Some(library_item.state.duration);
                        analytics_context.device_type = Some(device.to_owned());
                        analytics_context.device_name = Some(device.to_owned());
                        analytics_context.player_duration = Some(duration.to_owned());
                    };
                    let trakt_event_effects = if seeking && self.loaded && self.paused.is_some() {
                        if self.paused.expect("paused is None") {
                            Effects::msg(Msg::Event(Event::TraktPaused {
                                context: self
                                    .analytics_context
                                    .as_ref()
                                    .cloned()
                                    .unwrap_or_default(),
                            }))
                            .unchanged()
                        } else {
                            Effects::msg(Msg::Event(Event::TraktPlaying {
                                context: self
                                    .analytics_context
                                    .as_ref()
                                    .cloned()
                                    .unwrap_or_default(),
                            }))
                            .unchanged()
                        }
                    } else {
                        Effects::none()
                    };

                    let push_to_library_effects =
                        if E::now() - self.push_library_item_time >= *PUSH_TO_LIBRARY_EVERY {
                            self.push_library_item_time = E::now();

                            Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                                library_item.to_owned(),
                            )))
                            .unchanged()
                        } else {
                            Effects::none().unchanged()
                        };

                    trakt_event_effects.join(push_to_library_effects)
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::Player(ActionPlayer::PausedChanged { paused }))
                if self.selected.is_some() =>
            {
                self.paused = Some(*paused);
                let trakt_event_effects = if !self.loaded {
                    self.loaded = true;
                    Effects::msg(Msg::Event(Event::PlayerPlaying {
                        load_time: self
                            .load_time
                            .map(|load_time| {
                                E::now().timestamp_millis() - load_time.timestamp_millis()
                            })
                            .unwrap_or(-1),
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                } else if *paused {
                    Effects::msg(Msg::Event(Event::TraktPaused {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                } else {
                    Effects::msg(Msg::Event(Event::TraktPlaying {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                };
                let update_library_item_effects = match &self.library_item {
                    Some(library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                        library_item.to_owned(),
                    )))
                    .unchanged(),
                    _ => Effects::none().unchanged(),
                };
                trakt_event_effects.join(update_library_item_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::NextVideo)) => {
                let seek_history_effects = seek_update::<E>(
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.seek_history,
                    // use the current LibraryItem time offset as the outro time.
                    self.library_item
                        .as_ref()
                        .map(|library_item| library_item.state.time_offset),
                );

                // Load will actually take care of loading the next video

                seek_history_effects.join(
                    Effects::msg(Msg::Event(Event::PlayerNextVideo {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                        is_binge_enabled: ctx.profile.settings.binge_watching,
                        is_playing_next_video: self.next_video.is_some(),
                    }))
                    .unchanged(),
                )
            }
            Msg::Action(Action::Player(ActionPlayer::Ended)) if self.selected.is_some() => {
                self.ended = true;
                Effects::msg(Msg::Event(Event::PlayerEnded {
                    context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    is_binge_enabled: ctx.profile.settings.binge_watching,
                    is_playing_next_video: self.next_video.is_some(),
                }))
                .unchanged()
            }
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
                let next_streams_effects = match self.next_streams.as_mut() {
                    Some(next_streams) => resource_update_with_vector_content::<E, _>(
                        next_streams,
                        ResourceAction::ResourceRequestResult { request, result },
                    ),
                    None => Effects::none().unchanged(),
                };

                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.next_stream,
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let next_streams_effects = next_streams_effects.join(next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
                ));
                let next_stream_effects = next_stream_update(
                    &mut self.next_stream,
                    &self.next_streams,
                    &self.selected,
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
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);
                let (id, r#type, name, video_id, time, duration) = self
                    .library_item
                    .as_ref()
                    .map(|library_item| {
                        (
                            Some(library_item.id.to_owned()),
                            Some(library_item.r#type.to_owned()),
                            Some(library_item.name.to_owned()),
                            library_item.state.video_id.to_owned(),
                            Some(library_item.state.time_offset),
                            Some(library_item.state.duration),
                        )
                    })
                    .unwrap_or_default();
                if let Some(analytics_context) = &mut self.analytics_context {
                    analytics_context.id = id;
                    analytics_context.r#type = r#type;
                    analytics_context.name = name;
                    analytics_context.video_id = video_id;
                    analytics_context.time = time;
                    analytics_context.duration = duration;
                };
                meta_item_effects
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                if let Some(analytics_context) = &mut self.analytics_context {
                    analytics_context.has_trakt = ctx.profile.has_trakt::<E>();
                };
                Effects::none().unchanged()
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn switch_to_next_video(
    library_item: &mut Option<LibraryItem>,
    next_video: &Option<Video>,
) -> Effects {
    match library_item {
        Some(library_item)
            if library_item.state.time_offset as f64
                > library_item.state.duration as f64 * CREDITS_THRESHOLD_COEF =>
        {
            library_item.state.time_offset = 0;
            if let Some(next_video) = next_video {
                library_item.state.video_id = Some(next_video.id.to_owned());
                library_item.state.overall_time_watched = library_item
                    .state
                    .overall_time_watched
                    .saturating_add(library_item.state.time_watched);
                library_item.state.time_watched = 0;
                library_item.state.flagged_watched = 0;
                library_item.state.time_offset = 1;
            };
        }
        _ => {}
    };
    Effects::none().unchanged()
}

fn next_video_update(
    video: &mut Option<Video>,
    stream: &Option<Stream>,
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
            .map(|(_, next_video)| {
                let mut next_video = next_video.clone();
                if let Some(stream) = stream {
                    next_video.streams = vec![stream.clone()];
                }
                next_video
            }),
        _ => None,
    };
    eq_update(video, next_video)
}

fn next_streams_update<E>(
    next_streams: &mut Option<ResourceLoadable<Vec<Stream>>>,
    next_video: &Option<Video>,
    selected: &Option<Selected>,
) -> Effects
where
    E: Env + 'static,
{
    let next_video = match next_video {
        Some(next_video) => next_video,
        None => return Effects::none().unchanged(),
    };

    let mut stream_request = match selected
        .as_ref()
        .and_then(|selected| selected.stream_request.as_ref())
    {
        Some(stream_request) => stream_request.clone(),
        None => return Effects::none().unchanged(),
    };
    // use the next video id to update the stream request
    stream_request.path.id = next_video.id.clone();

    if let Some(stream) = next_video.stream() {
        return eq_update(
            next_streams,
            Some(ResourceLoadable {
                request: stream_request,
                content: Some(Loadable::Ready(vec![stream.into_owned()])),
            }),
        );
    }

    if !next_video.streams.is_empty() {
        return eq_update(
            next_streams,
            Some(ResourceLoadable {
                request: stream_request,
                content: Some(Loadable::Ready(next_video.streams.clone())),
            }),
        );
    }

    // otherwise, fetch te next streams using a request
    match next_streams.as_mut() {
        Some(next_streams) => resource_update_with_vector_content::<E, _>(
            next_streams,
            ResourceAction::ResourceRequested {
                request: &stream_request,
            },
        ),
        None => {
            let mut new_next_streams = ResourceLoadable {
                request: stream_request.to_owned(),
                content: None,
            };
            let next_streams_effects = resource_update::<E, _>(
                &mut new_next_streams,
                ResourceAction::ResourceRequested {
                    request: &stream_request,
                },
            );
            *next_streams = Some(new_next_streams);
            next_streams_effects
        }
    }
}

fn next_stream_update(
    stream: &mut Option<Stream>,
    next_streams: &Option<ResourceLoadable<Vec<Stream>>>,
    selected: &Option<Selected>,
    settings: &ProfileSettings,
) -> Effects {
    let next_stream = match (selected, next_streams) {
        (
            Some(Selected { stream, .. }),
            Some(ResourceLoadable {
                content: Some(Loadable::Ready(streams)),
                ..
            }),
        ) if settings.binge_watching => streams
            .iter()
            .find(|Stream { behavior_hints, .. }| {
                behavior_hints.binge_group == stream.behavior_hints.binge_group
            })
            .cloned(),
        _ => None,
    };

    eq_update(stream, next_stream)
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
        let update_library_item_effects = match &library_item {
            Some(library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                library_item.to_owned(),
            )))
            .unchanged(),
            _ => Effects::none().unchanged(),
        };
        let update_next_library_item_effects = match &next_library_item {
            Some(next_library_item) => Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                next_library_item.to_owned(),
            )))
            .unchanged(),
            _ => Effects::none().unchanged(),
        };
        *library_item = next_library_item;
        Effects::none()
            .join(update_library_item_effects)
            .join(update_next_library_item_effects)
    } else {
        Effects::none().unchanged()
    }
}

fn watched_update(
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

fn subtitles_update<E: Env + 'static>(
    subtitles: &mut Vec<ResourceLoadable<Vec<Subtitles>>>,
    selected: &Option<Selected>,
    video_params: &Option<VideoParams>,
    addons: &[Descriptor],
) -> Effects {
    match (selected, video_params) {
        (
            Some(Selected {
                subtitles_path: Some(subtitles_path),
                ..
            }),
            Some(video_params),
        ) => resources_update_with_vector_content::<E, _>(
            subtitles,
            ResourcesAction::force_request(
                &AggrRequest::AllOfResource(ResourcePath {
                    extra: subtitles_path
                        .extra
                        .to_owned()
                        .extend_one(&VIDEO_HASH_EXTRA_PROP, video_params.hash.to_owned())
                        .extend_one(
                            &VIDEO_SIZE_EXTRA_PROP,
                            video_params.size.as_ref().map(|size| size.to_string()),
                        ),
                    ..subtitles_path.to_owned()
                }),
                addons,
            ),
        ),
        _ => eq_update(subtitles, vec![]),
    }
}

fn seek_update<E: Env + 'static>(
    selected: Option<&Selected>,
    video_params: Option<&VideoParams>,
    series_info: Option<&SeriesInfo>,
    library_item: Option<&LibraryItem>,
    seek_history: &mut Vec<SeekLog>,
    outro: Option<u64>,
) -> Effects {
    // todo: Remove
    tracing::info!(
        "seek update starting... selected: {}; video_params: {}, series_info: {}, library_item: {}",
        selected.is_some(),
        video_params.is_some(),
        series_info.is_some(),
        library_item.is_some()
    );

    let seek_request_effects = match (selected, video_params, series_info, library_item) {
        (Some(selected), Some(video_params), Some(series_info), Some(library_item)) => {
            // todo: Remove
            tracing::info!("seek update continues... is stream a torrent: {}; stream name: {}, video_params.hash: {}", matches!(selected.stream.source, StreamSource::Torrent { .. }), selected.stream.name.is_some(),
            video_params.hash.is_some());

            match (
                &selected.stream.source,
                selected.stream.name.as_ref(),
                video_params.hash.clone(),
            ) {
                (StreamSource::Torrent { .. }, Some(stream_name), Some(opensubtitles_hash)) => {
                    let filename_hash = {
                        use sha2::Digest;
                        let mut sha256 = sha2::Sha256::new();
                        sha256.update(stream_name);
                        let sha256_encoded = sha256.finalize();

                        BASE64.encode(sha256_encoded)
                    };

                    let seek_log_req = SeekLogRequest {
                        opensubtitles_hash,
                        item_id: library_item.id.to_owned(),
                        series_info: series_info.to_owned(),
                        filename_hash,
                        duration: library_item.state.duration,
                        seek_history: seek_history.to_owned(),
                        skip_outro: outro.map(|time| vec![time]).unwrap_or_default(),
                    };

                    // todo: Remove
                    tracing::info!("SeekLog API request: {seek_log_req:?}");

                    Effects::one(push_seek_to_api::<E>(seek_log_req)).unchanged()
                }
                _ => Effects::none().unchanged(),
            }
        }
        _ => Effects::none().unchanged(),
    };

    seek_request_effects.join(eq_update(seek_history, vec![]))
}

fn push_seek_to_api<E: Env + 'static>(seek_log_req: SeekLogRequest) -> Effect {
    let api_request = APIRequest::SeekLog(seek_log_req.clone());

    EffectFuture::Concurrent(
        fetch_api::<E, _, _, SuccessResponse>(&api_request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok { result } => future::ok(result),
                APIResult::Err { error } => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::SeekLogsResult(seek_log_req, result)))
            .boxed_env(),
    )
    .into()
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};
    use url::Url;

    use crate::{
        constants::YOUTUBE_ADDON_ID_PREFIX,
        models::common::{Loadable, ResourceLoadable},
        types::{
            addon::{ResourcePath, ResourceRequest},
            resource::{SeriesInfo, Stream, Video},
        },
        unit_tests::TestEnv,
    };

    use super::{next_streams_update, Selected};

    #[test]
    fn next_streams_update_with_a_stream_from_next_video() {
        let current_youtube_1 = format!("{YOUTUBE_ADDON_ID_PREFIX}666:1");
        let current_youtube_stream = Stream::youtube(&current_youtube_1).unwrap();
        let next_youtube_1234 = format!("{YOUTUBE_ADDON_ID_PREFIX}666:1234");
        let next_youtube_stream = Stream::youtube(&next_youtube_1234).unwrap();

        let youtube_base = "https://youtube.com"
            .parse::<Url>()
            .expect("Valid youtube url");
        let next_streams = ResourceLoadable {
            request: ResourceRequest {
                base: youtube_base.clone(),
                path: ResourcePath::without_extra("stream", "movie", &next_youtube_1234),
            },
            content: None,
        };

        let selected = Selected {
            stream: current_youtube_stream,
            stream_request: Some(ResourceRequest {
                base: youtube_base,
                path: ResourcePath::without_extra("stream", "movie", &current_youtube_1),
            }),
            meta_request: None,
            subtitles_path: None,
        };

        // Test that it should update the next_streams from the next_video if Video has one stream
        {
            let mut next_streams = Some(next_streams.clone());
            let next_video = Video {
                id: "next_video".to_owned(),
                title: "title".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                overview: Some("overview".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                streams: vec![next_youtube_stream.clone()],
                series_info: Some(SeriesInfo::default()),
                trailer_streams: vec![],
            };
            let result_effects = next_streams_update::<TestEnv>(
                &mut next_streams,
                &Some(next_video),
                &Some(selected.clone()),
            );

            assert!(result_effects.has_changed);
            assert!(result_effects.into_iter().next().is_none());
            assert_eq!(
                next_streams.as_ref().unwrap().request.path.id,
                "next_video",
                "request should contain the next youtube video"
            );

            assert_eq!(
                &Loadable::Ready(vec![next_youtube_stream.clone()]),
                next_streams
                    .as_ref()
                    .unwrap()
                    .content
                    .as_ref()
                    .expect("Should have content")
            );
        }

        // Test that it should update next_streams using all streams in Video
        {
            let another_youtube_5678 = format!("{YOUTUBE_ADDON_ID_PREFIX}666:5678");
            let another_youtube_stream = Stream::youtube(&another_youtube_5678).unwrap();
            let youtube_streams = vec![next_youtube_stream, another_youtube_stream];
            let mut next_streams = Some(next_streams.clone());
            let next_video = Video {
                id: "next_video_2".to_owned(),
                title: "title".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                overview: Some("overview".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                streams: youtube_streams.clone(),
                series_info: Some(SeriesInfo::default()),
                trailer_streams: vec![],
            };
            let result_effects = next_streams_update::<TestEnv>(
                &mut next_streams,
                &Some(next_video),
                &Some(selected.clone()),
            );

            assert!(result_effects.has_changed);
            assert!(result_effects.into_iter().next().is_none());

            assert_eq!(
                next_streams.as_ref().unwrap().request.path.id,
                "next_video_2",
                "request should contain the next youtube video"
            );

            assert_eq!(
                &Loadable::Ready(youtube_streams),
                next_streams
                    .as_ref()
                    .unwrap()
                    .content
                    .as_ref()
                    .expect("Should have content")
            );
        }

        // Test that it should make a request to get next_streams if no streams are available in Video
        {
            let mut next_streams = Some(next_streams);
            let next_video = Video {
                id: "next_video_3".to_owned(),
                title: "title".to_owned(),
                released: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
                overview: Some("overview".to_owned()),
                thumbnail: Some("thumbnail".to_owned()),
                // empty streams will cause a request to be made
                streams: vec![],
                series_info: Some(SeriesInfo::default()),
                trailer_streams: vec![],
            };
            let result_effects = next_streams_update::<TestEnv>(
                &mut next_streams,
                &Some(next_video),
                &Some(selected),
            );

            assert!(result_effects.has_changed);
            assert_eq!(1, result_effects.into_iter().count());
            assert_eq!(
                next_streams.as_ref().unwrap().request.path.id,
                "next_video_3",
                "request should contain the next youtube video"
            );
            assert_eq!(
                &Loadable::Loading,
                next_streams
                    .as_ref()
                    .unwrap()
                    .content
                    .as_ref()
                    .expect("Should have content")
            );
        }
    }
}
