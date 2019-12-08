use crate::state_types::messages::{Action, ActionLoad, Event, Internal, Msg};
use crate::state_types::models::{Ctx, LibraryLoadable};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::{LibItem, UID};
use derivative::Derivative;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

#[derive(Derivative, Debug, Clone, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type", content = "content")]
pub enum LibraryState {
    #[derivative(Default)]
    NotLoaded,
    Loading {
        uid: UID,
    },
    Ready {
        uid: UID,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortProp {
    Year,
    Name,
    #[serde(rename = "_ctime")]
    CTime,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    type_name: String,
    sort_prop: SortProp,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryFiltered {
    pub library_state: LibraryState,
    pub selected: Option<Selected>,
    pub type_names: Vec<String>,
    pub lib_items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryFiltered {
                type_name,
                sort_prop,
            })) => {
                let selected_effects = selected_update(
                    &mut self.selected,
                    SelectedAction::Select {
                        type_name,
                        sort_prop,
                    },
                );
                let lib_items_effects = lib_items_update(
                    &mut self.lib_items,
                    LibItemsAction::Select {
                        selected: &self.selected,
                        library: &ctx.library,
                    },
                );
                selected_effects.join(lib_items_effects)
            }
            Msg::Internal(Internal::CtxLoaded(_))
            | Msg::Event(Event::CtxChanged)
            | Msg::Internal(Internal::LibLoaded(_))
            | Msg::Event(Event::LibPersisted) => {
                let library_state_effects = library_state_update(
                    &mut self.library_state,
                    LibraryStateAction::LibraryChanged {
                        library: &ctx.library,
                    },
                );
                let type_names_effects = type_names_update(
                    &mut self.type_names,
                    TypeNamesAction::LibraryChanged {
                        library: &ctx.library,
                    },
                );
                let lib_items_effects = lib_items_update(
                    &mut self.lib_items,
                    LibItemsAction::LibraryChanged {
                        library: &ctx.library,
                        selected: &self.selected,
                    },
                );
                library_state_effects
                    .join(type_names_effects)
                    .join(lib_items_effects)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

enum LibraryStateAction<'a> {
    LibraryChanged { library: &'a LibraryLoadable },
}

fn library_state_update(library_state: &mut LibraryState, action: LibraryStateAction) -> Effects {
    let next_library_state = match action {
        LibraryStateAction::LibraryChanged { library } => match library {
            LibraryLoadable::Ready(bucket) => LibraryState::Ready {
                uid: bucket.uid.to_owned(),
            },
            LibraryLoadable::Loading(uid) => LibraryState::Loading {
                uid: uid.to_owned(),
            },
            LibraryLoadable::NotLoaded => LibraryState::NotLoaded,
        },
    };
    if next_library_state.ne(library_state) {
        *library_state = next_library_state;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

enum SelectedAction<'a> {
    Select {
        type_name: &'a String,
        sort_prop: &'a Option<String>,
    },
}

fn selected_update(selected: &mut Option<Selected>, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select {
            type_name,
            sort_prop,
        } => {
            let type_name = type_name.to_owned();
            let sort_prop = match sort_prop {
                Some(sort_prop) => match serde_json::from_str(sort_prop) {
                    Ok(sort_prop) => sort_prop,
                    _ => SortProp::CTime,
                },
                _ => SortProp::CTime,
            };
            Some(Selected {
                type_name,
                sort_prop,
            })
        }
    };
    if next_selected.ne(selected) {
        *selected = next_selected;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

enum TypeNamesAction<'a> {
    LibraryChanged { library: &'a LibraryLoadable },
}

fn type_names_update(type_names: &mut Vec<String>, action: TypeNamesAction) -> Effects {
    let next_type_names = match action {
        TypeNamesAction::LibraryChanged {
            library: LibraryLoadable::Ready(bucket),
        } => bucket
            .items
            .values()
            .filter(|lib_item| !lib_item.removed)
            .map(|lib_item| lib_item.type_name.to_owned())
            .unique()
            .collect(),
        _ => vec![],
    };
    if next_type_names.ne(type_names) {
        *type_names = next_type_names;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

enum LibItemsAction<'a> {
    Select {
        selected: &'a Option<Selected>,
        library: &'a LibraryLoadable,
    },
    LibraryChanged {
        library: &'a LibraryLoadable,
        selected: &'a Option<Selected>,
    },
}

fn lib_items_update(lib_items: &mut Vec<LibItem>, action: LibItemsAction) -> Effects {
    let next_lib_items = match action {
        LibItemsAction::Select {
            selected: Some(selected),
            library: LibraryLoadable::Ready(bucket),
        }
        | LibItemsAction::LibraryChanged {
            library: LibraryLoadable::Ready(bucket),
            selected: Some(selected),
        } => bucket
            .items
            .values()
            .filter(|lib_item| !lib_item.removed && lib_item.type_name.eq(&selected.type_name))
            .sorted_by(|a, b| match &selected.sort_prop {
                SortProp::Year => b.year.cmp(&a.year),
                SortProp::Name => b.name.cmp(&a.name),
                SortProp::CTime => b.ctime.cmp(&a.ctime),
            })
            .cloned()
            .collect(),
        _ => vec![],
    };
    if next_lib_items.ne(lib_items) {
        *lib_items = next_lib_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
