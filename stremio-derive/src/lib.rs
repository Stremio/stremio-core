use case::CaseExt;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
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
            let struct_ident = input.ident;
            let env_ident = input
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("model"))
                .expect("model attribute required")
                .parse_args::<Ident>()
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
                .map(|(field, variant_ident)| {
                    let field_ident = field.ident.as_ref().unwrap();
                    if field_ident == "ctx" {
                        quote! {
                            Self::Field::#variant_ident => {
                                let ctx_effects = #core_ident::runtime::Update::<#env_ident>::update(&mut self.#field_ident, &msg);
                                let fields = if ctx_effects.has_changed {
                                    vec![Self::Field::#variant_ident]
                                } else {
                                    vec![]
                                };
                                let effects = ctx_effects.into_iter().collect::<Vec<_>>();
                                (effects, fields)
                            }
                        }
                    } else {
                        quote! {
                            Self::Field::#variant_ident => {
                                let model_effects = #core_ident::runtime::UpdateWithCtx::<#env_ident>::update(&mut self.#field_ident, &msg, &self.ctx);
                                let fields = if model_effects.has_changed {
                                    vec![Self::Field::#variant_ident]
                                } else {
                                    vec![]
                                };
                                let effects = model_effects.into_iter().collect::<Vec<_>>();
                                (effects, fields)
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            let field_updates_chain = fields
                .named
                .iter()
                .zip(field_enum_variant_idents.iter())
                .filter(|(field, _)| field.ident.as_ref().unwrap() != "ctx")
                .map(|(field, variant_ident)| {
                    let field_ident = field.ident.as_ref().unwrap();
                    quote! {
                        let model_effects = #core_ident::runtime::UpdateWithCtx::<#env_ident>::update(&mut self.#field_ident, &msg, &self.ctx);
                        if model_effects.has_changed {
                            fields.push(#field_enum_ident::#variant_ident);
                        };
                        effects.extend(model_effects.into_iter());
                    }
                })
                .chain(iter::once(quote! {
                    let mut effects = vec![];
                    let mut fields = vec![];

                    let ctx_effects = #core_ident::runtime::Update::<#env_ident>::update(&mut self.ctx, msg);
                    if ctx_effects.has_changed {
                        fields.push(#field_enum_ident::Ctx);
                    };
                    effects.extend(ctx_effects.into_iter());
                }))
                .rev()
                .chain(iter::once(quote! {
                    (effects, fields)
                }))
                .collect::<Vec<_>>();
            TokenStream::from(quote! {
                #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
                #[serde(rename_all = "snake_case")]
                pub enum #field_enum_ident {
                    #(#field_enum_variant_idents),*
                }

                impl #core_ident::runtime::Model<#env_ident> for #struct_ident {
                    type Field = #field_enum_ident;

                    fn update(&mut self, msg: &#core_ident::runtime::msg::Msg) -> (Vec<#core_ident::runtime::Effect>, Vec<Self::Field>) {
                        #(#field_updates_chain)*
                    }

                    fn update_field(&mut self, msg: &#core_ident::runtime::msg::Msg, field: &Self::Field) -> (Vec<#core_ident::runtime::Effect>, Vec<Self::Field>) {
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

fn get_core_ident() -> Result<Ident, proc_macro_crate::Error> {
    let core_crate_name = match env::var("CARGO_PKG_NAME") {
        Ok(cargo_pkg_name) if cargo_pkg_name == CORE_CRATE_ORIGINAL_NAME => Cow::Borrowed("crate"),
        _ => match crate_name(CORE_CRATE_ORIGINAL_NAME)? {
            FoundCrate::Itself => Cow::Borrowed("crate"),
            FoundCrate::Name(name) => Cow::Owned(name),
        },
    };
    Ok(Ident::new(&core_crate_name, Span::call_site()))
}
