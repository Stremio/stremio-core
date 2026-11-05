use futures::FutureExt;
use itertools::Itertools;

use crate::{
    constants::{CALENDAR_IDS_EXTRA_PROP, LAST_VIDEOS_IDS_EXTRA_PROP},
    models::common::ResourceLoadable,
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

pub fn board_spotlight_request(profile: &Profile) -> Option<ResourceRequest> {
    let plan = board_plan(profile);
    profile
        .settings
        .board_spotlight
        .catalog
        .as_ref()
        .filter(|catalog| plan.iter().any(|(_, _, request)| request == *catalog))
        .cloned()
        .or_else(|| plan.into_iter().next().map(|(_, _, request)| request))
}

pub fn spotlight_items(items: &[MetaItemPreview]) -> impl Iterator<Item = &MetaItemPreview> {
    items
        .iter()
        .filter(|item| item.background.is_some())
        .unique_by(|item| &item.id)
        .take(SPOTLIGHT_ITEMS_COUNT)
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
