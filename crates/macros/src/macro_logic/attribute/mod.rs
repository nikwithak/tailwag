use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

#[allow(unused)]
fn build_function_definition(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    let _table_name = tailwag_utils::strings::to_snake_case(&ident.to_string());
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported.")
    };
    let syn::Fields::Named(_fields) = &data.fields else {
        panic!("Unnamed fields found in the struct.")
    };
    let tokens = quote!(
        fn my_function() {}
    );
    tokens
}

#[allow(unused)]
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
            let functions: Vec<TokenStream> = vec![build_function_definition(input)];
            let parse_args_impl_tokens = quote! {
              impl tailwag::web_service::traits::BuildRoutes for #ident {
                #(#functions)*
              }
            };
            parse_args_impl_tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
