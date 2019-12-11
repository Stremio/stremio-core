use itertools::Itertools;
use serde::Serialize;
use stremio_core::state_types::messages::{Event, Internal, Msg};
use stremio_core::state_types::models::{Ctx, LibraryLoadable};
use stremio_core::state_types::{Effects, Environment, UpdateWithCtx};

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryItems {
    pub ids: Vec<String>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryItems {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::CtxLoaded(_))
            | Msg::Event(Event::CtxChanged)
            | Msg::Internal(Internal::LibLoaded(_))
            | Msg::Event(Event::LibPersisted) => ids_update(&mut self.ids, &ctx.library),
            _ => Effects::none().unchanged(),
        }
    }
}

fn ids_update(ids: &mut Vec<String>, library: &LibraryLoadable) -> Effects {
    let next_ids = match library {
        LibraryLoadable::Ready(bucket) => bucket
            .items
            .values()
            .filter(|lib_item| !lib_item.removed)
            .map(|lib_item| &lib_item.id)
            .cloned()
            .unique()
            .collect(),
        _ => vec![],
    };
    if next_ids.ne(ids) {
        *ids = next_ids;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
