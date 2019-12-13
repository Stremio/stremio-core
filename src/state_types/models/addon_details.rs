use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{descriptor_update, DescriptorAction, DescriptorLoadable};
use crate::state_types::models::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize)]
pub struct AddonDetails {
    pub descriptor: Option<DescriptorLoadable>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for AddonDetails {
    fn update(&mut self, _ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::AddonDetails { transport_url })) => {
                descriptor_update::<Env>(
                    &mut self.descriptor,
                    DescriptorAction::DescriptorRequested { transport_url },
                )
            }
            Msg::Action(Action::Unload) => descriptor_update::<Env>(
                &mut self.descriptor,
                DescriptorAction::DescriptorReplaced { descriptor: None },
            ),
            Msg::Internal(Internal::ManifestResponse(transport_url, response)) => {
                descriptor_update::<Env>(
                    &mut self.descriptor,
                    DescriptorAction::ManifestResponseReceived {
                        transport_url,
                        response,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}
