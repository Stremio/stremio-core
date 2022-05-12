use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::models::common::{
    eq_update, resources_update_with_vector_content, ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ExtraValue};
use crate::types::resource::MetaItemPreview;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub extra: Vec<ExtraValue>,
}

#[derive(Default, Serialize)]
pub struct CatalogsWithExtra {
    pub selected: Option<Selected>,
    pub catalogs: Vec<ResourceLoadable<Vec<MetaItemPreview>>>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for CatalogsWithExtra {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let catalogs_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.catalogs,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: &ctx.profile.addons,
                        range: None,
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalogs_effects = eq_update(&mut self.catalogs, vec![]);
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                resources_update_with_vector_content::<E, _>(
                    &mut self.catalogs,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: Some(CATALOG_PREVIEW_SIZE),
                    },
                )
            }
            Msg::Internal(Internal::ProfileChanged) => match &self.selected {
                Some(selected) => resources_update_with_vector_content::<E, _>(
                    &mut self.catalogs,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: &ctx.profile.addons,
                        range: None,
                    },
                ),
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}
