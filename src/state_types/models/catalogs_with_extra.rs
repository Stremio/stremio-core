use crate::constants::META_CATALOG_PREVIEW_SIZE;
use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{
    eq_update, resources_update_with_vector_content, validate_extra, ResourceLoadable,
    ResourcesAction,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ExtraProp};
use crate::types::MetaPreview;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    extra: Vec<ExtraProp>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct CatalogsWithExtra {
    pub selected: Option<Selected>,
    pub catalog_resources: Vec<ResourceLoadable<Vec<MetaPreview>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogsWithExtra {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(selected))) => {
                let extra = validate_extra(&selected.extra, &None);
                let selected = Selected { extra };
                let selected_effects = eq_update(&mut self.selected, &Some(selected.to_owned()));
                let catalogs_effects = resources_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        aggr_request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: ctx.user_data.addons(),
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, &None);
                let catalogs_effects = resources_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesReplaced { resources: vec![] },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                resources_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourceResultReceived {
                        request,
                        result,
                        limit: &Some(META_CATALOG_PREVIEW_SIZE),
                    },
                )
            }
            Msg::Internal(Internal::UserDataChanged) => match &self.selected {
                Some(selected) => resources_update_with_vector_content::<Env, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        aggr_request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: ctx.user_data.addons(),
                    },
                ),
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}
