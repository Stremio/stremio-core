use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{manifest_update, ManifestAction, ManifestLoadable};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize)]
pub struct AddonDetails {
    pub manifest: Option<ManifestLoadable>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for AddonDetails {
    fn update(&mut self, _ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::AddonDetails { transport_url })) => {
                manifest_update::<Env>(
                    &mut self.manifest,
                    ManifestAction::ManifestRequested { transport_url },
                )
            }
            Msg::Action(Action::Unload) => manifest_update::<Env>(
                &mut self.manifest,
                ManifestAction::ManifestReplaced { manifest: None },
            ),
            Msg::Internal(Internal::ManifestResponse(transport_url, response)) => {
                manifest_update::<Env>(
                    &mut self.manifest,
                    ManifestAction::ManifestResponseReceived {
                        transport_url,
                        response,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}
