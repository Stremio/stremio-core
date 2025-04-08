use std::{borrow::Cow, marker::PhantomData};

use serde::{Deserialize, Serialize};

use stremio_watched_bitfield::WatchedBitField;

use crate::{
    constants::{LIBRARY_COLLECTION_NAME, META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        common::{
            eq_update, resources_update, resources_update_with_vector_content, Loadable,
            ResourceLoadable, ResourcesAction,
        },
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad, ActionMetaDetails, Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        addon::{AggrRequest, ResourcePath, ResourceRequest},
        api::{DatastoreCommand, DatastoreRequest},
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
        resource::{MetaItem, Stream},
        streams::StreamsBucket,
    },
};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub meta_path: ResourcePath,
    pub stream_path: Option<ResourcePath>,
    #[serde(default)]
    /// if `stream_path` is `None` then we try to guess the video and make a request
    /// to the addons to load the streams for that video id
    pub guess_stream: bool,
}

#[derive(Default, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetaDetails {
    pub selected: Option<Selected>,
    pub meta_items: Vec<ResourceLoadable<MetaItem>>,
    pub meta_streams: Vec<ResourceLoadable<Vec<Stream>>>,
    pub streams: Vec<ResourceLoadable<Vec<Stream>>>,
    /// A Stream from addon responses, based on your already watched streams on the device
    /// (i.e. [`StreamsBucket`]), which could be played if binge watching or continuing to watch.
    ///
    /// Finds a proper stream for the binge watching group and matching stream source.
    pub last_used_stream: Option<ResourceLoadable<Option<Stream>>>,
    pub library_item: Option<LibraryItem>,
    #[serde(skip_serializing)]
    pub watched: Option<WatchedBitField>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for MetaDetails {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_items_effects =
                    meta_items_update::<E>(&mut self.meta_items, &self.selected, &ctx.profile);
                let selected_override_effects =
                    selected_guess_stream_update(&mut self.selected, &self.meta_items);
                let meta_streams_effects =
                    meta_streams_update(&mut self.meta_streams, &self.selected, &self.meta_items);
                let streams_effects =
                    streams_update::<E>(&mut self.streams, &self.selected, &ctx.profile);
                let last_used_stream_effects = last_used_stream_update(
                    &mut self.last_used_stream,
                    &self.selected,
                    &self.meta_items,
                    &self.meta_streams,
                    &self.streams,
                    &ctx.streams,
                );
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_items,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_items, &self.library_item);
                let library_item_sync_effects = library_item_sync(&self.library_item, &ctx.profile);
                library_item_sync_effects
                    .join(selected_effects)
                    .join(selected_override_effects)
                    .join(meta_items_effects)
                    .join(meta_streams_effects)
                    .join(streams_effects)
                    .join(last_used_stream_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_items_effects = eq_update(&mut self.meta_items, vec![]);
                let meta_streams_effects = eq_update(&mut self.meta_streams, vec![]);
                let streams_effects = eq_update(&mut self.streams, vec![]);
                let library_item_effects = eq_update(&mut self.library_item, None);
                let last_used_stream_effects = eq_update(&mut self.last_used_stream, None);
                let watched_effects = eq_update(&mut self.watched, None);
                selected_effects
                    .join(meta_items_effects)
                    .join(meta_streams_effects)
                    .join(streams_effects)
                    .join(last_used_stream_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            Msg::Action(Action::MetaDetails(ActionMetaDetails::MarkAsWatched(is_watched))) => {
                match &self.library_item {
                    Some(library_item) => {
                        let mut library_item = library_item.to_owned();
                        library_item.mark_as_watched::<E>(*is_watched);
                        Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(library_item)))
                            .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::MetaDetails(ActionMetaDetails::MarkVideoAsWatched(
                video,
                is_watched,
            ))) => match (&self.library_item, &self.watched) {
                (Some(library_item), Some(watched)) => {
                    let mut library_item = library_item.to_owned();
                    library_item.mark_video_as_watched::<E>(watched, video, *is_watched);

                    Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(library_item)))
                        .unchanged()
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Action(Action::MetaDetails(ActionMetaDetails::MarkSeasonAsWatched(
                season,
                is_watched,
            ))) => match (&self.library_item, &self.watched) {
                (Some(library_item), Some(watched)) => {
                    // Find videos of given season from the first ready meta item loadable
                    let videos = self
                        .meta_items
                        .iter()
                        .find(|meta_item| matches!(&meta_item.content, Some(Loadable::Ready(_))))
                        .and_then(|meta_item| meta_item.content.as_ref())
                        .and_then(|meta_item| meta_item.ready())
                        .map(|meta_item| meta_item.videos_by_season(*season));

                    match videos {
                        Some(videos) => {
                            let mut library_item = library_item.to_owned();
                            library_item.mark_videos_as_watched::<E>(watched, videos, *is_watched);

                            Effects::msg(Msg::Internal(Internal::UpdateLibraryItem(library_item)))
                                .unchanged()
                        }
                        None => Effects::none().unchanged(),
                    }
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == META_RESOURCE_NAME =>
            {
                let meta_items_effects = resources_update::<E, _>(
                    &mut self.meta_items,
                    ResourcesAction::ResourceRequestResult { request, result },
                );
                let selected_override_effects =
                    selected_guess_stream_update(&mut self.selected, &self.meta_items);
                let streams_effects = if selected_override_effects.has_changed {
                    streams_update::<E>(&mut self.streams, &self.selected, &ctx.profile)
                } else {
                    Effects::default()
                };
                let meta_streams_effects =
                    meta_streams_update(&mut self.meta_streams, &self.selected, &self.meta_items);
                let last_used_stream_effects = last_used_stream_update(
                    &mut self.last_used_stream,
                    &self.selected,
                    &self.meta_items,
                    &self.meta_streams,
                    &self.streams,
                    &ctx.streams,
                );
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_items,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_items, &self.library_item);
                selected_override_effects
                    .join(meta_items_effects)
                    .join(meta_streams_effects)
                    .join(streams_effects)
                    .join(last_used_stream_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == STREAM_RESOURCE_NAME =>
            {
                let streams_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.streams,
                    ResourcesAction::ResourceRequestResult { request, result },
                );
                let last_used_stream_effects = last_used_stream_update(
                    &mut self.last_used_stream,
                    &self.selected,
                    &self.meta_items,
                    &self.meta_streams,
                    &self.streams,
                    &ctx.streams,
                );
                streams_effects.join(last_used_stream_effects)
            }
            Msg::Internal(Internal::LibraryChanged(_)) => {
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_items,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_items, &self.library_item);
                library_item_effects.join(watched_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                let meta_items_effects =
                    meta_items_update::<E>(&mut self.meta_items, &self.selected, &ctx.profile);
                let meta_streams_effects =
                    meta_streams_update(&mut self.meta_streams, &self.selected, &self.meta_items);
                let streams_effects =
                    streams_update::<E>(&mut self.streams, &self.selected, &ctx.profile);
                let last_used_stream_effects = last_used_stream_update(
                    &mut self.last_used_stream,
                    &self.selected,
                    &self.meta_items,
                    &self.meta_streams,
                    &self.streams,
                    &ctx.streams,
                );
                let library_item_effects = library_item_update::<E>(
                    &mut self.library_item,
                    &self.selected,
                    &self.meta_items,
                    &ctx.library,
                );
                let watched_effects =
                    watched_update(&mut self.watched, &self.meta_items, &self.library_item);
                meta_items_effects
                    .join(meta_streams_effects)
                    .join(streams_effects)
                    .join(last_used_stream_effects)
                    .join(library_item_effects)
                    .join(watched_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn library_item_sync(library_item: &Option<LibraryItem>, profile: &Profile) -> Effects {
    match (library_item, profile.auth_key()) {
        (Some(library_item), Some(auth_key)) => {
            Effects::msg(Msg::Internal(Internal::LibrarySyncPlanResult(
                DatastoreRequest {
                    auth_key: auth_key.to_owned(),
                    collection: LIBRARY_COLLECTION_NAME.to_owned(),
                    command: DatastoreCommand::Meta {},
                },
                Ok((vec![library_item.id.to_owned()], vec![])),
            )))
            .unchanged()
        }
        _ => Effects::none().unchanged(),
    }
}

/// If `Selected::guess_stream` is `true` then we will override the selected stream
/// no matter if it's set (`Some`) or not (`None`).
///
/// How we override the stream:
///
/// 1. We find the first `MetaItem` that's successfully loaded from the addons.
/// 2. Selecting the video id for the stream request:
///    2.1 If there's a `MetaItem.preview.behavior_hints.default_video_id`
///    we use it for the request
///    2.2 If there's no `default_video_id` and no `MetaItem.videos` returned by the addon,
///    we use the `MetaItem.preview.id`
///
/// If we haven't found a suitable `video_id`, then we do not override the `Selected::stream_path`.
fn selected_guess_stream_update(
    selected: &mut Option<Selected>,
    meta_items: &[ResourceLoadable<MetaItem>],
) -> Effects {
    let meta_path = match &selected {
        Some(Selected {
            meta_path,
            // guess the stream only if `stream_path` is `None`!
            stream_path: None,
            guess_stream: true,
        }) => meta_path,
        _ => return Effects::default(),
    };

    // Wait for all requests to finish before retrieving the meta_item
    let meta_item = if meta_items.iter().all(|meta_item| {
        matches!(meta_item.content, Some(Loadable::Ready(..)))
            || matches!(meta_item.content, Some(Loadable::Err(..)))
    }) {
        match meta_items
            .iter()
            .find_map(|meta_item| match &meta_item.content {
                Some(Loadable::Ready(meta_item)) => Some(meta_item),
                _ => None,
            }) {
            Some(meta_item) => meta_item,
            _ => return Effects::default(),
        }
    } else {
        return Effects::default();
    };

    let video_id = match (
        meta_item.videos.len(),
        &meta_item.preview.behavior_hints.default_video_id,
    ) {
        (_, Some(default_video_id)) => default_video_id.to_owned(),
        (0, None) => meta_item.preview.id.to_owned(),
        _ => return Effects::default(),
    };

    eq_update(
        selected,
        Some(Selected {
            meta_path: meta_path.to_owned(),
            stream_path: Some(ResourcePath {
                resource: STREAM_RESOURCE_NAME.to_owned(),
                r#type: meta_path.r#type.to_owned(),
                id: video_id,
                extra: vec![],
            }),
            // we must set the `guess_stream` to `false` after we've overridden it
            // to make it consistent
            guess_stream: false,
        }),
    )
}

fn meta_items_update<E: Env + 'static>(
    meta_items: &mut Vec<ResourceLoadable<MetaItem>>,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    match selected {
        Some(Selected { meta_path, .. }) => resources_update::<E, _>(
            meta_items,
            ResourcesAction::ResourcesRequested {
                request: &AggrRequest::AllOfResource(meta_path.to_owned()),
                addons: &profile.addons,
                // use existing loaded MetaItems instead of making a request every time.
                force: false,
            },
        ),
        _ => eq_update(meta_items, vec![]),
    }
}

fn meta_streams_update(
    meta_streams: &mut Vec<ResourceLoadable<Vec<Stream>>>,
    selected: &Option<Selected>,
    meta_items: &[ResourceLoadable<MetaItem>],
) -> Effects {
    match selected {
        Some(Selected {
            stream_path: Some(stream_path),
            ..
        }) => {
            let next_meta_streams = meta_items
                .iter()
                .find_map(|meta_item| match meta_item {
                    ResourceLoadable {
                        request,
                        content: Some(Loadable::Ready(meta_item)),
                    } => Some((request, meta_item)),
                    _ => None,
                })
                .and_then(|(request, meta_item)| {
                    meta_item
                        .videos
                        .iter()
                        .find(|video| video.id == stream_path.id)
                        .and_then(|video| {
                            if !video.streams.is_empty() {
                                Some(Cow::Borrowed(&video.streams))
                            } else {
                                Stream::youtube(&video.id)
                                    .map(|stream| vec![stream])
                                    .map(Cow::Owned)
                            }
                        })
                        .map(|streams| (request, streams))
                })
                .map(|(request, streams)| ResourceLoadable {
                    request: ResourceRequest {
                        base: request.base.to_owned(),
                        path: ResourcePath {
                            resource: STREAM_RESOURCE_NAME.to_owned(),
                            r#type: request.path.r#type.to_owned(),
                            id: stream_path.id.to_owned(),
                            extra: request.path.extra.to_owned(),
                        },
                    },
                    content: Some(Loadable::Ready(streams.into_owned())),
                })
                .into_iter()
                .collect();
            eq_update(meta_streams, next_meta_streams)
        }
        _ => Effects::none().unchanged(),
    }
}

fn streams_update<E: Env + 'static>(
    streams: &mut Vec<ResourceLoadable<Vec<Stream>>>,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    match selected {
        Some(Selected {
            stream_path: Some(stream_path),
            ..
        }) => resources_update_with_vector_content::<E, _>(
            streams,
            ResourcesAction::ResourcesRequested {
                request: &AggrRequest::AllOfResource(stream_path.to_owned()),
                addons: &profile.addons,
                // use existing loaded MetaItems instead of making a request every time.
                force: false,
            },
        ),
        _ => eq_update(streams, vec![]),
    }
}

/// Find a stream from addon responses, which should be played if binge watching or continuing to watch.
/// We've already loaded the next Video id and we need to find a proper stream for the binge watching.
///
/// First find the latest `StreamItem` stored based on last **30** videos from current video
/// (ie. we're in E4, so we're going to check E4, E3, E2, E1 in this order until we hit a stored `StreamItem`).
/// Then with the stream item we try to find a stream from addon responses (including the streams inside the meta itself `meta_streams`) -
/// we find the responses from the addon based on `StreamItem.stream_transport_url`,
/// then first we try to find the stream based on equality (as otherwise stored stream might be expired/no longer valid),
/// if not found we try to find a stream based on it's `StreamBehaviorHints.bingeGroup`.
/// One note, why we cannot return `StreamItem.stream` directly if it's for the same episode,
/// is that user might have played a stream from an addon which he no longer has due to some constrains (ie p2p addon),
/// that's why we have to try to find it first and verify that's it's still available.
fn last_used_stream_update(
    last_used_stream: &mut Option<ResourceLoadable<Option<Stream>>>,
    selected: &Option<Selected>,
    meta_items: &[ResourceLoadable<MetaItem>],
    meta_streams: &[ResourceLoadable<Vec<Stream>>],
    streams: &[ResourceLoadable<Vec<Stream>>],
    stream_bucket: &StreamsBucket,
) -> Effects {
    let all_streams = [meta_streams, streams].concat();
    let next_last_used_stream = match selected {
        Some(Selected {
            stream_path: Some(stream_path),
            ..
        }) => meta_items
            .iter()
            .filter(|_| !all_streams.is_empty())
            .find_map(|meta_item_res| match &meta_item_res.content {
                Some(Loadable::Ready(meta_item)) => stream_bucket
                    .last_stream_item(&stream_path.id, meta_item)
                    .and_then(|stream_item| {
                        all_streams
                            .iter()
                            .find(|resource| {
                                resource.request.base == stream_item.stream_transport_url
                            })
                            .and_then(|resource| match &resource.content {
                                Some(Loadable::Ready(streams)) => Some(ResourceLoadable {
                                    request: resource.request.clone(),
                                    content: Some(Loadable::Ready(
                                        streams
                                            .iter()
                                            .find(|stream| {
                                                stream.is_source_match(&stream_item.stream)
                                            })
                                            .or_else(|| {
                                                streams.iter().find(|stream| {
                                                    stream.is_binge_match(&stream_item.stream)
                                                })
                                            })
                                            .cloned(),
                                    )),
                                }),
                                Some(Loadable::Loading) => Some(ResourceLoadable {
                                    request: resource.request.clone(),
                                    content: Some(Loadable::Loading),
                                }),
                                Some(Loadable::Err(error)) => Some(ResourceLoadable {
                                    request: resource.request.clone(),
                                    content: Some(Loadable::Err(error.clone())),
                                }),
                                _ => None,
                            })
                    })
                    .or_else(|| {
                        Some(ResourceLoadable {
                            request: meta_item_res.request.clone(),
                            content: Some(Loadable::Ready(None)),
                        })
                    }),
                _ => None,
            }),
        _ => None,
    };
    eq_update(last_used_stream, next_last_used_stream)
}

fn library_item_update<E: Env + 'static>(
    library_item: &mut Option<LibraryItem>,
    selected: &Option<Selected>,
    meta_items: &[ResourceLoadable<MetaItem>],
    library: &LibraryBucket,
) -> Effects {
    let meta_item = meta_items
        .iter()
        .find_map(|meta_item| match &meta_item.content {
            Some(Loadable::Ready(meta_item)) => Some(meta_item),
            _ => None,
        });
    let next_library_item = match selected {
        Some(selected) => library
            .items
            .get(&selected.meta_path.id)
            .map(|library_item| {
                meta_item.map_or_else(
                    || library_item.to_owned(),
                    |meta_item| LibraryItem::from((&meta_item.preview, library_item)),
                )
            })
            .or_else(|| {
                meta_item.map(|meta_item| LibraryItem::from((&meta_item.preview, PhantomData::<E>)))
            }),
        _ => None,
    };
    eq_update(library_item, next_library_item)
}

fn watched_update(
    watched: &mut Option<WatchedBitField>,
    meta_items: &[ResourceLoadable<MetaItem>],
    library_item: &Option<LibraryItem>,
) -> Effects {
    let next_watched = meta_items
        .iter()
        .find_map(|meta_item| match &meta_item.content {
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
