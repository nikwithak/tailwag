use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::util::type_parsing::{is_option, GetQualifiedPath};

fn build_form_items(input: &DeriveInput) -> Vec<TokenStream> {
    let &DeriveInput {
        ident: _ident,
        data,
        ..
    } = &input;
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported.")
    };
    let syn::Fields::Named(fields) = &data.fields else {
        panic!("Unnamed fields found in the struct.")
    };
    let form_items = fields
        .named
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let label = field
                .ident
                .as_ref()
                .expect("Found named field with no value for ident")
                .to_string();

            let qualified_path = field.get_qualified_path_for_option();
            let is_option = is_option(field);
            let initial_val_tokens = match is_option {
                true => quote!(self.#ident.unwrap_or_default()),
                false => quote!(self.#ident),
            };
            let edit_form = match qualified_path.as_str() {
                "std::string::String" | "string::String" | "String" => {
                    quote!(.with_text_field(#label, #initial_val_tokens, true))
                },
                "bool" => quote!(.with_checkbox(&mut self.#ident, #initial_val_tokens)),
                // UUIDs are read-only  (at least for now)
                "uuid::Uuid" | "Uuid" => {
                    quote!(.with_immutable_text("name", #initial_val_tokens.to_string()))
                },
                "u32" | "u64" | "i32" | "i64" | "usize" | "isize" => todo!(),
                "f32" | "f64" | "fsize" => todo!(),
                "chrono::_" => todo!(),
                _ => {
                    quote!(.with_immutable_text("not ready"))
                },
            };

            edit_form
        })
        .collect();

    form_items
}
pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };
    match &data.fields {
        syn::Fields::Named(fields) => {
            let _field_names = fields.named.iter().map(|f| &f.ident);
            let form_field_tokens: Vec<TokenStream> = build_form_items(input);
            let parse_args_impl_tokens = quote!(

                // TODO: Fully qualify this
                impl follicle::widgets::form::IntoForm for #ident {
                    fn into_form(
                        self,
                    ) -> follicle::widgets::form::Form<Self> {
                        follicle::widgets::form::Form::new(self)
                            #(#form_field_tokens)*
                    }
                }

            );
            parse_args_impl_tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
