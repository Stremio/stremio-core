use crate::models::common::{
    board_plan, board_spotlight_request, eq_update, resource_effect,
    resource_update_with_vector_content, CatalogPage, Loadable, ResourceAction, ResourceLoadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::profile::Profile;
use crate::types::resource::MetaItemPreview;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub all: bool,
}

#[derive(Default, Clone, Serialize, Debug)]
pub struct BoardCatalogs {
    pub selected: Option<Selected>,
    pub catalogs: Vec<CatalogPage<MetaItemPreview>>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for BoardCatalogs {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::BoardCatalogs(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let catalogs_effects =
                    catalogs_update::<E>(&mut self.catalogs, &self.selected, &ctx.profile, false);
                selected_effects.join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalogs_effects = eq_update(&mut self.catalogs, vec![]);
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => self
                .catalogs
                .iter_mut()
                .find(|page| page.request == *request)
                .map(|page| {
                    resource_update_with_vector_content::<E, _>(
                        page,
                        ResourceAction::ResourceRequestResult { request, result },
                    )
                })
                .unwrap_or_else(|| Effects::none().unchanged()),
            Msg::Internal(Internal::ProfileChanged) => {
                catalogs_update::<E>(&mut self.catalogs, &self.selected, &ctx.profile, true)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn catalogs_update<E: Env + 'static>(
    catalogs: &mut Vec<CatalogPage<MetaItemPreview>>,
    selected: &Option<Selected>,
    profile: &Profile,
    keep_errors: bool,
) -> Effects {
    let (next_catalogs, effects) = match selected {
        Some(Selected { all }) => {
            let spotlight_request = board_spotlight_request(profile);
            board_plan(profile)
                .into_iter()
                .map(|(_, _, request)| {
                    catalogs
                        .iter()
                        .find(|page| {
                            page.request == request
                                && match &page.content {
                                    Some(Loadable::Ready(_)) | Some(Loadable::Loading) => true,
                                    Some(Loadable::Err(_)) => keep_errors,
                                    None => false,
                                }
                        })
                        .map(|page| (page.to_owned(), None))
                        .unwrap_or_else(|| {
                            if *all || spotlight_request.as_ref() == Some(&request) {
                                (
                                    ResourceLoadable {
                                        request: request.to_owned(),
                                        content: Some(Loadable::Loading),
                                    },
                                    Some(resource_effect::<E>(request)),
                                )
                            } else {
                                (
                                    ResourceLoadable {
                                        request,
                                        content: None,
                                    },
                                    None,
                                )
                            }
                        })
                })
                .unzip::<_, _, Vec<_>, Vec<_>>()
        }
        None => Default::default(),
    };
    Effects::many(effects.into_iter().flatten().collect())
        .unchanged()
        .join(eq_update(catalogs, next_catalogs))
}
