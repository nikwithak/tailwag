use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::util::type_parsing::{GetQualifiedPath, IsOption};

fn build_function(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident: _ident,
        data,
        ..
    } = &input;
    let Data::Struct(data) = data else { panic!("Only Structs are supported.")
    };
    let syn::Fields::Named(fields) =  &data.fields else {
        panic!("Unnamed fields found in the struct.")
    };
    let form_items: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let label = field
                .ident
                .as_ref()
                .expect("Found named field with no value for ident")
                .to_string();

            // N.B. Currently only Option<String> is supported, other Option<T> types WILL fail to compile.
            let qualified_path = field.get_qualified_path_for_option();
            let edit_form = match qualified_path.as_str() {
                "std::string::String" | "string::String" | "String" => {
                    // The idea of this trait is to build an egui form that will edit the actual object *in real time*.
                    // Before this, I was trying to implement it through conversion types / typestate, but it turned out
                    // to be really messy and gross. This has the added of benefit of actually updating the data in real time.
                    // The downsie: This is a mess o' spaghetti.

                    // TODO: Factor this into smaller functions for easier readability
                    if field.is_option() {
                        quote!(
                            ui.horizontal(|ui| {
                                ui.label(#label);
                                if let Some(value) = self.#ident.as_mut() {
                                    ui.text_edit_singleline(value);
                                } else {
                                    let mut value = "".to_string();
                                    let text_input = ui.text_edit_singleline(&mut value);
                                    if text_input.changed() {
                                        self.#ident = Some(value);
                                    }
                                }
                                if self.#ident.as_ref().map_or(false, |f| f.eq("")) {
                                    self.#ident = None;
                                }
                            });
                        )
                    } else {
                        quote!(
                            ui.horizontal(|ui| {
                                ui.label(#label);
                                ui.text_edit_singleline(&mut self.#ident);
                            });
                        )
                    }
                },
                "bool" => quote!(ui.checkbox(&mut self.#ident, #label)),
                // Currently UUIDs are just the static Identifier in the data srouce, so they shouldn't be editable.
                // Will need to change in the future.
                "uuid::Uuid" | "Uuid" => {
                    quote!(
                        ui.horizontal(|ui| {
                            ui.label(#label);
                            ui.label(self.#ident.to_string());
                        });
                    )
                },
                "u32" | "u64" | "i32" | "i64" | "usize" | "isize" => todo!(),
                "f32" | "f64" | "fsize" => todo!(),
                "chrono::_" => todo!(),
                _ => {
                    quote!()
                },
            };
            edit_form
        })
        .collect();

    let tokens = quote!(
        fn render(
            &mut self,
            ui: &mut eframe::egui::Ui,
        ) -> eframe::egui::Response {
            ui.vertical(|ui| {
                #(#form_items)*
            })
            .response
        }
    );
    tokens
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
            let functions: Vec<TokenStream> = vec![build_function(input)];
            let parse_args_impl_tokens = quote!(

                // TODO: Fully qualify this
                impl tailwag::gui::widgets::item_manager::item_edit_page::AsEguiForm for #ident {
                    #(#functions)*
                }

            );
            parse_args_impl_tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
