use itertools::Itertools;
use serde::Serialize;
use stremio_core::state_types::models::ctx::Ctx;
use stremio_core::state_types::msg::{Internal, Msg};
use stremio_core::state_types::{Effects, Environment, UpdateWithCtx};
use stremio_core::types::LibBucket;

#[derive(Default, Debug, Clone, Serialize)]
pub struct LibraryItems {
    pub ids: Vec<String>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for LibraryItems {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::LibraryChanged(_)) => ids_update(&mut self.ids, &ctx.library),
            _ => Effects::none().unchanged(),
        }
    }
}

fn ids_update(ids: &mut Vec<String>, library: &LibBucket) -> Effects {
    let next_ids = library
        .items
        .values()
        .filter(|lib_item| !lib_item.removed)
        .map(|lib_item| &lib_item.id)
        .cloned()
        .unique()
        .collect::<Vec<_>>();
    if next_ids.ne(ids) {
        *ids = next_ids;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
