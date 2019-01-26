use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Default)]
pub struct CatalogMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> CatalogMiddleware<T> {
    pub fn new() -> Self {
        CatalogMiddleware { env: PhantomData }
    }
    fn for_catalog(&self, addon: &AddonDescriptor, cat: &ManifestCatalog, emit: Rc<DispatcherFn>) {
        // @TODO use transport
        // @TODO: better identifier?
        let url = addon.transport_url.replace(
            "/manifest.json",
            &format!("/catalog/{}/{}.json", cat.type_name, cat.id),
        );
        emit(&Action::CatalogRequested(url.to_owned()));
        let req = Request::builder().uri(&url).body(()).unwrap();
        let fut = T::fetch_serde::<(), CatalogResponse>(&req).then(move |res| {
            emit(&match res {
                Ok(resp) => Action::CatalogReceived(url, Ok(*resp)),
                Err(e) => Action::CatalogReceived(url, Err(e.description().to_owned())),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
impl<T: Environment> Handler for CatalogMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO: match on CatalogLoad action
        if let Action::WithAddons(addons, _) = action {
            // @TODO we might need is_catalog_supported
            // https://github.com/Stremio/stremio-aggregators/blob/master/lib/isCatalogSupported.js
            for addon in addons.iter() {
                addon
                    .manifest
                    .catalogs
                    .iter()
                    .filter(|cat| cat.extra_required.is_empty())
                    .for_each(|cat| self.for_catalog(addon, cat, emit.clone()));
            }
        }
    }
}
