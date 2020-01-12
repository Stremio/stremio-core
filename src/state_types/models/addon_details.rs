use crate::state_types::messages::{Action, ActionLoad, Internal, Msg};
use crate::state_types::models::common::{
    descriptor_update, DescriptorAction, DescriptorContent, DescriptorLoadable,
};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use serde::Serialize;
use std::ops::Deref;

#[derive(Default, Debug, Clone, Serialize)]
pub struct AddonDetails {
    pub addon: Option<DescriptorLoadable>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for AddonDetails {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::AddonDetails(transport_url))) => {
                let addon = ctx
                    .user_data
                    .addons()
                    .iter()
                    .find(|addon| addon.transport_url.eq(transport_url));
                match addon {
                    Some(addon) => descriptor_update::<Env>(
                        &mut self.addon,
                        DescriptorAction::DescriptorReplaced {
                            descriptor: Some(DescriptorLoadable {
                                transport_url: transport_url.to_owned(),
                                content: DescriptorContent::Ready(addon.to_owned()),
                            }),
                        },
                    ),
                    None => descriptor_update::<Env>(
                        &mut self.addon,
                        DescriptorAction::DescriptorRequested { transport_url },
                    ),
                }
            }
            Msg::Action(Action::Unload) => descriptor_update::<Env>(
                &mut self.addon,
                DescriptorAction::DescriptorReplaced { descriptor: None },
            ),
            Msg::Internal(Internal::ManifestRequestResult(transport_url, result)) => {
                descriptor_update::<Env>(
                    &mut self.addon,
                    DescriptorAction::ManifestResultReceived {
                        transport_url,
                        result: result.deref(),
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}
