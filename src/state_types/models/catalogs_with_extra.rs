use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::state_types::messages::{Action, ActionLoad, Event, Internal, Msg};
use crate::state_types::models::common::{
    resources_update_with_vector_content, validate_extra, ResourceLoadable, ResourcesAction,
};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::addons::{AggrRequest, ExtraProp};
use crate::types::MetaPreview;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
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
            Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra { extra })) => {
                let extra = validate_extra(extra);
                let selected_effects =
                    selected_update(&mut self.selected, SelectedAction::Select { extra: &extra });
                let catalogs_effects = resources_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourcesRequested {
                        aggr_request: &AggrRequest::AllCatalogs { extra: &extra },
                        addons: &ctx.content.addons,
                    },
                );
                selected_effects.join(catalogs_effects)
            }
            Msg::Action(Action::Unload) => {
                let next = CatalogsWithExtra::default();
                if next.ne(&self) {
                    *self = next;
                    Effects::none()
                } else {
                    Effects::none().unchanged()
                }
            }
            Msg::Internal(Internal::AddonResponse(request, response)) => {
                resources_update_with_vector_content::<_, Env>(
                    &mut self.catalog_resources,
                    ResourcesAction::ResourceResponseReceived {
                        request,
                        response,
                        limit: Some(CATALOG_PREVIEW_SIZE),
                    },
                )
            }
            Msg::Internal(Internal::CtxLoaded(_)) | Msg::Event(Event::CtxChanged) => {
                if let Some(selected) = &self.selected {
                    resources_update_with_vector_content::<_, Env>(
                        &mut self.catalog_resources,
                        ResourcesAction::ResourcesRequested {
                            aggr_request: &AggrRequest::AllCatalogs {
                                extra: &selected.extra,
                            },
                            addons: &ctx.content.addons,
                        },
                    )
                } else {
                    Effects::none().unchanged()
                }
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
