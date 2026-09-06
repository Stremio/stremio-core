use std::cmp::Reverse;

use chrono::{DateTime, Duration, Utc};

use itertools::Itertools;
use serde::Serialize;

use crate::{
    constants::{CATALOG_PREVIEW_SIZE, META_RESOURCE_NAME},
    models::{
        common::{eq_update, resource_update, Loadable, ResourceAction, ResourceLoadable},
        ctx::Ctx,
    },
    runtime::{
        msg::{Action, ActionLoad, Internal, Msg},
        Effects, Env, UpdateWithCtx,
    },
    types::{
        addon::{ResourcePath, ResourceRequest},
        library::{LibraryBucket, LibraryItem},
        profile::Profile,
        resource::{MetaItem, MetaItemPreview, Video},
        streams::StreamsBucket,
    },
};

#[derive(Clone, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub channel: MetaItemPreview,
    pub request: ResourceRequest,
    pub shows: Vec<Video>,
}

#[derive(Default, Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveTvContinueWatching {
    pub items: Vec<Item>,
    /// One `meta` loadable per watched channel, in most-recently-watched order.
    #[serde(skip)]
    pub catalog: Vec<ResourceLoadable<MetaItem>>,
    /// Whether the model has been loaded; gates reacting to library/profile
    /// changes so nothing is fetched before the home board is shown.
    #[serde(skip)]
    pub active: bool,
    #[serde(skip)]
    pub requested_at: Vec<(ResourceRequest, DateTime<Utc>)>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for LiveTvContinueWatching {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LiveTvContinueWatching)) => {
                self.active = true;
                catalog_and_items_update::<E>(
                    &mut self.catalog,
                    &mut self.items,
                    &mut self.requested_at,
                    ctx,
                )
            }
            Msg::Action(Action::Unload) => {
                self.active = false;
                self.requested_at.clear();
                let catalog_effects = eq_update(&mut self.catalog, vec![]);
                let items_effects = eq_update(&mut self.items, vec![]);

                catalog_effects.join(items_effects)
            }
            Msg::Internal(Internal::LibraryChanged(true))
            | Msg::Internal(Internal::ProfileChanged)
                if self.active =>
            {
                catalog_and_items_update::<E>(
                    &mut self.catalog,
                    &mut self.items,
                    &mut self.requested_at,
                    ctx,
                )
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => self
                .catalog
                .iter_mut()
                .find(|resource| resource.request == *request)
                .map(|resource| {
                    resource_update::<E, MetaItem>(
                        resource,
                        ResourceAction::ResourceRequestResult { request, result },
                    )
                })
                .map(|catalog_effects| {
                    let items_effects = items_update(&mut self.items, &self.catalog, &ctx.library);
                    catalog_effects.join(items_effects)
                })
                .unwrap_or_else(|| Effects::none().unchanged()),
            _ => Effects::none().unchanged(),
        }
    }
}

fn catalog_and_items_update<E: Env + 'static>(
    catalog: &mut Vec<ResourceLoadable<MetaItem>>,
    items: &mut Vec<Item>,
    requested_at: &mut Vec<(ResourceRequest, DateTime<Utc>)>,
    ctx: &Ctx,
) -> Effects {
    let requests = channel_requests(&ctx.library, &ctx.profile, &ctx.streams);
    requested_at.retain(|(request, _)| requests.contains(request));

    let mut effects = Effects::none().unchanged();
    let mut previous = std::mem::take(catalog);
    let mut next_catalog = Vec::with_capacity(requests.len());
    for request in &requests {
        match previous
            .iter()
            .position(|resource| resource.request == *request)
        {
            Some(index) => {
                let mut resource = previous.swap_remove(index);
                let elapsed = requested_at
                    .iter()
                    .find(|(key, _)| key == request)
                    .map(|(_, time)| E::now() - *time);
                let stale = match resource.content.as_ref() {
                    Some(Loadable::Loading) => false,
                    Some(Loadable::Ready(meta)) => {
                        let coverage_end = meta
                            .videos
                            .iter()
                            .filter_map(|video| video.epg_info.as_ref())
                            .filter(|info| info.start_time < info.end_time)
                            .map(|info| info.end_time)
                            .max();
                        elapsed.map_or(true, |elapsed| {
                            elapsed >= Duration::minutes(15)
                                || (elapsed >= Duration::minutes(1)
                                    && coverage_end.is_some_and(|end| end <= E::now()))
                        })
                    }
                    _ => elapsed.map_or(true, |elapsed| elapsed >= Duration::minutes(1)),
                };
                if stale {
                    resource.content = None;
                    requested_at.retain(|(key, _)| key != request);
                    requested_at.push((request.clone(), E::now()));
                    effects = effects.join(resource_update::<E, MetaItem>(
                        &mut resource,
                        ResourceAction::ResourceRequested { request },
                    ));
                }
                next_catalog.push(resource);
            }
            None => {
                requested_at.retain(|(key, _)| key != request);
                requested_at.push((request.clone(), E::now()));
                let mut resource = ResourceLoadable {
                    request: request.to_owned(),
                    content: None,
                };
                effects = effects.join(resource_update::<E, MetaItem>(
                    &mut resource,
                    ResourceAction::ResourceRequested { request },
                ));
                next_catalog.push(resource);
            }
        }
    }
    *catalog = next_catalog;

    let items_effects = items_update(items, catalog, &ctx.library);
    effects.join(items_effects)
}

fn items_update(
    items: &mut Vec<Item>,
    catalog: &[ResourceLoadable<MetaItem>],
    library: &LibraryBucket,
) -> Effects {
    let next_items = catalog
        .iter()
        .map(|resource| {
            let meta = resource
                .content
                .as_ref()
                .and_then(|content| content.ready());
            let channel = meta
                .map(|meta_item| meta_item.preview.to_owned())
                .unwrap_or_else(|| fallback_preview(library, &resource.request));
            let shows = meta
                .map(|meta_item| {
                    meta_item
                        .videos
                        .iter()
                        .filter(|video| video.epg_info.is_some())
                        .cloned()
                        .collect()
                })
                .unwrap_or_else(|| {
                    items
                        .iter()
                        .find(|item| item.request == resource.request)
                        .map(|item| item.shows.clone())
                        .unwrap_or_default()
                });

            Item {
                channel,
                request: resource.request.to_owned(),
                shows,
            }
        })
        .collect::<Vec<_>>();

    eq_update(items, next_items)
}

fn channel_requests(
    library: &LibraryBucket,
    profile: &Profile,
    streams: &StreamsBucket,
) -> Vec<ResourceRequest> {
    library
        .items
        .values()
        .filter(|item| {
            item.r#type != "other"
                && (!item.removed || item.temp)
                && item.state.last_watched.is_some()
        })
        .filter(|item| item.is_live())
        .sorted_by_key(|item| Reverse(item.state.last_watched.unwrap_or(item.mtime)))
        .take(CATALOG_PREVIEW_SIZE)
        .filter_map(|item| meta_request(profile, streams, item))
        .collect()
}

fn meta_request(
    profile: &Profile,
    streams: &StreamsBucket,
    item: &LibraryItem,
) -> Option<ResourceRequest> {
    let path = ResourcePath::without_extra(META_RESOURCE_NAME, &item.r#type, &item.id);
    let source = streams
        .items
        .values()
        .filter(|stream| stream.meta_id == item.id && stream.r#type == item.r#type)
        .max_by_key(|stream| stream.mtime)
        .map(|stream| &stream.meta_transport_url);
    let addon = profile
        .addons
        .iter()
        .filter(|addon| addon.manifest.is_resource_supported(&path))
        .min_by_key(|addon| {
            if source == Some(&addon.transport_url) {
                0
            } else if addon.manifest.behavior_hints.epg_provider {
                1
            } else {
                2
            }
        })?;
    Some(ResourceRequest {
        base: addon.transport_url.clone(),
        path,
    })
}

fn fallback_preview(library: &LibraryBucket, request: &ResourceRequest) -> MetaItemPreview {
    let item = library.items.get(&request.path.id);
    MetaItemPreview {
        id: item.map_or_else(|| request.path.id.to_owned(), |item| item.id.to_owned()),
        r#type: item.map_or_else(
            || request.path.r#type.to_owned(),
            |item| item.r#type.to_owned(),
        ),
        name: item.map(|item| item.name.to_owned()).unwrap_or_default(),
        poster: item.and_then(|item| item.poster.to_owned()),
        poster_shape: item
            .map(|item| item.poster_shape.to_owned())
            .unwrap_or_default(),
        background: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        links: vec![],
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    }
}
