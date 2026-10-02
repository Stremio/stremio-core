use futures::FutureExt;
use stremio_watched_bitfield::WatchedBitField;

use crate::{
    constants::{LIBRARY_RESOURCE_NAME, PLAYER_RESOURCE_NAME},
    runtime::{
        msg::{Internal, Msg},
        EffectFuture, Effects, Env, EnvFutureExt,
    },
    types::{
        addon::{AggrRequest, Descriptor, ExtraValue, ResourcePath},
        library::LibraryItem,
        resource::Video,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerEventAction {
    Start,
    Pause,
    Stop,
}

impl PlayerEventAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayerEventAction::Start => "start",
            PlayerEventAction::Pause => "pause",
            PlayerEventAction::Stop => "stop",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LibraryEventAction {
    LibraryAdd,
    LibraryRemove,
    Watched,
    Unwatched,
}

impl LibraryEventAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            LibraryEventAction::LibraryAdd => "libraryAdd",
            LibraryEventAction::LibraryRemove => "libraryRemove",
            LibraryEventAction::Watched => "watched",
            LibraryEventAction::Unwatched => "unwatched",
        }
    }

    pub fn from_watched(is_watched: bool) -> Self {
        if is_watched {
            LibraryEventAction::Watched
        } else {
            LibraryEventAction::Unwatched
        }
    }
}

fn extra_value(name: &str, value: &str) -> ExtraValue {
    ExtraValue {
        name: name.to_owned(),
        value: value.to_owned(),
    }
}

pub fn player_event_path(
    r#type: &str,
    video_id: &str,
    action: PlayerEventAction,
    time: u64,
    duration: u64,
) -> ResourcePath {
    ResourcePath::with_extra(
        PLAYER_RESOURCE_NAME,
        r#type,
        video_id,
        &[
            extra_value("action", action.as_str()),
            extra_value("currentTime", &time.to_string()),
            extra_value("duration", &duration.to_string()),
        ],
    )
}

pub fn library_event_path(
    r#type: &str,
    id: &str,
    action: LibraryEventAction,
    video_id: Option<&str>,
) -> ResourcePath {
    let mut extra = vec![extra_value("action", action.as_str())];
    if let Some(video_id) = video_id {
        extra.push(extra_value("videoId", video_id));
    }
    ResourcePath::with_extra(LIBRARY_RESOURCE_NAME, r#type, id, &extra)
}

pub fn item_watched_event_path(library_item: &LibraryItem, is_watched: bool) -> ResourcePath {
    library_event_path(
        &library_item.r#type,
        &library_item.id,
        LibraryEventAction::from_watched(is_watched),
        None,
    )
}

pub fn videos_watched_event_paths(
    library_item: &LibraryItem,
    watched: &WatchedBitField,
    videos: &[&Video],
    is_watched: bool,
) -> Vec<ResourcePath> {
    videos
        .iter()
        .filter(|video| watched.get_video(&video.id) != is_watched)
        .map(|video| {
            library_event_path(
                &library_item.r#type,
                &library_item.id,
                LibraryEventAction::from_watched(is_watched),
                Some(&video.id),
            )
        })
        .collect()
}

pub fn addon_events_effects<E: Env + 'static>(
    addons: &[Descriptor],
    paths: Vec<ResourcePath>,
) -> Effects {
    let futures = paths
        .into_iter()
        .flat_map(|path| AggrRequest::AllOfResource(path).plan(addons))
        .map(|(_, request)| {
            EffectFuture::Concurrent(
                E::addon_transport(&request.base)
                    .resource(&request.path)
                    .map(move |result| {
                        Msg::Internal(Internal::AddonEventResult(request, Box::new(result)))
                    })
                    .boxed_env(),
            )
        })
        .collect();
    Effects::futures(futures).unchanged()
}
