use serde::Serialize;
use stremio_core::state_types::messages::{Event, Internal, Msg};
use stremio_core::state_types::models::Ctx;
use stremio_core::state_types::{Effects, Environment, UpdateWithCtx};
use stremio_core::types::addons::{Descriptor, TransportUrl};

#[derive(Default, Debug, Clone, Serialize)]
pub struct InstalledAddons {
    pub transport_urls: Vec<TransportUrl>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for InstalledAddons {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::CtxLoaded(_)) | Msg::Event(Event::CtxChanged) => {
                transport_urls_update(&mut self.transport_urls, &ctx.content.addons)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn transport_urls_update(transport_urls: &mut Vec<TransportUrl>, addons: &[Descriptor]) -> Effects {
    let next_transport_urls = addons
        .iter()
        .map(|addon| &addon.transport_url)
        .cloned()
        .collect::<Vec<_>>();
    if next_transport_urls.ne(transport_urls) {
        *transport_urls = next_transport_urls;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}
