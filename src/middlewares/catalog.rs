use crate::state_types::*;
use crate::types::*;
use futures::{future,Future};
use std::marker::PhantomData;
use std::rc::Rc;

pub struct CatalogMiddleware<T: Environment>{
    //id: usize,
    pub env: PhantomData<T>,
}
impl<T: Environment> CatalogMiddleware<T> {
    fn for_catalog(&self, addon: &AddonDescriptor, cat: &ManifestCatalog, emit: Rc<DispatcherFn>) {
        // @TODO use transport
        let url = addon.transport_url.replace("/manifest.json", &format!("/catalog/{}/{}.json", cat.type_name, cat.id));
        emit(&Action::CatalogRequested(url.to_owned()));
        let fut =  T::fetch_serde::<CatalogResponse>(url.to_owned())
            .then(move |res| {
                emit(&match res {
                    Ok(resp) => Action::CatalogReceived(url, Ok(*resp)),
                    Err(e) => Action::CatalogReceived(url, Err(e.description().to_owned())),
                });
                future::ok(())
            });
        T::exec(Box::new(fut));
    }
}
impl<T> Handler for CatalogMiddleware<T> where T: Environment {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        match action {
            // @TODO: match on CatalogLoad in particular
            Action::WithAddons(addons, _) => {
                for addon in addons.iter() {
                    for cat in addon.manifest.catalogs.iter() {
                        self.for_catalog(addon, cat, emit.clone());
                    }
                }
            },
            _ => {}
        }
    }
}
