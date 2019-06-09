extern crate proc_macro;
use crate::proc_macro::TokenStream;

use quote::quote;

use syn::{DeriveInput, DataStruct, Data, Fields, FieldsNamed, parse_macro_input};

#[proc_macro_derive(Model)]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { named, .. }), .. }) = input.data {
        // @TODO: shall we assert that the first one needs to be Context?
        let name = &input.ident;
        let expanded = quote! {
            impl crate::state_types::Update for #name {
                fn update(&mut self, msg: &crate::state_types::Msg) -> crate::state_types::Effects {
                    crate::state_types::Effects::none().unchanged()
                }
            }
        };

        TokenStream::from(expanded)
    } else {
       panic!("#[derive(Model)] is only defined for structs with named fields");
    }
}

