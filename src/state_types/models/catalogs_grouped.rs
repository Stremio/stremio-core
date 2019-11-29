use super::common::{catalogs_update, Catalog, CatalogsAction};
use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ExtraProp};
use crate::types::MetaPreview;
use serde_derive::Serialize;
use std::marker::PhantomData;

#[derive(Default, Debug, Clone, Serialize)]
pub struct CatalogsGrouped {
    pub selected: Vec<ExtraProp>,
    pub groups: Vec<Catalog<Vec<MetaPreview>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogsGrouped {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsGrouped { extra })) => {
                let selected_effects =
                    selected_update(&mut self.selected, SelectedAction::Select { extra });
                let groups_effects = catalogs_update::<_, Env>(
                    &mut self.groups,
                    CatalogsAction::CatalogsRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllCatalogs { extra },
                        env: PhantomData,
                    },
                );
                selected_effects.join(groups_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => catalogs_update::<_, Env>(
                &mut self.groups,
                CatalogsAction::CatalogResponseReceived {
                    request,
                    response,
                    limit: Some(100),
                },
            ),
            _ => Effects::none().unchanged(),
        }
    }
}

enum SelectedAction<'a> {
    Select { extra: &'a [ExtraProp] },
}

fn selected_update(selected: &mut Vec<ExtraProp>, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select { extra } => extra.to_owned(),
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
