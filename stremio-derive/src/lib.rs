use case::CaseExt;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
use proc_macro_roids::IdentExt;
use quote::quote;
use std::borrow::Cow;
use std::{env, iter};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};

const CORE_CRATE_ORIGINAL_NAME: &str = "stremio-core";

#[proc_macro_derive(Model, attributes(model))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            assert!(
                fields
                    .named
                    .iter()
                    .any(|field| field.ident.as_ref().unwrap() == "ctx"),
                "ctx field is required"
            );
            let core_ident = get_core_ident().unwrap();
            let struct_ident = &input.ident;
            let env_ident = input
                .attrs
                .into_iter()
                .find(|attr| attr.path.is_ident("model"))
                .expect("model attribute required")
                .parse_args::<syn::Ident>()
                .expect("model attribute parse failed");
            let field_enum_ident = struct_ident.append("Field");
            let field_enum_variant_idents = fields
                .named
                .iter()
                .map(|field| {
                    Ident::new(
                        &field.ident.as_ref().unwrap().to_string().to_camel(),
                        Span::call_site(),
                    )
                })
                .collect::<Vec<_>>();
            let field_update_match_arms = fields
                .named
                .iter()
                .zip(field_enum_variant_idents.iter())
                .filter(|(field, _)| field.ident.as_ref().unwrap() != "ctx")
                .map(|(field, variant_ident)| {
                    let field_ident = &field.ident;
                    quote! {
                        Self::Field::#variant_ident => #core_ident::runtime::UpdateWithCtx::<#env_ident>::update(&mut self.#field_ident, &msg, &self.ctx)
                    }
                })
                .chain(iter::once(quote! {
                    Ctx => #core_ident::runtime::Update::<#env_ident>::update(&mut self.ctx, msg)
                }))
                .collect::<Vec<_>>();
            let field_updates = fields
                .named
                .iter()
                .filter(|field| field.ident.as_ref().unwrap() != "ctx")
                .map(|field| {
                    let field_ident = &field.ident;
                    quote! {
                        .join(#core_ident::runtime::UpdateWithCtx::<#env_ident>::update(&mut self.#field_ident, &msg, &self.ctx))
                    }
                })
                .chain(iter::once(quote! {
                    #core_ident::runtime::Update::<#env_ident>::update(&mut self.ctx, msg)
                }))
                .rev()
                .collect::<Vec<_>>();
            TokenStream::from(quote! {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "snake_case")]
                pub enum #field_enum_ident {
                    #(#field_enum_variant_idents),*
                }

                impl #core_ident::runtime::Update<#env_ident> for #struct_ident {
                    fn update(&mut self, msg: &#core_ident::runtime::msg::Msg) -> #core_ident::runtime::Effects {
                        #(#field_updates)*
                    }
                }

                impl #core_ident::runtime::Model<#env_ident> for #struct_ident {
                    type Field = #field_enum_ident;
                    fn update_field(&mut self, msg: &#core_ident::runtime::msg::Msg, field: &Self::Field) -> #core_ident::runtime::Effects {
                        match field {
                            #(#field_update_match_arms),*
                        }
                    }
                }
            })
        }
        _ => panic!("#[derive(Model)] is only defined for structs with named fields"),
    }
}

fn get_core_ident() -> Result<Ident, String> {
    let core_crate_name = match env::var("CARGO_PKG_NAME") {
        Ok(cargo_pkg_name) if cargo_pkg_name == CORE_CRATE_ORIGINAL_NAME => Cow::Borrowed("crate"),
        _ => Cow::Owned(crate_name(CORE_CRATE_ORIGINAL_NAME)?),
    };
    Ok(Ident::new(&core_crate_name, Span::call_site()))
}
