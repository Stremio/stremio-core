use super::common::{catalogs_update_with_vector_content, Catalog, CatalogsAction};
use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ExtraProp};
use crate::types::MetaPreview;
use serde_derive::Serialize;
use std::marker::PhantomData;

const CATALOG_CONTENT_LIMIT: usize = 10;

#[derive(Default, Debug, Clone, Serialize)]
pub struct CatalogsWithExtra {
    pub selected: Vec<ExtraProp>,
    pub catalogs: Vec<Catalog<Vec<MetaPreview>>>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for CatalogsWithExtra {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra { extra })) => {
                let selected_effects =
                    selected_update(&mut self.selected, SelectedAction::Select { extra });
                let catalogs_effects = catalogs_update_with_vector_content::<_, Env>(
                    &mut self.catalogs,
                    CatalogsAction::CatalogsRequested {
                        addons: &ctx.content.addons,
                        request: &AggrRequest::AllCatalogs { extra },
                        env: PhantomData,
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                catalogs_update_with_vector_content::<_, Env>(
                    &mut self.catalogs,
                    CatalogsAction::CatalogResponseReceived {
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

fn selected_update(selected: &mut Vec<ExtraProp>, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select { extra } => extra.to_owned(),
    };
    if selected.iter().ne(next_selected.iter()) {
        *selected = next_selected.to_owned();
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
