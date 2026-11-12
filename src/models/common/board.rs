use std::collections::HashSet;

use futures::FutureExt;

use crate::{
    constants::{CALENDAR_IDS_EXTRA_PROP, LAST_VIDEOS_IDS_EXTRA_PROP},
    models::common::{Loadable, ResourceLoadable},
    runtime::{
        msg::{Internal, Msg},
        Effect, EffectFuture, Env, EnvFutureExt,
    },
    types::{
        addon::{AggrRequest, Descriptor, ManifestCatalog, ResourceRequest},
        profile::Profile,
        resource::MetaItemPreview,
    },
};

pub const SPOTLIGHT_ITEMS_COUNT: usize = 6;
pub const SPOTLIGHT_CATALOGS_COUNT: usize = 3;
const SPOTLIGHT_DEFAULT_TYPES: [&str; 2] = ["movie", "series"];

pub type CatalogPage<T> = ResourceLoadable<Vec<T>>;

pub fn board_plan(profile: &Profile) -> Vec<(&Descriptor, &ManifestCatalog, ResourceRequest)> {
    let extra = vec![];
    AggrRequest::AllCatalogs {
        extra: &extra,
        r#type: &None,
    }
    .plan(&profile.addons)
    .into_iter()
    .filter_map(|(addon, request)| {
        addon
            .manifest
            .catalogs
            .iter()
            .find(|catalog| catalog.id == request.path.id && catalog.r#type == request.path.r#type)
            .map(|catalog| (addon, catalog, request))
    })
    .filter(|(_, catalog, _)| {
        !catalog.extra.iter().any(|extra_prop| {
            extra_prop.name == LAST_VIDEOS_IDS_EXTRA_PROP.name
                || extra_prop.name == CALENDAR_IDS_EXTRA_PROP.name
        })
    })
    .collect()
}

pub fn default_spotlight_requests(
    plan: &[(&Descriptor, &ManifestCatalog, ResourceRequest)],
) -> Vec<ResourceRequest> {
    let requests = SPOTLIGHT_DEFAULT_TYPES
        .iter()
        .filter_map(|r#type| {
            plan.iter()
                .find(|(_, catalog, _)| catalog.r#type == *r#type)
        })
        .map(|(_, _, request)| request.to_owned())
        .collect::<Vec<_>>();
    if requests.is_empty() {
        plan.iter()
            .take(1)
            .map(|(_, _, request)| request.to_owned())
            .collect()
    } else {
        requests
    }
}

pub fn board_spotlight_requests(profile: &Profile) -> Vec<ResourceRequest> {
    let plan = board_plan(profile);
    let picks = profile
        .settings
        .board_spotlight
        .catalogs
        .iter()
        .filter(|catalog| plan.iter().any(|(_, _, request)| request == *catalog))
        .take(SPOTLIGHT_CATALOGS_COUNT)
        .cloned()
        .collect::<Vec<_>>();
    if picks.is_empty() {
        default_spotlight_requests(&plan)
    } else {
        picks
    }
}

fn merge_spotlight_items<'a>(
    pages: impl Iterator<Item = &'a CatalogPage<MetaItemPreview>>,
) -> Vec<(&'a ResourceRequest, &'a MetaItemPreview)> {
    let mut queues = pages
        .filter_map(|page| match &page.content {
            Some(Loadable::Ready(items)) => Some((
                &page.request,
                items.iter().filter(|item| item.background.is_some()),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut seen = HashSet::new();
    let mut slides = vec![];
    while slides.len() < SPOTLIGHT_ITEMS_COUNT {
        let turn = queues
            .iter_mut()
            .filter_map(|(request, items)| {
                items
                    .find(|item| seen.insert(&item.id))
                    .map(|item| (*request, item))
            })
            .collect::<Vec<_>>();
        if turn.is_empty() {
            break;
        }
        slides.extend(turn);
    }
    slides.truncate(SPOTLIGHT_ITEMS_COUNT);
    slides
}

pub fn spotlight_items<'a>(
    pages: impl Iterator<Item = &'a CatalogPage<MetaItemPreview>>,
) -> Vec<(&'a ResourceRequest, &'a MetaItemPreview)> {
    merge_spotlight_items(pages.take_while(|page| spotlight_page_answered(page)))
}

pub fn spotlight_page_answered(page: &CatalogPage<MetaItemPreview>) -> bool {
    matches!(
        page.content,
        Some(Loadable::Ready(_)) | Some(Loadable::Err(_))
    )
}

pub fn spotlight_hides_row(
    page: &CatalogPage<MetaItemPreview>,
    pages: &[CatalogPage<MetaItemPreview>],
) -> bool {
    match &page.content {
        Some(Loadable::Ready(_)) => merge_spotlight_items(pages.iter())
            .iter()
            .any(|(request, _)| **request == page.request),
        Some(Loadable::Err(_)) => false,
        _ => true,
    }
}

pub fn resource_effect<E: Env + 'static>(request: ResourceRequest) -> Effect {
    EffectFuture::Concurrent(
        E::addon_transport(&request.base)
            .resource(&request.path)
            .map(|result| Msg::Internal(Internal::ResourceRequestResult(request, Box::new(result))))
            .boxed_env(),
    )
    .into()
}
