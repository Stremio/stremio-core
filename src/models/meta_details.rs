use crate::constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME};
use crate::models::common::{
    eq_update, resources_update, resources_update_with_vector_content, Loadable, ResourceLoadable,
    ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ResourcePath};
use crate::types::resource::{MetaItem, Stream};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub meta_path: ResourcePath,
    pub streams_path: Option<ResourcePath>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaDetails {
    pub selected: Option<Selected>,
    pub meta_catalogs: Vec<ResourceLoadable<MetaItem>>,
    pub streams_catalogs: Vec<ResourceLoadable<Vec<Stream>>>,
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for MetaDetails {
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let meta_effects = resources_update::<E, _>(
                    &mut self.meta_catalogs,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllOfResource(selected.meta_path.to_owned()),
                        addons: &ctx.profile.addons,
                    },
                );
                let streams_effects = match &selected.streams_path {
                    Some(streams_path) => {
                        if let Some(streams_catalog) =
                            streams_catalog_from_meta_catalog(&self.meta_catalogs, &streams_path.id)
                        {
                            eq_update(&mut self.streams_catalogs, vec![streams_catalog])
                        } else {
                            resources_update_with_vector_content::<E, _>(
                                &mut self.streams_catalogs,
                                ResourcesAction::ResourcesRequested {
                                    request: &AggrRequest::AllOfResource(streams_path.to_owned()),
                                    addons: &ctx.profile.addons,
                                },
                            )
                        }
                    }
                    None => eq_update(&mut self.streams_catalogs, vec![]),
                };
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let meta_effects = eq_update(&mut self.meta_catalogs, vec![]);
                let streams_effects = eq_update(&mut self.streams_catalogs, vec![]);
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == META_RESOURCE_NAME =>
            {
                let meta_effects = resources_update::<E, _>(
                    &mut self.meta_catalogs,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                );
                let streams_effects = match &self.selected {
                    Some(Selected {
                        streams_path: Some(streams_path),
                        ..
                    }) => {
                        if let Some(streams_catalog) =
                            streams_catalog_from_meta_catalog(&self.meta_catalogs, &streams_path.id)
                        {
                            eq_update(&mut self.streams_catalogs, vec![streams_catalog])
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_effects.join(streams_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result))
                if request.path.resource == STREAM_RESOURCE_NAME =>
            {
                resources_update_with_vector_content::<E, _>(
                    &mut self.streams_catalogs,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &None,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn streams_catalog_from_meta_catalog(
    meta_catalogs: &[ResourceLoadable<MetaItem>],
    video_id: &str,
) -> Option<ResourceLoadable<Vec<Stream>>> {
    meta_catalogs
        .iter()
        .find_map(|catalog| match &catalog.content {
            Loadable::Ready(meta_detail) => Some((&catalog.request, meta_detail)),
            _ => None,
        })
        .and_then(|(request, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id == video_id && !video.streams.is_empty())
                .map(|video| (request, &video.streams))
        })
        .map(|(request, streams)| ResourceLoadable {
            request: request.to_owned(),
            content: Loadable::Ready(streams.to_owned()),
        })
}
