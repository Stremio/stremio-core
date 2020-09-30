use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use crate::types::library::{LibBucket, LibItem};
use lazysort::SortedBy;
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct ContinueWatchingPreview {
    pub lib_items: Vec<LibItem>,
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for ContinueWatchingPreview {
    fn update(&mut self, ctx: &Ctx<E>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::LibraryChanged(_)) => {
                lib_items_update(&mut self.lib_items, &ctx.library)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn lib_items_update(lib_items: &mut Vec<LibItem>, library: &LibBucket) -> Effects {
    let next_lib_items = library
        .items
        .values()
        .filter(|lib_item| lib_item.is_in_continue_watching())
        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
        .take(CATALOG_PREVIEW_SIZE)
        .cloned()
        .collect::<Vec<_>>();
    if *lib_items != next_lib_items {
        *lib_items = next_lib_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
