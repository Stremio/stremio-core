use std::marker::PhantomData;

use crate::constants::{
    CREDITS_THRESHOLD_COEF, VIDEO_HASH_EXTRA_PROP, VIDEO_SIZE_EXTRA_PROP, WATCHED_THRESHOLD_COEF,
};
use crate::models::common::{
    eq_update, resource_update, resources_update_with_vector_content, Loadable, ResourceAction,
    ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, ActionPlayer, Event, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ExtraExt, ResourcePath, ResourceRequest};
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::profile::Settings as ProfileSettings;
use crate::types::resource::{MetaItem, SeriesInfo, Stream, Subtitles, Video};

use stremio_watched_bitfield::WatchedBitField;

use chrono::{DateTime, Duration, Utc};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

use super::common::resource_update_with_vector_content;

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
    pub hash: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub stream: Stream,
    pub stream_request: Option<ResourceRequest>,
    pub meta_request: Option<ResourceRequest>,
    pub subtitles_path: Option<ResourcePath>,
    pub video_params: Option<VideoParams>,
}

#[derive(Default, Derivative, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub selected: Option<Selected>,
    pub meta_item: Option<ResourceLoadable<MetaItem>>,
    pub subtitles: Vec<ResourceLoadable<Vec<Subtitles>>>,
    pub next_video: Option<Video>,
    pub next_streams: Option<ResourceLoadable<Vec<Stream>>>,
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
                let subtitles_effects = match &selected.subtitles_path {
                    Some(subtitles_path) => resources_update_with_vector_content::<E, _>(
                        &mut self.subtitles,
                        ResourcesAction::ResourcesRequested {
                            request: &AggrRequest::AllOfResource(ResourcePath {
                                extra: subtitles_path
                                    .extra
                                    .to_owned()
                                    .extend_one(
                                        &VIDEO_HASH_EXTRA_PROP,
                                        selected
                                            .video_params
                                            .as_ref()
                                            .and_then(|params| params.hash.to_owned()),
                                    )
                                    .extend_one(
                                        &VIDEO_SIZE_EXTRA_PROP,
                                        selected
                                            .video_params
                                            .as_ref()
                                            .and_then(|params| params.size)
                                            .map(|size| size.to_string()),
                                    ),
                                ..subtitles_path.to_owned()
                            }),
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
                let next_streams_effects = next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
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
                    .join(selected_effects)
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
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
                let meta_item_effects = eq_update(&mut self.meta_item, None);
                let subtitles_effects = eq_update(&mut self.subtitles, vec![]);
                let next_video_effects = eq_update(&mut self.next_video, None);
                let next_streams_effects = eq_update(&mut self.next_streams, None);
                let series_info_effects = eq_update(&mut self.series_info, None);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let watched_effects = eq_update(&mut self.watched, None);
                self.analytics_context = None;
                self.load_time = None;
                self.loaded = false;
                self.ended = false;
                self.paused = None;
                switch_to_next_video_effects
                    .join(push_to_library_effects)
                    .join(selected_effects)
                    .join(meta_item_effects)
                    .join(subtitles_effects)
                    .join(next_video_effects)
                    .join(next_streams_effects)
                    .join(series_info_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
                    .join(ended_effects)
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
                    &self.selected,
                    &self.meta_item,
                    &ctx.profile.settings,
                );
                let next_streams_effects = next_streams_effects.join(next_streams_update::<E>(
                    &mut self.next_streams,
                    &self.next_video,
                    &self.selected,
                ));
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
            video_params: None,
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
