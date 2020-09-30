use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::models::common::{
    eq_update, resources_update_with_vector_content, ResourceLoadable, ResourcesAction,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{AggrRequest, ExtraProp};
use crate::types::resource::MetaItemPreview;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub extra: Vec<ExtraProp>,
}

#[derive(Default, Serialize)]
pub struct CatalogsWithExtra {
    pub selected: Option<Selected>,
    pub catalog_resources: Vec<ResourceLoadable<Vec<MetaItemPreview>>>,
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for CatalogsWithExtra {
    fn update(&mut self, ctx: &Ctx<E>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let catalogs_effects = resources_update_with_vector_content::<E, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: &ctx.profile.addons,
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let catalogs_effects = eq_update(&mut self.catalog_resources, vec![]);
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::ResourceRequestResult(request, result)) => {
                resources_update_with_vector_content::<E, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourceRequestResult {
                        request,
                        result,
                        limit: &Some(CATALOG_PREVIEW_SIZE),
                    },
                )
            }
            Msg::Internal(Internal::ProfileChanged) => match &self.selected {
                Some(selected) => resources_update_with_vector_content::<E, _>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        request: &AggrRequest::AllCatalogs {
                            extra: &selected.extra,
                        },
                        addons: &ctx.profile.addons,
                    },
                ),
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        }
    }
}
