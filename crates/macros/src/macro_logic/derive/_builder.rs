use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn _derive_builder(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &&input;

    let data = match data {
        Data::Struct(data) => data,
        Data::Enum(_) => unimplemented!("Not implemented for enums"),
        Data::Union(_) => unimplemented!("Not implemented for unions"),
    };

    match &data.fields {
        syn::Fields::Named(fields) => {
            let field_names = fields.named.iter().map(|f| &f.ident);

            // Need to clone the iterator, so we can use it for two different interpolations when
            // making `tokens_builder`. Makes the borrow checker happier - is there an easier / better
            // way to do this when using quote!()?
            let field_names_copy = field_names.clone();

            let field_types = fields.named.iter().map(|f| &f.ty);
            let builder_struct_name = format!("{}Builder", &ident);
            let derive_default = "#[derive(Default)]";

            let builder_tokens = quote!(
                #derive_default
                struct #builder_struct_name {
                    #(#field_names: Option<#field_types>,)*
                }

                impl From<#builder_struct_name> for #ident {
                    fn from(builder: #builder_struct_name) -> Self {
                        Self {
                            #(#field_names_copy: builder.#field_names_copy.unwrap_or_default(),)*
                        }
                    }
                }
            );

            builder_tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
