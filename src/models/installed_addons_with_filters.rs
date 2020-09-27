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
    type_name: Option<String>,
}

#[derive(Serialize)]
pub struct InstalledAddonsWithFilters {
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub addons: Vec<DescriptorPreview>,
}

impl InstalledAddonsWithFilters {
    pub fn new(profile: &Profile) -> (Self, Effects) {
        let mut type_names = vec![];
        let effects = type_names_update(&mut type_names, &profile);
        (
            InstalledAddonsWithFilters {
                type_names,
                selected: None,
                addons: vec![],
            },
            effects.unchanged(),
        )
    }
}

impl<E> UpdateWithCtx<Ctx<E>> for InstalledAddonsWithFilters
where
    E: Env + 'static,
{
    fn update(&mut self, ctx: &Ctx<E>, msg: &Msg) -> Effects {
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
                let type_names_effects = type_names_update(&mut self.type_names, &ctx.profile);
                let addons_effects = addons_update(&mut self.addons, &self.selected, &ctx.profile);
                type_names_effects.join(addons_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn type_names_update(type_names: &mut Vec<String>, profile: &Profile) -> Effects {
    let next_type_names = profile
        .addons
        .iter()
        .flat_map(|addon| &addon.manifest.types)
        .unique()
        .cloned()
        .collect::<Vec<_>>();
    if *type_names != next_type_names {
        *type_names = next_type_names;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
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
            .filter(|addon| match &selected.type_name {
                Some(type_name) => addon.manifest.types.contains(type_name),
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
    if *addons != next_addons {
        *addons = next_addons;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
