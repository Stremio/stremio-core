use crate::state_types::messages::{Action, ActionLoad, Event, Internal, Msg};
use crate::state_types::models::{Ctx, LibraryLoadable};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::{LibItem, UID};
use derivative::Derivative;
use itertools::Itertools;
use serde_derive::Serialize;

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

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Selected {
    type_name: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryFiltered {
    pub library_state: LibraryState,
    pub selected: Selected,
    pub type_names: Vec<String>,
    pub lib_items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryFiltered {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::LibraryFiltered { type_name })) => {
                let selected_effects =
                    selected_update(&mut self.selected, SelectedAction::Select { type_name });
                let lib_items_effects = lib_items_update(
                    &mut self.lib_items,
                    LibItemsAction::Select {
                        type_name,
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
                        type_name: &self.selected.type_name,
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
    Select { type_name: &'a String },
}

fn selected_update(selected: &mut Selected, action: SelectedAction) -> Effects {
    let next_selected = match action {
        SelectedAction::Select { type_name } => Selected {
            type_name: Some(type_name.to_owned()),
        },
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
            .filter(|x| !x.removed)
            .map(|x| x.type_name.to_owned())
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
    LibraryChanged {
        library: &'a LibraryLoadable,
        type_name: &'a Option<String>,
    },
    Select {
        type_name: &'a String,
        library: &'a LibraryLoadable,
    },
}

fn lib_items_update(lib_items: &mut Vec<LibItem>, action: LibItemsAction) -> Effects {
    let next_lib_items = match action {
        LibItemsAction::LibraryChanged {
            library: LibraryLoadable::Ready(bucket),
            type_name: Some(type_name),
        }
        | LibItemsAction::Select {
            library: LibraryLoadable::Ready(bucket),
            type_name,
        } => bucket
            .items
            .values()
            .filter(|item| !item.removed && item.type_name.eq(type_name))
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
