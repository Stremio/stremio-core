use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{
    catalogs_update, catalogs_update_with_vector_content, Catalog, CatalogContent, CatalogsAction,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ResourceRef};
use crate::types::{MetaDetail, Stream};
use serde_derive::Serialize;
use std::marker::PhantomData;

const META_RESOURCE_NAME: &str = "meta";
const STREAM_RESOURCE_NAME: &str = "stream";

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    meta_resource_ref: Option<ResourceRef>,
    streams_resource_ref: Option<ResourceRef>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct MetaDetails {
    pub selected: Selected,
    pub meta_groups: Vec<Catalog<MetaDetail>>,
    pub streams_groups: Vec<Catalog<Vec<Stream>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for MetaDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::MetaDetails {
                type_name,
                id,
                video_id,
            })) => {
                let selected_effects = selected_update(
                    &mut self.selected,
                    SelectedAction::Select {
                        type_name,
                        id,
                        video_id,
                    },
                );
                let meta_effects = catalogs_update::<_, Env>(
                    &mut self.meta_groups,
                    CatalogsAction::CatalogsRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllOfResource(ResourceRef::without_extra(
                            META_RESOURCE_NAME,
                            type_name,
                            id,
                        )),
                        env: PhantomData,
                    },
                );
                let streams_effects = match video_id {
                    Some(video_id) => {
                        if let Some(streams_group) =
                            streams_group_from_meta_groups(&self.meta_groups, video_id)
                        {
                            catalogs_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                CatalogsAction::CatalogsReplaced {
                                    catalogs: vec![streams_group],
                                },
                            )
                        } else {
                            catalogs_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                CatalogsAction::CatalogsRequested {
                                    addons: &ctx.content.addons,
                                    request: &AggrRequest::AllOfResource(
                                        ResourceRef::without_extra(
                                            STREAM_RESOURCE_NAME,
                                            type_name,
                                            video_id,
                                        ),
                                    ),
                                    env: PhantomData,
                                },
                            )
                        }
                    }
                    None => catalogs_update_with_vector_content::<_, Env>(
                        &mut self.streams_groups,
                        CatalogsAction::CatalogsReplaced { catalogs: vec![] },
                    ),
                };
                selected_effects.join(meta_effects).join(streams_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response))
                if request.path.resource.eq(META_RESOURCE_NAME) =>
            {
                let meta_effects = catalogs_update::<_, Env>(
                    &mut self.meta_groups,
                    CatalogsAction::CatalogResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                );
                let streams_effects = match &self.selected {
                    Selected {
                        streams_resource_ref: Some(streams_resource_ref),
                        ..
                    } => {
                        if let Some(streams_group) = streams_group_from_meta_groups(
                            &self.meta_groups,
                            &streams_resource_ref.id,
                        ) {
                            catalogs_update_with_vector_content::<_, Env>(
                                &mut self.streams_groups,
                                CatalogsAction::CatalogsReplaced {
                                    catalogs: vec![streams_group],
                                },
                            )
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                meta_effects.join(streams_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response))
                if request.path.resource.eq(STREAM_RESOURCE_NAME) =>
            {
                catalogs_update_with_vector_content::<_, Env>(
                    &mut self.streams_groups,
                    CatalogsAction::CatalogResponseReceived {
                        request,
                        response,
                        limit: None,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Select {
        type_name: &'a String,
        id: &'a String,
        video_id: &'a Option<String>,
    },
}

fn selected_update(selected: &mut Selected, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select {
            type_name,
            id,
            video_id: Some(video_id),
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra(
                META_RESOURCE_NAME,
                type_name,
                id,
            )),
            streams_resource_ref: Some(ResourceRef::without_extra(
                STREAM_RESOURCE_NAME,
                type_name,
                video_id,
            )),
        },
        SelectedAction::Select {
            type_name,
            id,
            video_id: None,
        } => Selected {
            meta_resource_ref: Some(ResourceRef::without_extra(
                META_RESOURCE_NAME,
                type_name,
                id,
            )),
            streams_resource_ref: None,
        },
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

fn streams_group_from_meta_groups(
    meta_groups: &[Catalog<MetaDetail>],
    video_id: &str,
) -> Option<Catalog<Vec<Stream>>> {
    meta_groups
        .iter()
        .find_map(|meta_catalog| match &meta_catalog.content {
            CatalogContent::Ready(meta_detail) => Some((&meta_catalog.request, meta_detail)),
            _ => None,
        })
        .and_then(|(request, meta_detail)| {
            meta_detail
                .videos
                .iter()
                .find(|video| video.id.eq(video_id) && !video.streams.is_empty())
                .map(|video| (request, &video.streams))
        })
        .map(|(request, streams)| Catalog {
            request: request.to_owned(),
            content: CatalogContent::Ready(streams.to_owned()),
        })
}
