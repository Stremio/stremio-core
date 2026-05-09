use std::marker::PhantomData;

use base64::Engine;
use futures::{future, FutureExt, TryFutureExt};
use url::Url;

use crate::constants::{
    BASE64, CREDITS_THRESHOLD_COEF, INTRODB_API_URL, META_RESOURCE_NAME, PLAYER_IGNORE_SEEK_AFTER,
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
use crate::runtime::{Effect, EffectFuture, Effects, Env, EnvError, EnvFutureExt, UpdateWithCtx};
use crate::types::addon::{AggrRequest, Descriptor, ExtraExt, ResourcePath, ResourceRequest};
use crate::types::api::{
    fetch_api, APIRequest, APIResult, SeekLog, SeekLogRequest, SuccessResponse,
};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::player::{
    IntroData, IntroDbRequest, IntroDbResponse, IntroOutro, IntroSegment, SegmentRange,
};
use crate::types::profile::AuthKey;
use crate::types::rating::{Rating, RatingSendRequest, RatingSendResponse};
use crate::types::resource::{
    MetaItem, SeriesInfo, Stream, StreamSource, StreamUrls, Subtitles, Video,
};
use crate::types::streams::{
    ConvertedStreamSource, StreamItemState, StreamsBucket, StreamsItemKey,
};

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
    pub stream: Option<Loadable<(StreamUrls, Stream<ConvertedStreamSource>), EnvError>>,
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
    pub intro_db: Option<(IntroDbRequest, Loadable<IntroDbResponse, CtxError>)>,
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
                // make sure we send the correct Trakt event if the model hasn't been unloaded
                let trakt_event_effects = if self.selected.is_some() {
                    Effects::msg(Msg::Event(Event::TraktPaused {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                } else {
                    Effects::none().unchanged()
                };
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

                let stream_effects = stream_update(
                    &mut self.stream,
                    self.selected.as_ref(),
                    &ctx.profile.settings.streaming_server_url,
                );

                let subtitles_effects = subtitles_update::<E>(
                    &mut self.subtitles,
                    &self.selected,
                    &self.video_params,
                    self.stream.as_ref(),
                    &ctx.profile.addons,
                );
                let next_video_effects = next_video_update(
                    &mut self.next_video,
                    &self.next_stream,
                    &self.selected,
                    &self.meta_item,
                );
                let next_streams_effects = next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
                );
                let next_stream_effects =
                    next_stream_update(&mut self.next_stream, &self.next_streams, &self.selected);
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

                let library_item_state_effects = library_item_state_update(
                    &mut self.library_item,
                    self.next_video.as_ref(),
                    &self.selected,
                );

                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                let intro_db_effects = eq_update(&mut self.intro_db, None);
                let intro_outro_update_effects = intro_outro_update::<E>(
                    &mut self.intro_outro,
                    self.selected.as_ref(),
                    self.series_info.as_ref(),
                    &mut self.intro_db,
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
                trakt_event_effects
                    .join(item_state_update_effects)
                    .join(selected_effects)
                    .join(meta_item_effects)
                    .join(stream_state_effects)
                    .join(video_params_effects)
                    .join(stream_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(update_streams_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(library_item_state_effects)
                    .join(watched_effects)
                    .join(intro_db_effects)
                    .join(intro_outro_update_effects)
                    .join(notification_effects)
            }
            Msg::Action(Action::Unload) => {
                let trakt_event_effects = if self.selected.is_some() {
                    Effects::msg(Msg::Event(Event::TraktPaused {
                        context: self.analytics_context.as_ref().cloned().unwrap_or_default(),
                    }))
                    .unchanged()
                } else {
                    Effects::none().unchanged()
                };

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
                let stream_effects = eq_update(&mut self.stream, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let next_streams_effects = eq_update(&mut self.next_streams, None);
                let next_stream_effects = eq_update(&mut self.next_stream, None);
                let series_info_effects = eq_update(&mut self.series_info, None);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let watched_effects = eq_update(&mut self.watched, None);
                let intro_db_effects = eq_update(&mut self.intro_db, None);
                self.analytics_context = None;
                self.load_time = None;
                self.loaded = false;
                self.ended = false;
                self.paused = None;

                trakt_event_effects
                    .join(seek_history_effects)
                    .join(item_state_update_effects)
                    .join(push_to_library_effects)
                    .join(selected_effects)
                    .join(video_params_effects)
                    .join(stream_effects)
                    .join(meta_item_effects)
                    .join(stream_state_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(next_stream_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(intro_db_effects)
                    .join(ended_effects)
            }
            Msg::Action(Action::Player(ActionPlayer::VideoParamsChanged { video_params })) => {
                let video_params_effects =
                    eq_update(&mut self.video_params, video_params.to_owned());

                let subtitles_effects = subtitles_update::<E>(
                    &mut self.subtitles,
                    &self.selected,
                    &self.video_params,
                    self.stream.as_ref(),
                    &ctx.profile.addons,
                );
                let intro_db_effects = intro_db_update::<E>(
                    self.selected.as_ref(),
                    self.series_info.as_ref(),
                    &mut self.intro_db,
                );

                video_params_effects
                    .join(subtitles_effects)
                    .join(intro_db_effects)
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

                    // Watched threshold for marking an episode/movie as watched
                    let should_send_watched = if library_item.state.flagged_watched == 0
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

                        true
                    } else {
                        false
                    };

                    // send Watched for MetaDetail id
                    // single episode should mark the item as watched.
                    let send_watched_effects = match (
                        should_send_watched,
                        ctx.profile.auth_key(),
                        self.selected.as_ref(),
                    ) {
                        (
                            true,
                            Some(auth_key),
                            Some(Selected {
                                meta_request:
                                    Some(ResourceRequest {
                                        path: meta_path, ..
                                    }),
                                ..
                            }),
                        ) => Effects::one(send_watched::<E>(auth_key.to_owned(), meta_path))
                            .unchanged(),
                        _ => Effects::none().unchanged(),
                    };

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

                    send_watched_effects.join(push_to_library::<E>(
                        &mut self.push_library_item_time,
                        library_item,
                    ))
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

                // Set time_offset to 0 or 1 as we switch to next video
                let library_item_effects = self
                    .library_item
                    .as_mut()
                    .map(|library_item| {
                        // instantly update the library item's time_offset.
                        library_item.state.time_offset =
                            if self.next_video.is_some() { 1 } else { 0 };

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

                let library_item_state_effects = library_item_state_update(
                    &mut self.library_item,
                    self.next_video.as_ref(),
                    &self.selected,
                );

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
                );

                let next_streams_effects = next_streams_effects.join(next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
                ));

                let next_stream_effects =
                    next_stream_update(&mut self.next_stream, &self.next_streams, &self.selected);

                let series_info_effects =
                    series_info_update(&mut self.series_info, &self.selected, &self.meta_item);
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_item,
                    &ctx.library,
                );

                let library_item_state_effects = library_item_state_update(
                    &mut self.library_item,
                    self.next_video.as_ref(),
                    &self.selected,
                );

                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_item, &self.library_item);

                let intro_db_effects = intro_db_update::<E>(
                    self.selected.as_ref(),
                    self.series_info.as_ref(),
                    &mut self.intro_db,
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
                    .join(intro_db_effects)
            }
            Msg::Internal(Internal::IntroDbResult(intro_db_request, result)) => {
                let intro_db_next = match result.to_owned() {
                    Ok(response) => Loadable::Ready(response),
                    Err(err) => Loadable::Err(err),
                };

                let intro_db_effects = eq_update(
                    &mut self.intro_db,
                    Some((intro_db_request.to_owned(), intro_db_next)),
                );

                let intro_outro_effects = intro_outro_update::<E>(
                    &mut self.intro_outro,
                    self.selected.as_ref(),
                    self.series_info.as_ref(),
                    &mut self.intro_db,
                );

                intro_db_effects.join(intro_outro_effects)
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
        ) => meta_item
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
) -> Effects {
    let next_stream = match (selected, next_streams) {
        (
            Some(Selected { stream, .. }),
            Some(ResourceLoadable {
                content: Some(Loadable::Ready(streams)),
                ..
            }),
        ) => streams
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
    next_video: Option<&Video>,
    selected: &Option<Selected>,
) -> Effects {
    match (library_item, selected) {
        (Some(library_item), Some(selected)) => {
            match (&selected.stream_request, &library_item.state.video_id) {
                (Some(stream_request), Some(state_video_id))
                    if stream_request.path.id != *state_video_id =>
                {
                    library_item.state.time_offset = if next_video.is_some() { 1 } else { 0 };
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

fn stream_update(
    stream: &mut Option<Loadable<(StreamUrls, Stream<ConvertedStreamSource>), EnvError>>,
    selected: Option<&Selected>,
    streaming_server_url: &Url,
) -> Effects {
    match selected {
        Some(selected) => {
            let next_stream = match selected.stream.convert(Some(&streaming_server_url)) {
                Ok(converted_stream) => {
                    let stream_urls =
                        StreamUrls::new(converted_stream.clone(), Some(&streaming_server_url));

                    Loadable::Ready((stream_urls, converted_stream))
                }
                Err(err) => Loadable::Err(err),
            };

            eq_update(stream, Some(next_stream))
        }
        None => eq_update(stream, None),
    }
}

fn subtitles_update<E: Env + 'static>(
    subtitles: &mut Vec<ResourceLoadable<Vec<Subtitles>>>,
    selected: &Option<Selected>,
    video_params: &Option<VideoParams>,
    stream: Option<&Loadable<(StreamUrls, Stream<ConvertedStreamSource>), EnvError>>,
    addons: &[Descriptor],
) -> Effects {
    match (selected, stream) {
        (
            Some(Selected {
                subtitles_path: Some(subtitles_path),
                ..
            }),
            Some(Loadable::Ready((_stream_urls, converted_stream))),
        ) => {
            let video_hash = converted_stream
                .behavior_hints
                .video_hash
                .clone()
                .or_else(|| {
                    video_params
                        .as_ref()
                        .and_then(|video_params| video_params.hash.to_owned())
                });
            let video_size = converted_stream.behavior_hints.video_size.or_else(|| {
                video_params
                    .as_ref()
                    .and_then(|video_params| video_params.size)
            });
            let video_filename =
                converted_stream
                    .behavior_hints
                    .filename
                    .to_owned()
                    .or_else(|| {
                        video_params
                            .as_ref()
                            .and_then(|video_params| video_params.filename.clone())
                    });

            if video_hash.is_none()
                && video_size.is_none()
                && video_filename.is_none()
                && video_params.is_none()
            {
                return Effects::none().unchanged();
            }

            resources_update_with_vector_content::<E, _>(
                subtitles,
                ResourcesAction::request(
                    &AggrRequest::AllOfResource(ResourcePath {
                        extra: subtitles_path
                            .extra
                            .to_owned()
                            .extend_one(&VIDEO_HASH_EXTRA_PROP, video_hash)
                            .extend_one(
                                &VIDEO_SIZE_EXTRA_PROP,
                                video_size.map(|size| size.to_string()),
                            )
                            .extend_one(&VIDEO_FILENAME_EXTRA_PROP, video_filename),
                        ..subtitles_path.to_owned()
                    }),
                    addons,
                ),
            )
        }
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
                (true, Some(stream_name), Some(os_hash)) => {
                    let stream_name_hash = {
                        use sha2::Digest;
                        let mut sha256 = sha2::Sha256::new();
                        sha256.update(stream_name);
                        let sha256_encoded = sha256.finalize();

                        BASE64.encode(sha256_encoded)
                    };

                    let seek_log_req = SeekLogRequest {
                        os_hash,
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

const INTRODB_MIN_CONFIDENCE: f64 = 0.5;
const INTRODB_OUTRO_SEGMENT: &str = "outro";
const INTRODB_INTRO_SEGMENT: &str = "intro";

fn intro_outro_update<E: Env + 'static>(
    intro_outro: &mut Option<IntroOutro>,
    selected: Option<&Selected>,
    series_info: Option<&SeriesInfo>,
    intro_db: &mut Option<(IntroDbRequest, Loadable<IntroDbResponse, CtxError>)>,
) -> Effects {
    let intro_db_effects = intro_db_update::<E>(selected, series_info, intro_db);

    let intro_outro_effects = match intro_db {
        Some((_, Loadable::Ready(response))) => {
            let intro = response
                .segments
                .iter()
                .find(|segment| segment.segment.eq_ignore_ascii_case(INTRODB_INTRO_SEGMENT))
                .map(|segment| IntroData {
                    from: segment.from,
                    to: segment.to,
                    duration: None,
                });

            let outro = response.outro.as_ref().map(|segment| segment.from);

            if intro.is_none() && outro.is_none() && response.segments.is_empty() {
                eq_update(intro_outro, None)
            } else {
                eq_update(
                    intro_outro,
                    Some(IntroOutro {
                        intro,
                        outro,
                        segments: response.segments.to_owned(),
                    }),
                )
            }
        }
        _ => eq_update(intro_outro, None),
    };

    intro_db_effects.join(intro_outro_effects)
}

fn intro_db_update<E: Env + 'static>(
    selected: Option<&Selected>,
    series_info: Option<&SeriesInfo>,
    intro_db: &mut Option<(IntroDbRequest, Loadable<IntroDbResponse, CtxError>)>,
) -> Effects {
    let intro_db_request = selected
        .zip(series_info)
        .and_then(|(selected, series_info)| {
            selected
                .meta_request
                .as_ref()
                .map(|meta_request| (meta_request.path.id.as_str(), series_info))
        })
        .and_then(|(meta_id, series_info)| {
            let imdb_id = meta_id.split(':').next().unwrap_or(meta_id);
            if is_introdb_imdb_id(imdb_id) {
                Some(IntroDbRequest {
                    imdb_id: imdb_id.to_owned(),
                    season: series_info.season,
                    episode: series_info.episode,
                })
            } else {
                None
            }
        });

    match intro_db_request {
        Some(intro_db_request)
            if intro_db.is_none()
                || matches!(intro_db, Some((request, Loadable::Err(_))) | Some((request, _)) if request != &intro_db_request) =>
        {
            let intro_db_request_effects = get_intro_db::<E>(intro_db_request.to_owned());
            let intro_db_effects = eq_update(intro_db, Some((intro_db_request, Loadable::Loading)));

            Effects::one(intro_db_request_effects)
                .unchanged()
                .join(intro_db_effects)
        }
        Some(_) => Effects::none().unchanged(),
        None => eq_update(intro_db, None),
    }
}

fn is_introdb_imdb_id(imdb_id: &str) -> bool {
    let Some(digits) = imdb_id.strip_prefix("tt") else {
        return false;
    };

    (7..=12).contains(&digits.len()) && digits.chars().all(|digit| digit.is_ascii_digit())
}

fn get_intro_db<E: Env + 'static>(intro_db_request: IntroDbRequest) -> Effect {
    let mut endpoint = INTRODB_API_URL
        .join("segments")
        .expect("url builder failed");
    endpoint.set_query(Some(
        &serde_url_params::to_string(&intro_db_request).expect("Serialize query params failed"),
    ));
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(endpoint.as_str())
        .body(())
        .expect("request builder failed");

    EffectFuture::Concurrent(
        E::fetch::<_, serde_json::Value>(request)
            .map_err(CtxError::from)
            .and_then(|response| match parse_intro_db_response(response) {
                Ok(response) => future::ok(response),
                Err(error) => future::err(CtxError::from(error)),
            })
            .map(move |result: Result<IntroDbResponse, CtxError>| {
                Msg::Internal(Internal::IntroDbResult(intro_db_request, result))
            })
            .boxed_env(),
    )
    .into()
}

fn parse_intro_db_response(response: serde_json::Value) -> Result<IntroDbResponse, EnvError> {
    let response = response.get("result").cloned().unwrap_or(response);
    let segments_json = response
        .as_object()
        .ok_or_else(|| EnvError::Serde("IntroDB response must be a JSON object".to_owned()))?;

    let mut segments = vec![];
    let mut outro = None;

    for (segment_name, segment_data) in segments_json {
        if let Some(segment_range) = parse_intro_db_segment(segment_data) {
            if segment_name.eq_ignore_ascii_case(INTRODB_OUTRO_SEGMENT) {
                outro = Some(segment_range);
            } else {
                segments.push(IntroSegment {
                    segment: segment_name.to_owned(),
                    from: segment_range.from,
                    to: segment_range.to,
                });
            }
        }
    }

    segments.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.segment.cmp(&b.segment)));

    Ok(IntroDbResponse { segments, outro })
}

fn parse_intro_db_segment(segment_data: &serde_json::Value) -> Option<SegmentRange> {
    let segment_data = segment_data.as_object()?;
    let confidence = segment_data.get("confidence").and_then(json_value_to_f64)?;

    if confidence < INTRODB_MIN_CONFIDENCE {
        return None;
    }

    let from = segment_data.get("start_ms").and_then(json_value_to_u64)?;
    let to = segment_data.get("end_ms").and_then(json_value_to_u64)?;

    if from >= to {
        return None;
    }

    Some(SegmentRange { from, to })
}

fn json_value_to_u64(value: &serde_json::Value) -> Option<u64> {
    match value {
        serde_json::Value::Number(number) => number.as_u64(),
        serde_json::Value::String(number) => number.parse::<u64>().ok(),
        _ => None,
    }
}

fn json_value_to_f64(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(number) => number.parse::<f64>().ok(),
        _ => None,
    }
}

/// Sends watched state for meta item on every Watched state change of the library item
fn send_watched<E: Env + 'static>(auth_key: AuthKey, meta_path: &ResourcePath) -> Effect {
    let meta_id = meta_path.id.to_owned();

    let request = RatingSendRequest {
        auth_key,
        meta_item_id: meta_id.to_owned(),
        meta_item_type: meta_path.r#type.to_owned(),
        rating: Some(Rating::Watched),
    };

    EffectFuture::Concurrent(
        E::fetch::<_, RatingSendResponse>(request.into())
            .map(enclose::enclose!((meta_id) move |result| {
                Msg::Internal(Internal::WatchedSendResult(
                    meta_id, result,
                ))
            }))
            .boxed_env(),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::player::{is_introdb_imdb_id, parse_intro_db_response};

    #[test]
    fn parse_intro_db_response_with_valid_segments() {
        let response = json!({
            "intro": {
                "start_ms": 1000,
                "end_ms": 5000,
                "confidence": 0.95
            },
            "recap": {
                "start_ms": "7000",
                "end_ms": "9000",
                "confidence": "0.80"
            },
            "outro": {
                "start_ms": 1000000,
                "end_ms": 1100000,
                "confidence": 0.9
            }
        });

        let parsed = parse_intro_db_response(response).expect("IntroDB response should parse");

        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].segment, "intro");
        assert_eq!(parsed.segments[0].from, 1000);
        assert_eq!(parsed.segments[0].to, 5000);
        assert_eq!(parsed.segments[1].segment, "recap");
        assert_eq!(
            parsed.outro.as_ref().map(|segment| segment.from),
            Some(1_000_000)
        );
    }

    #[test]
    fn parse_intro_db_response_ignores_invalid_or_low_confidence_segments() {
        let response = json!({
            "intro": {
                "start_ms": 1000,
                "end_ms": 5000
            },
            "recap": {
                "start_ms": 9000,
                "end_ms": 7000,
                "confidence": 0.95
            },
            "credits": {
                "start_ms": 10000,
                "end_ms": 12000,
                "confidence": 0.1
            },
            "outro": {
                "start_ms": 1,
                "end_ms": 2,
                "confidence": 0.9
            }
        });

        let parsed = parse_intro_db_response(response).expect("IntroDB response should parse");

        assert!(parsed.segments.is_empty());
        assert_eq!(parsed.outro.as_ref().map(|segment| segment.from), Some(1));
    }

    #[test]
    fn is_introdb_imdb_id_accepts_realistic_imdb_ids() {
        assert!(is_introdb_imdb_id("tt1254207"));
        assert!(is_introdb_imdb_id("tt15264452"));
    }

    #[test]
    fn is_introdb_imdb_id_rejects_short_or_non_numeric_ids() {
        assert!(!is_introdb_imdb_id("tt1"));
        assert!(!is_introdb_imdb_id("ttabcdefg"));
        assert!(!is_introdb_imdb_id("kitsu:123"));
    }
}
