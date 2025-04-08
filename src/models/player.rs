use std::marker::PhantomData;
use std::ops::Div;

use base64::Engine;
use futures::{future, FutureExt, TryFutureExt};
use num::rational::Ratio;

use crate::constants::{
    BASE64, CREDITS_THRESHOLD_COEF, META_RESOURCE_NAME, PLAYER_IGNORE_SEEK_AFTER,
    STREAM_RESOURCE_NAME, SUBTITLES_RESOURCE_NAME, VIDEO_FILENAME_EXTRA_PROP,
    VIDEO_HASH_EXTRA_PROP, VIDEO_SIZE_EXTRA_PROP, WATCHED_THRESHOLD_COEF,
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
use crate::types::api::{
    fetch_api, APIRequest, APIResult, SeekLog, SeekLogRequest, SkipGapsRequest, SkipGapsResponse,
    SuccessResponse,
};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::player::{IntroData, IntroOutro};
use crate::types::profile::{Profile, Settings as ProfileSettings};
use crate::types::resource::{MetaItem, SeriesInfo, Stream, StreamSource, Subtitles, Video};
use crate::types::streams::{StreamItemState, StreamsBucket, StreamsItemKey};

use stremio_watched_bitfield::WatchedBitField;

use chrono::{DateTime, Duration, TimeZone, Utc};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use once_cell::sync::Lazy;

/// The duration that must have passed in order for a library item to be updated.
pub static PUSH_TO_LIBRARY_EVERY: Lazy<Duration> = Lazy::new(|| Duration::seconds(90));

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
    pub filename: Option<String>,
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
    pub stream_state: Option<StreamItemState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro_outro: Option<IntroOutro>,
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
    #[serde(skip_serializing)]
    pub skip_gaps: Option<(SkipGapsRequest, Loadable<SkipGapsResponse, CtxError>)>,
    /// Enable or disable Seek log collection.
    ///
    /// Default: `false` (Do not collect)
    #[serde(default, skip_serializing)]
    pub collect_seek_logs: bool,
}

impl<E: Env + 'static> UpdateWithCtx<E> for Player {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Player(selected))) => {
                let item_state_update_effects = if self
                    .selected
                    .as_ref()
                    .and_then(|selected| selected.meta_request.as_ref())
                    .map(|meta_request| &meta_request.path.id)
                    != selected
                        .meta_request
                        .as_ref()
                        .map(|meta_request| &meta_request.path.id)
                {
                    item_state_update(&mut self.library_item, self.next_video.as_ref())
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
                let stream_state_effects = eq_update(&mut self.stream_state, None);
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
                // Make sure to update the steams and in term the StreamsBucket
                // once the player loads the newly selected item
                let update_streams_effects = match (&self.selected, &self.meta_item) {
                    (Some(selected), Some(meta_item)) => {
                        Effects::msg(Msg::Internal(Internal::StreamLoaded {
                            stream: selected.stream.to_owned(),
                            stream_request: selected.stream_request.to_owned(),
                            meta_item: meta_item.to_owned(),
                        }))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                };
                let series_info_effects =
                    series_info_update(&mut self.series_info, &self.selected, &self.meta_item);
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_item,
                    &ctx.library,
                );

                let library_item_state_effects =
                    library_item_state_update(&mut self.library_item, &self.selected);

                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                let skip_gaps_effects = eq_update(&mut self.skip_gaps, None);
                let intro_outro_update_effects = intro_outro_update::<E>(
                    &mut self.intro_outro,
                    &ctx.profile,
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.skip_gaps,
                );

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
                item_state_update_effects
                    .join(selected_effects)
                    .join(meta_item_effects)
                    .join(stream_state_effects)
                    .join(video_params_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(update_streams_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(library_item_state_effects)
                    .join(watched_effects)
                    .join(skip_gaps_effects)
                    .join(intro_outro_update_effects)
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

                let item_state_update_effects =
                    item_state_update(&mut self.library_item, self.next_video.as_ref());
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
                let stream_state_effects = eq_update(&mut self.stream_state, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let next_streams_effects = eq_update(&mut self.next_streams, None);
                let next_stream_effects = eq_update(&mut self.next_stream, None);
                let series_info_effects = eq_update(&mut self.series_info, None);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let watched_effects = eq_update(&mut self.watched, None);
                let skip_gaps_effects = eq_update(&mut self.skip_gaps, None);
                self.analytics_context = None;
                self.load_time = None;
                self.loaded = false;
                self.ended = false;
                self.paused = None;

                seek_history_effects
                    .join(item_state_update_effects)
                    .join(push_to_library_effects)
                    .join(selected_effects)
                    .join(video_params_effects)
                    .join(meta_item_effects)
                    .join(stream_state_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(skip_gaps_effects)
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
                let skip_gaps_effects = skip_gaps_update::<E>(
                    &ctx.profile,
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.skip_gaps,
                );

                video_params_effects
                    .join(subtitles_effects)
                    .join(skip_gaps_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::StreamStateChanged { state })) => {
                Effects::msg(Msg::Internal(Internal::StreamStateChanged {
                    state: state.to_owned(),
                    stream_request: self
                        .selected
                        .as_ref()
                        .and_then(|selected| selected.stream_request.to_owned()),
                    meta_request: self
                        .selected
                        .as_ref()
                        .and_then(|selected| selected.meta_request.to_owned()),
                }))
                .unchanged()
            }
            Msg::Action(Action::Player(ActionPlayer::Seek {
                time,
                duration,
                device,
            })) => match (&self.selected, &mut self.library_item) {
                (
                    // make sure we have a Selected
                    Some(_selected),
                    Some(library_item),
                ) => {
                    // We might want to consider whether we want to update the LibraryItem for next video
                    // like we do for TimeChanged

                    // update the last_watched
                    library_item.state.last_watched = Some(E::now());

                    if self.collect_seek_logs {
                        // collect seek history
                        if library_item.r#type == "series" && time < &PLAYER_IGNORE_SEEK_AFTER {
                            self.seek_history.push(SeekLog {
                                from: library_item.state.time_offset,
                                to: *time,
                            });
                        }
                    }
                    // };
                    time.clone_into(&mut library_item.state.time_offset);
                    duration.clone_into(&mut library_item.state.duration);
                    // No need to check and flag the library item as watched,
                    // seeking does not update the time_watched!

                    // Nor there's a need to update removed and temp, this can only happen
                    // after we mark a LibraryItem as watched! Leave this to TimeChanged

                    // Update the analytics, we still want to keep the correct time and duration updated
                    if let Some(analytics_context) = &mut self.analytics_context {
                        library_item
                            .state
                            .video_id
                            .clone_into(&mut analytics_context.video_id);
                        analytics_context.time = Some(library_item.state.time_offset);
                        analytics_context.duration = Some(library_item.state.duration);
                        analytics_context.device_type = Some(device.to_owned());
                        analytics_context.device_name = Some(device.to_owned());
                        analytics_context.player_duration = Some(duration.to_owned());
                    };

                    // on seeking we want to make sure we send the correct Trakt events
                    let trakt_event_effects = match (self.loaded, self.paused) {
                        (true, Some(true)) => Effects::msg(Msg::Event(Event::TraktPaused {
                            context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                        }))
                        .unchanged(),
                        (true, Some(false)) => Effects::msg(Msg::Event(Event::TraktPlaying {
                            context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                        }))
                        .unchanged(),
                        _ => Effects::none(),
                    };

                    let push_to_library_effects =
                        push_to_library::<E>(&mut self.push_library_item_time, library_item);

                    trakt_event_effects.join(push_to_library_effects)
                }
                _ => Effects::none().unchanged(),
            },
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
                    // if we've selected a new video (like the next episode)
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
                        let time_watched = time.saturating_sub(library_item.state.time_offset);
                        library_item.state.time_watched =
                            library_item.state.time_watched.saturating_add(time_watched);
                        library_item.state.overall_time_watched = library_item
                            .state
                            .overall_time_watched
                            .saturating_add(time_watched);
                    };

                    // if we seek forward, time will be < time_offset
                    // this is the only thing we can guard against!
                    //
                    // for both backward and forward seeking we expect the apps to
                    // send the right actions and update the times accordingly
                    // when the state changes (from seeking to playing and vice versa)
                    if time > &library_item.state.time_offset {
                        time.clone_into(&mut library_item.state.time_offset);
                        duration.clone_into(&mut library_item.state.duration);
                    }

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
                    }

                    if library_item.temp && library_item.state.times_watched == 0 {
                        library_item.removed = true;
                    }

                    if library_item.removed {
                        library_item.temp = true;
                    }

                    if let Some(analytics_context) = &mut self.analytics_context {
                        library_item
                            .state
                            .video_id
                            .clone_into(&mut analytics_context.video_id);
                        analytics_context.time = Some(library_item.state.time_offset);
                        analytics_context.duration = Some(library_item.state.duration);
                        analytics_context.device_type = Some(device.to_owned());
                        analytics_context.device_name = Some(device.to_owned());
                        analytics_context.player_duration = Some(duration.to_owned());
                    };

                    push_to_library::<E>(&mut self.push_library_item_time, library_item)
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

                // Set time_offset to 0 as we switch to next video
                let library_item_effects = self
                    .library_item
                    .as_mut()
                    .map(|library_item| {
                        // instantly update the library item's time_offset.
                        library_item.state.time_offset = 0;

                        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                            library_item.to_owned(),
                        )))
                        .unchanged()
                    })
                    .unwrap_or(Effects::none().unchanged());

                // Load will actually take care of loading the next video
                seek_history_effects
                    .join(
                        Effects::msg(Msg::Event(Event::PlayerNextVideo {
                            context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                            is_binge_enabled: ctx.profile.settings.binge_watching,
                            is_playing_next_video: self.next_video.is_some(),
                        }))
                        .unchanged(),
                    )
                    .join(library_item_effects)
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
            Msg::Action(Action::Player(ActionPlayer::MarkVideoAsWatched(video, is_watched))) => {
                match (&self.library_item, &self.watched) {
                    (Some(library_item), Some(watched)) => {
                        let mut library_item = library_item.to_owned();
                        library_item.mark_video_as_watched::<E>(watched, video, *is_watched);

                        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(library_item)))
                            .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Player(ActionPlayer::MarkSeasonAsWatched(season, is_watched))) => {
                match (&self.library_item, &self.watched) {
                    (Some(library_item), Some(watched)) => {
                        // Find videos of given season from the meta item loadable
                        let videos = self
                            .meta_item
                            .as_ref()
                            .and_then(|meta_item| meta_item.content.as_ref())
                            .and_then(|meta_item| meta_item.ready())
                            .map(|meta_item| meta_item.videos_by_season(*season));

                        match videos {
                            Some(videos) => {
                                let mut library_item = library_item.to_owned();
                                library_item.mark_videos_as_watched::<E>(
                                    watched,
                                    videos,
                                    *is_watched,
                                );

                                Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                                    library_item,
                                )))
                                .unchanged()
                            }
                            None => Effects::none().unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_item,
                    &ctx.library,
                );

                let library_item_state_effects =
                    library_item_state_update(&mut self.library_item, &self.selected);

                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                library_item_effects
                    .join(library_item_state_effects)
                    .join(watched_effects)
            }
            Msg::Internal(Internal::StreamsChanged(_)) => {
                stream_state_update(&mut self.stream_state, &self.selected, &ctx.streams)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if self.selected.is_some() =>
            {
                let meta_item_effects = match &mut self.meta_item {
                    Some(meta_item) if request.path.resource == META_RESOURCE_NAME => {
                        resource_update::<E, _>(
                            meta_item,
                            ResourceAction::ResourceRequestResult { request, result },
                        )
                    }
                    _ => Effects::none().unchanged(),
                };

                let update_streams_effects = match (&self.selected, &self.meta_item) {
                    (Some(selected), Some(meta_item))
                        if request.path.resource == META_RESOURCE_NAME =>
                    {
                        Effects::msg(Msg::Internal(Internal::StreamLoaded {
                            stream: selected.stream.to_owned(),
                            stream_request: selected.stream_request.to_owned(),
                            meta_item: meta_item.to_owned(),
                        }))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                };

                let subtitles_effects = if request.path.resource == SUBTITLES_RESOURCE_NAME {
                    resources_update_with_vector_content::<E, _>(
                        &mut self.subtitles,
                        ResourcesAction::ResourceRequestResult { request, result },
                    )
                } else {
                    Effects::none().unchanged()
                };

                let next_streams_effects = match self.next_streams.as_mut() {
                    Some(next_streams) if request.path.resource == STREAM_RESOURCE_NAME => {
                        resource_update_with_vector_content::<E, _>(
                            next_streams,
                            ResourceAction::ResourceRequestResult { request, result },
                        )
                    }
                    _ => Effects::none().unchanged(),
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

                let library_item_state_effects =
                    library_item_state_update(&mut self.library_item, &self.selected);

                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                let skip_gaps_effects = skip_gaps_update::<E>(
                    &ctx.profile,
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.skip_gaps,
                );

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
                    .join(update_streams_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(library_item_state_effects)
                    .join(watched_effects)
                    .join(skip_gaps_effects)
            }
            Msg::Internal(Internal::SkipGapsResult(skip_gaps_request, result)) => {
                let skip_gaps_next = match result.to_owned() {
                    Ok(response) => Loadable::Ready(response),
                    Err(err) => Loadable::Err(err),
                };

                let skip_gaps_effects = eq_update(
                    &mut self.skip_gaps,
                    Some((skip_gaps_request.to_owned(), skip_gaps_next)),
                );

                let intro_outro_effects = intro_outro_update::<E>(
                    &mut self.intro_outro,
                    &ctx.profile,
                    self.selected.as_ref(),
                    self.video_params.as_ref(),
                    self.series_info.as_ref(),
                    self.library_item.as_ref(),
                    &mut self.skip_gaps,
                );

                skip_gaps_effects.join(intro_outro_effects)
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

/// We will push an [`Internal::UpdateLibraryItem`] message only if
/// at least [`PUSH_TO_LIBRARY_EVERY`] time has passed since the last update.
fn push_to_library<E: Env + 'static>(
    push_library_item_time: &mut DateTime<Utc>,
    library_item: &mut LibraryItem,
) -> Effects {
    if E::now() - *push_library_item_time >= *PUSH_TO_LIBRARY_EVERY {
        *push_library_item_time = E::now();

        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
            library_item.to_owned(),
        )))
        .unchanged()
    } else {
        Effects::none().unchanged()
    }
}

fn item_state_update(
    library_item: &mut Option<LibraryItem>,
    next_video: Option<&Video>,
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

fn stream_state_update(
    state: &mut Option<StreamItemState>,
    selected: &Option<Selected>,
    streams: &StreamsBucket,
) -> Effects {
    let next_state = match selected {
        Some(Selected {
            stream_request: Some(stream_request),
            meta_request: Some(meta_request),
            ..
        }) => {
            let key = StreamsItemKey {
                meta_id: meta_request.path.id.to_owned(),
                video_id: stream_request.path.id.to_owned(),
            };
            streams
                .items
                .get(&key)
                .and_then(|stream_item| stream_item.state.to_owned())
        }
        _ => None,
    };
    eq_update(state, next_state)
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
    let mut stream_request = match selected
        .as_ref()
        .and_then(|selected| selected.stream_request.as_ref())
    {
        Some(stream_request) => stream_request.clone(),
        None => return Effects::none().unchanged(),
    };

    match next_video {
        Some(next_video) => {
            stream_request.path.id.clone_from(&next_video.id);

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
                    let next_streams_effects = resource_update_with_vector_content::<E, _>(
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
        None => Effects::none().unchanged(),
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
            .find(|next_stream| next_stream.is_binge_match(stream))
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
            let library_item = library.items.get(&meta_request.path.id);
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
    eq_update(library_item, next_library_item)
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

fn library_item_state_update(
    library_item: &mut Option<LibraryItem>,
    selected: &Option<Selected>,
) -> Effects {
    match (library_item, selected) {
        (Some(library_item), Some(selected)) => {
            match (&selected.stream_request, &library_item.state.video_id) {
                (Some(stream_request), Some(state_video_id))
                    if stream_request.path.id != *state_video_id =>
                {
                    library_item.state.time_offset = 0;
                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(
                        library_item.to_owned(),
                    )))
                }
                _ => Effects::none().unchanged(),
            }
        }
        _ => Effects::none().unchanged(),
    }
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
                        )
                        .extend_one(&VIDEO_FILENAME_EXTRA_PROP, video_params.filename.to_owned()),
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
    let has_seeks_or_outro = !seek_history.is_empty() || matches!(outro, Some(outro) if outro > 0);
    let seek_request_effects = match (
        has_seeks_or_outro,
        selected,
        video_params,
        series_info,
        library_item,
    ) {
        (true, Some(selected), Some(video_params), Some(series_info), Some(library_item)) => {
            // live streams will not have opensubtitle hash so just relying on URL and Torrent is enough.
            let stream_source_supported = matches!(
                &selected.stream.source,
                StreamSource::Url { .. } | StreamSource::Torrent { .. }
            );
            match (
                stream_source_supported,
                selected.stream.name.as_ref(),
                video_params.hash.clone(),
            ) {
                (true, Some(stream_name), Some(opensubtitles_hash)) => {
                    let stream_name_hash = {
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
                        stream_name_hash,
                        duration: library_item.state.duration,
                        seek_history: seek_history.to_owned(),
                        skip_outro: outro.map(|time| vec![time]).unwrap_or_default(),
                    };

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
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result| Msg::Internal(Internal::SeekLogsResult(seek_log_req, result)))
            .boxed_env(),
    )
    .into()
}

fn calculate_outro(library_item: &LibraryItem, closest_duration: u64, closest_outro: u64) -> u64 {
    // will floor the result before dividing by 10 again
    let duration_diff_in_secs =
        (library_item.state.duration.abs_diff(closest_duration)).div(1000 * 10) / 10;
    tracing::debug!(
        "Player: Outro match by duration with difference of {duration_diff_in_secs} seconds"
    );
    library_item
        .state
        .duration
        .abs_diff(closest_duration.abs_diff(closest_outro))
}

fn intro_outro_update<E: Env + 'static>(
    intro_outro: &mut Option<IntroOutro>,
    profile: &Profile,
    selected: Option<&Selected>,
    video_params: Option<&VideoParams>,
    series_info: Option<&SeriesInfo>,
    library_item: Option<&LibraryItem>,
    skip_gaps: &mut Option<(SkipGapsRequest, Loadable<SkipGapsResponse, CtxError>)>,
) -> Effects {
    let skip_gaps_effects = skip_gaps_update::<E>(
        profile,
        selected,
        video_params,
        series_info,
        library_item,
        skip_gaps,
    );

    let intro_outro_effects = match (skip_gaps, library_item) {
        (Some((_, Loadable::Ready(response))), Some(library_item)) => {
            let outro_time = {
                let outro_durations = response.gaps.iter().filter_map(|(duration, skip_gaps)| {
                    skip_gaps.outro.map(|outro| (duration, outro))
                });

                let closest_duration = outro_durations.reduce(
                    |(previous_duration, previous_outro), (current_duration, current_outro)| {
                        if current_duration.abs_diff(library_item.state.duration)
                            < previous_duration.abs_diff(library_item.state.duration)
                        {
                            (current_duration, current_outro)
                        } else {
                            (previous_duration, previous_outro)
                        }
                    },
                );
                closest_duration.map(|(closest_duration, closest_outro)| {
                    calculate_outro(library_item, *closest_duration, closest_outro)
                })
            };

            let intro_time = {
                let intro_durations = response
                    .gaps
                    .iter()
                    .filter(|(_duration, skip_gaps)| !skip_gaps.seek_history.is_empty());
                let closest_duration = intro_durations.reduce(
                    |(previous_duration, previous_skip_gaps),
                     (current_duration, current_skip_gaps)| {
                        if current_duration.abs_diff(library_item.state.duration)
                            < previous_duration.abs_diff(library_item.state.duration)
                        {
                            (current_duration, current_skip_gaps)
                        } else {
                            (previous_duration, previous_skip_gaps)
                        }
                    },
                );

                closest_duration.and_then(|(closest_duration, skip_gaps)| {
                let duration_diff_in_secs = (library_item.state.duration.abs_diff(*closest_duration)).div(1000 * 10) / 10;
                tracing::trace!("Player: Intro match by duration with difference of {duration_diff_in_secs} seconds");

                let duration_ration = Ratio::new(library_item.state.duration, *closest_duration);

                // even though we checked for len() > 0 make sure we don't panic if somebody decides to remove that check!
                skip_gaps.seek_history.first().map(|seek_event| {
                    IntroData {
                        from: (duration_ration * seek_event.from).to_integer(),
                        to: (duration_ration * seek_event.to).to_integer(),
                        duration: if duration_diff_in_secs > 0 { Some(seek_event.to.abs_diff(seek_event.from)) } else { None }
                    }
                })
              })
            };

            eq_update(
                intro_outro,
                Some(IntroOutro {
                    intro: intro_time,
                    outro: outro_time,
                }),
            )
        }
        _ => Effects::none().unchanged(),
    };

    skip_gaps_effects.join(intro_outro_effects)
}

fn skip_gaps_update<E: Env + 'static>(
    profile: &Profile,
    selected: Option<&Selected>,
    video_params: Option<&VideoParams>,
    series_info: Option<&SeriesInfo>,
    library_item: Option<&LibraryItem>,
    skip_gaps: &mut Option<(SkipGapsRequest, Loadable<SkipGapsResponse, CtxError>)>,
) -> Effects {
    let active_premium = profile.auth.as_ref().and_then(|auth| {
        auth.user
            .premium_expire
            .filter(|premium_expire| premium_expire > &E::now())
            .map(|premium_expire| (premium_expire, auth.key.clone()))
    });

    let skip_gaps_request_effects = match (
        active_premium,
        selected,
        video_params,
        series_info,
        library_item,
    ) {
        (
            Some((_expires, auth_key)),
            Some(selected),
            Some(video_params),
            Some(series_info),
            Some(library_item),
        ) => {
            let stream_source_supported = matches!(
                &selected.stream.source,
                StreamSource::Url { .. } | StreamSource::Torrent { .. }
            );
            // live streams will not have opensubtitle hash so just relying on URL and Torrent is enough.
            match (
                stream_source_supported,
                selected.stream.name.as_ref(),
                video_params.hash.clone(),
            ) {
                (true, Some(stream_name), Some(opensubtitles_hash)) => {
                    let stream_name_hash = {
                        use sha2::Digest;
                        let mut sha256 = sha2::Sha256::new();
                        sha256.update(stream_name);
                        let sha256_encoded = sha256.finalize();

                        BASE64.encode(sha256_encoded)
                    };

                    let skip_gaps_request = SkipGapsRequest {
                        auth_key,
                        opensubtitles_hash,
                        item_id: library_item.id.to_owned(),
                        series_info: series_info.to_owned(),
                        stream_name_hash,
                    };

                    // no previous request, error, or different request
                    if skip_gaps.is_none()
                        || matches!(skip_gaps, Some((request, Loadable::Err(_))) | Some((request, _)) if request != &skip_gaps_request)
                    {
                        let skip_gaps_request_effects =
                            get_skip_gaps::<E>(skip_gaps_request.clone());

                        let skip_gaps_effects =
                            eq_update(skip_gaps, Some((skip_gaps_request, Loadable::Loading)));

                        Effects::one(skip_gaps_request_effects)
                            .unchanged()
                            .join(skip_gaps_effects)
                    } else {
                        Effects::none().unchanged()
                    }
                }
                _ => Effects::none().unchanged(),
            }
        }
        _ => Effects::none().unchanged(),
    };

    skip_gaps_request_effects
}

fn get_skip_gaps<E: Env + 'static>(skip_gaps_request: SkipGapsRequest) -> Effect {
    let api_request = APIRequest::SkipGaps(skip_gaps_request.clone());

    EffectFuture::Concurrent(
        fetch_api::<E, _, _, SkipGapsResponse>(&api_request)
            .map_err(CtxError::from)
            .and_then(|result| match result {
                APIResult::Ok(result) => future::ok(result),
                APIResult::Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result: Result<SkipGapsResponse, CtxError>| {
                Msg::Internal(Internal::SkipGapsResult(skip_gaps_request, result))
            })
            .boxed_env(),
    )
    .into()
}
#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        models::player::calculate_outro,
        types::{
            library::{LibraryItem, LibraryItemState},
            resource::PosterShape,
        },
    };

    #[test]
    fn test_underflow_calculate_outro() {
        let library_item = LibraryItem {
            id: "tt13622776".to_string(),
            name: "Ahsoka".to_string(),
            r#type: "series".to_string(),
            poster: None,
            poster_shape: PosterShape::Poster,
            removed: false,
            temp: true,
            ctime: None,
            mtime: Utc::now(),
            state: LibraryItemState {
                last_watched: None,
                time_watched: 999,
                time_offset: 0,
                overall_time_watched: 999,
                times_watched: 999,
                flagged_watched: 1,
                duration: 10_000,
                video_id: None,
                watched: None,
                no_notif: true,
            },
            behavior_hints: Default::default(),
        };
        {
            let closest_duration = 11000;
            let closest_outro = 1;
            assert_eq!(
                calculate_outro(&library_item, closest_duration, closest_outro),
                999
            );
        }
        {
            let closest_duration = 11000;
            let closest_outro = 12000;
            assert_eq!(
                calculate_outro(&library_item, closest_duration, closest_outro),
                9000
            );
        }
    }
}
