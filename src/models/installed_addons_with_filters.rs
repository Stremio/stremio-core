use crate::models::common::eq_update;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::{DescriptorPreview, ManifestPreview};
use crate::types::profile::Profile;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    r#type: Option<String>,
}

#[derive(Default, Serialize)]
pub struct InstalledAddonsWithFilters {
    pub selected: Option<Selected>,
    pub types: Vec<String>,
    pub addons: Vec<DescriptorPreview>,
}

impl InstalledAddonsWithFilters {
    pub fn new(profile: &Profile) -> (Self, Effects) {
        let mut types = vec![];
        let effects = types_update(&mut types, &profile);
        (
            Self {
                types,
                ..Self::default()
            },
            effects.unchanged(),
        )
    }
}

impl<E> UpdateWithCtx<Ctx<E>> for InstalledAddonsWithFilters
where
    E: Env + 'static,
{
    fn update(&mut self, msg: &Msg, ctx: &Ctx<E>) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::InstalledAddonsWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let addons_effects = addons_update(&mut self.addons, &self.selected, &ctx.profile);
                selected_effects.join(addons_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let addons_effects = addons_update(&mut self.addons, &self.selected, &ctx.profile);
                selected_effects.join(addons_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                let types_effects = types_update(&mut self.types, &ctx.profile);
                let addons_effects = addons_update(&mut self.addons, &self.selected, &ctx.profile);
                types_effects.join(addons_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn types_update(types: &mut Vec<String>, profile: &Profile) -> Effects {
    let next_types = profile
        .addons
        .iter()
        .flat_map(|addon| &addon.manifest.types)
        .unique()
        .cloned()
        .collect::<Vec<_>>();
    eq_update(types, next_types)
}

fn addons_update(
    addons: &mut Vec<DescriptorPreview>,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    let next_addons = match selected {
        Some(selected) => profile
            .addons
            .iter()
            .filter(|addon| match &selected.r#type {
                Some(r#type) => addon.manifest.types.contains(r#type),
                None => true,
            })
            .map(|addon| DescriptorPreview {
                transport_url: addon.transport_url.to_owned(),
                manifest: ManifestPreview {
                    id: addon.manifest.id.to_owned(),
                    version: addon.manifest.version.to_owned(),
                    name: addon.manifest.name.to_owned(),
                    description: addon.manifest.description.to_owned(),
                    logo: addon.manifest.logo.to_owned(),
                    background: addon.manifest.background.to_owned(),
                    types: addon.manifest.types.to_owned(),
                },
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    eq_update(addons, next_addons)
}
