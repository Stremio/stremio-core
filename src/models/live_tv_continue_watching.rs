use std::cmp::Reverse;

use itertools::Itertools;
use serde::Serialize;

use crate::{
    constants::{CATALOG_PREVIEW_SIZE, META_RESOURCE_NAME},
    models::{
        common::{eq_update, resource_update, ResourceAction, ResourceLoadable},
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
}

impl<E: Env + 'static> UpdateWithCtx<E> for LiveTvContinueWatching {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LiveTvContinueWatching)) => {
                self.active = true;
                catalog_and_items_update::<E>(
                    &mut self.catalog,
                    &mut self.items,
                    &ctx.library,
                    &ctx.profile,
                )
            }
            Msg::Action(Action::Unload) => {
                self.active = false;
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
                    &ctx.library,
                    &ctx.profile,
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
                    let items_effects =
                        items_update(&mut self.items, &self.catalog, &ctx.library);
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
    library: &LibraryBucket,
    profile: &Profile,
) -> Effects {
    let requests = channel_requests(library, profile);

    let mut effects = Effects::none().unchanged();
    let mut previous = std::mem::take(catalog);
    let mut next_catalog = Vec::with_capacity(requests.len());
    for request in &requests {
        match previous
            .iter()
            .position(|resource| resource.request == *request)
        {
            Some(index) => next_catalog.push(previous.swap_remove(index)),
            None => {
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

    let items_effects = items_update(items, catalog, library);
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
                .unwrap_or_default();

            Item {
                channel,
                request: resource.request.to_owned(),
                shows,
            }
        })
        .collect::<Vec<_>>();

    eq_update(items, next_items)
}

fn channel_requests(library: &LibraryBucket, profile: &Profile) -> Vec<ResourceRequest> {
    library
        .items
        .values()
        .filter(|item| {
            item.r#type != "other"
                && (!item.removed || item.temp)
                && item.state.last_watched.is_some()
        })
        .filter(|item| profile.is_epg_channel_id(&item.id))
        .sorted_by_key(|item| Reverse(item.state.last_watched.unwrap_or(item.mtime)))
        .take(CATALOG_PREVIEW_SIZE)
        .filter_map(|item| meta_request(profile, item))
        .collect()
}

fn meta_request(profile: &Profile, item: &LibraryItem) -> Option<ResourceRequest> {
    profile
        .addons
        .iter()
        .filter(|addon| addon.manifest.behavior_hints.epg_provider)
        .find(|addon| {
            addon
                .manifest
                .id_prefixes
                .iter()
                .flatten()
                .any(|prefix| item.id.starts_with(prefix))
        })
        .map(|addon| ResourceRequest {
            base: addon.transport_url.to_owned(),
            path: ResourcePath::without_extra(META_RESOURCE_NAME, &item.r#type, &item.id),
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
