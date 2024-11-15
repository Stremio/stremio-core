use crate::constants::TYPE_PRIORITIES;
use crate::models::common::{compare_with_priorities, eq_update};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::addon::Descriptor;
use crate::types::profile::Profile;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct InstalledAddonsRequest {
    pub r#type: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Selected {
    pub request: InstalledAddonsRequest,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct SelectableType {
    pub r#type: Option<String>,
    pub selected: bool,
    pub request: InstalledAddonsRequest,
}

#[derive(Default, Clone, PartialEq, Eq, Serialize)]
pub struct Selectable {
    pub types: Vec<SelectableType>,
}

#[derive(Default, Clone, Serialize)]
pub struct InstalledAddonsWithFilters {
    pub selected: Option<Selected>,
    pub selectable: Selectable,
    pub catalog: Vec<Descriptor>,
}

impl InstalledAddonsWithFilters {
    pub fn new(profile: &Profile) -> (Self, Effects) {
        let selected = None;
        let mut selectable = Selectable::default();
        let effects = selectable_update(&mut selectable, &selected, profile);
        (
            Self {
                selected,
                selectable,
                ..Self::default()
            },
            effects.unchanged(),
        )
    }
}

impl<E: Env + 'static> UpdateWithCtx<E> for InstalledAddonsWithFilters {
    fn update(&mut self, msg: &Msg, ctx: &Ctx) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::InstalledAddonsWithFilters(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.selected, &ctx.profile);
                let catalog_effects =
                    catalog_update(&mut self.catalog, &self.selected, &ctx.profile);
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.selected, &ctx.profile);
                let catalog_effects =
                    catalog_update(&mut self.catalog, &self.selected, &ctx.profile);
                selected_effects
                    .join(selectable_effects)
                    .join(catalog_effects)
            }
            Msg::Internal(Internal::ProfileChanged) => {
                let selectable_effects =
                    selectable_update(&mut self.selectable, &self.selected, &ctx.profile);
                let catalog_effects =
                    catalog_update(&mut self.catalog, &self.selected, &ctx.profile);
                selectable_effects.join(catalog_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn selectable_update(
    selectable: &mut Selectable,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    let selectable_types = profile
        .addons
        .iter()
        .flat_map(|addon| &addon.manifest.types)
        .unique()
        .cloned()
        .sorted_by(|a, b| compare_with_priorities(a.as_ref(), b.as_ref(), &*TYPE_PRIORITIES))
        .rev()
        .map(|r#type| SelectableType {
            r#type: Some(r#type.to_owned()),
            selected: selected
                .as_ref()
                .and_then(|selected| selected.request.r#type.as_ref())
                .map(|type_| *type_ == r#type)
                .unwrap_or_default(),
            request: InstalledAddonsRequest {
                r#type: Some(r#type),
            },
        });
    let selectable_types = iter::once(SelectableType {
        r#type: None,
        request: InstalledAddonsRequest { r#type: None },
        selected: selected
            .as_ref()
            .map(|selected| selected.request.r#type.is_none())
            .unwrap_or_default(),
    })
    .chain(selectable_types)
    .collect::<Vec<_>>();
    let next_selectable = Selectable {
        types: selectable_types,
    };
    eq_update(selectable, next_selectable)
}

fn catalog_update(
    catalog: &mut Vec<Descriptor>,
    selected: &Option<Selected>,
    profile: &Profile,
) -> Effects {
    let next_catalog = match selected {
        Some(selected) => profile
            .addons
            .iter()
            .filter(|addon| match &selected.request.r#type {
                Some(r#type) => addon.manifest.types.contains(r#type),
                None => true,
            })
            .cloned()
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    eq_update(catalog, next_catalog)
}
