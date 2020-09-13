use crate::models::common::eq_update;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Environment, UpdateWithCtx};
use crate::types::addon::{Descriptor, DescriptorPreview, ManifestPreview};
use serde::Serialize;

pub type Selected = String;

#[derive(Default, Serialize)]
pub struct InstalledAddonsWithFilters {
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub addons: Vec<DescriptorPreview>,
}

impl<Env> UpdateWithCtx<Ctx<Env>> for InstalledAddonsWithFilters
where
    Env: Environment + 'static,
{
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::InstalledAddonsWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let addons_effects =
                    addons_update(&mut self.addons, &self.selected, &ctx.profile.addons);
                selected_effects.join(addons_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let addons_effects =
                    addons_update(&mut self.addons, &self.selected, &ctx.profile.addons);
                selected_effects.join(addons_effects)
            }
            Msg::Internal(Internal::ProfileChanged(_)) => {
                let type_names_effects =
                    type_names_update(&mut self.type_names, &ctx.profile.addons);
                let addons_effects =
                    addons_update(&mut self.addons, &self.selected, &ctx.profile.addons);
                type_names_effects.join(addons_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn type_names_update(type_names: &mut Vec<String>, installed_addons: &[Descriptor]) -> Effects {
    let next_type_names = installed_addons
        .iter()
        .flat_map(|addon| &addon.manifest.types)
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
    installed_addons: &[Descriptor],
) -> Effects {
    let next_addons = match selected {
        Some(type_name) => installed_addons
            .iter()
            .filter(|addon| addon.manifest.types.contains(type_name))
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
