use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };

    match &data.fields {
        syn::Fields::Named(_) => {
            quote!(
                impl tailwag::orm::data_manager::rest_api::Id for #ident
                {
                    fn id(&self) -> &uuid::Uuid {
                        &self.id
                    }
                }
            )
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
