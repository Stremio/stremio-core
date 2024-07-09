use crate::models::common::{descriptor_update, eq_update, DescriptorAction, DescriptorLoadable};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::Descriptor;
use crate::types::profile::Profile;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selected {
    #[serde(deserialize_with = "deserialize_transport_url")]
    pub transport_url: Url,
}

fn deserialize_transport_url<'de, D>(de: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let url = Url::deserialize(de)?;

    if url.scheme() == "stremio" {
        let replaced_url = url.as_str().replacen("stremio://", "https://", 1);

        Ok(replaced_url.parse().expect("Should be able to parse URL"))
    } else {
        Ok(url)
    }
}

#[derive(Default, Clone, Serialize)]
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

#[cfg(test)]
mod test {
    use super::*;

    use serde_json::{from_value, json};

    #[test]
    fn test_deserialization_of_select() {
        let cases = [
            ("stremio://transport_url.com", "https://transport_url.com/"),
            ("http://transport_url.com", "http://transport_url.com/"),
            ("https://transport_url.com", "https://transport_url.com/"),
        ];

        for (transport_url, expected) in cases {
            let selected_json = json!({ "transportUrl": transport_url });

            let selected = from_value::<Selected>(selected_json).expect("Should deserialize");

            assert_eq!(expected, selected.transport_url.as_str());
        }
    }
}
