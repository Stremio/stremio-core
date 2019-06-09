extern crate proc_macro;
use crate::proc_macro::TokenStream;

use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, DataStruct, Data, Fields, FieldsNamed, parse_macro_input};

#[proc_macro_derive(Model)]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { named, .. }), .. }) = input.data {
        // @TODO: assert that the first one needs to be named 'ctx'
        // @TODO: add proper trait bounds for more sensible errors
        let name = &input.ident;
        let container_updates = named
            .iter()
            .filter_map(|f| {
                let name = &f.ident;
                if name.as_ref().map_or(true, |n| n == "ctx") {
                    return None;
                }
                Some(quote_spanned! {f.span() =>
                    .join(self.#name.update(&self.ctx, msg))
                })
            });
        let expanded = quote! {
            impl crate::state_types::Update for #name {
                fn update(&mut self, msg: &crate::state_types::Msg) -> crate::state_types::Effects {
                    self.ctx.update(msg)
                        #(#container_updates)*
                }
            }
        };

        TokenStream::from(expanded)
    } else {
       panic!("#[derive(Model)] is only defined for structs with named fields");
    }
}

