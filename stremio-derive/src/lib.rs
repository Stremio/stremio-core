extern crate proc_macro;
use crate::proc_macro::TokenStream;

use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(Model)]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        // @TODO: add proper trait bounds for more sensible errors
        let name = &input.ident;
        let mut fields = named.iter();
        let first = fields.next().expect("at least one field required");
        assert!(
            first.ident.as_ref().map_or(false, |n| n == "ctx"),
            "first field must be named ctx"
        );
        let container_updates = fields.map(|f| {
            let name = &f.ident;
            quote_spanned! {f.span() =>
                .join(self.#name.update(&self.ctx, msg))
            }
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
