use crate::models::common::{descriptor_update, eq_update, DescriptorAction, DescriptorLoadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::Descriptor;
use crate::types::profile::Profile;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    pub transport_url: Url,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonDetails {
    pub selected: Option<Selected>,
    pub local_addon: Option<Descriptor>,
    pub remote_addon: Option<DescriptorLoadable>,
}

impl<E: Env + 'static> UpdateWithCtx<E> for AddonDetails {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::AddonDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let local_addon_effects =
                    local_addon_update(&mut self.local_addon, &self.selected, &ctx.profile);
                let remote_addon_effects = descriptor_update::<E>(
                    &mut self.remote_addon,
                    DescriptorAction::DescriptorRequested {
                        transport_url: &selected.transport_url,
                    },
                );
                selected_effects
                    .join(local_addon_effects)
                    .join(remote_addon_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let local_addon_effects = eq_update(&mut self.local_addon, None);
                let remote_addon_effects = eq_update(&mut self.remote_addon, None);
                selected_effects
                    .join(local_addon_effects)
                    .join(remote_addon_effects)
            }
            Msg::Internal(Internal::ManifestRequestResult(transport_url, result)) => {
                descriptor_update::<E>(
                    &mut self.remote_addon,
                    DescriptorAction::ManifestRequestResult {
                        transport_url,
                        result,
                    },
                )
            }
            Msg::Internal(Internal::ProfileChanged) => {
                local_addon_update(&mut self.local_addon, &self.selected, &ctx.profile)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn local_addon_update(
    local_addon: &mut Option<Descriptor>,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    let next_local_addon = selected.as_ref().and_then(|selected| {
        profile
            .addons
            .iter()
            .find(|addon| addon.transport_url == selected.transport_url)
            .cloned()
    });
    eq_update(local_addon, next_local_addon)
}
