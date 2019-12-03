use super::common::{resources_update_with_vector_content, ResourceLoadable, ResourcesAction};
use crate::state_types::messages::{Action, ActionLoad, Event, Internal, Msg};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ExtraProp};
use crate::types::MetaPreview;
use serde_derive::Serialize;
use std::marker::PhantomData;

const CATALOG_CONTENT_LIMIT: usize = 10;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    extra: Vec<ExtraProp>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct CatalogsWithExtra {
    pub selected: Option<Selected>,
    pub catalog_resources: Vec<ResourceLoadable<Vec<MetaPreview>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogsWithExtra {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra { extra })) => {
                let selected_effects =
                    selected_update(&mut self.selected, SelectedAction::Select { extra });
                let catalogs_effects = resources_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllCatalogs { extra },
                        env: PhantomData,
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                resources_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: Some(CATALOG_CONTENT_LIMIT),
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Select { extra: &'a [ExtraProp] },
}

fn selected_update(selected: &mut Option<Selected>, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select { extra } => Some(Selected {
            extra: extra.to_owned(),
        }),
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
