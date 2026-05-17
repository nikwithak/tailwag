use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field};
use tailwag_utils::{
    macro_utils::{attribute_parsing::GetAttribute, type_parsing::get_type_from_field},
    strings::ToSnakeCase,
};

pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    let table_name = format_ident!("{}", ident.to_string().to_snake_case());

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };
    let filter_type_struct_ident = format_ident!("{}Filters", &ident);

    fn is_base_type(field: &&Field) -> bool {
        match get_type_from_field(field) {
            tailwag_utils::macro_utils::type_parsing::BaseType::Other => {
                // TODO: Wire this through for string-like types!
                // if field.get_attribute("string").is_some() {
                //     true
                // } else {
                false
                // }
            },
            _ => true,
        }
    }

    match &data.fields {
        syn::Fields::Named(fields) => {
            let filterable_fields = fields
                .named
                .iter()
                .filter(|field| field.get_attribute("no_filter").is_none())
                .filter(|field| field.get_attribute("db_ignore").is_none());

            let new_fields = filterable_fields.clone()
            .filter(is_base_type)
            .map(|field| {
                let field_ident = field.ident.clone().expect("Should only have named fields.");
                let orig_type = field.ty.clone();
                quote!(pub #field_ident: tailwag::orm::queries::filterable_types::FilterableType<#orig_type>)
            });

            let default_fields = filterable_fields.clone()
            .filter(is_base_type)
            .map(|field| {
                let field_ident = field.ident.clone().expect("Should only have named fields.");
                let field_ident_str = format!("{table_name}.{field_ident}");
                let orig_type = field.ty.clone();
                quote!(#field_ident: tailwag::orm::queries::filterable_types::FilterableType::<#orig_type>::new(
                    tailwag::orm::data_definition::table::Identifier::new_unchecked(format!("{}{}", prefix.to_string(), #field_ident_str)))
                )
            });

            // These are for filtering based on children
            let child_fields =
                filterable_fields.clone().filter(|f| !is_base_type(f)).map(|field| {
                    let field_ident = field.ident.clone().expect("Should only have named fields.");
                    let orig_type = field.ty.clone();
                    if field.get_attribute("string").is_some() {
                        // To treat enums as string-like types
                        quote!(pub #field_ident: tailwag::orm::queries::filterable_types::FilterableType<String>)
                    } else {
                        quote!(pub #field_ident: <#orig_type as Filterable>::FilterType)
                    }
                });

            let default_child_fields =
                filterable_fields.clone().filter(|f| !is_base_type(f)).map(|field| {
                    let field_ident = field.ident.clone().expect("Should only have named fields.");
                    // let field_ident_str = format!("{table_name}.{field_ident}");
                    let orig_type = field.ty.clone();
                    let table_name = table_name.to_string();
                    if field.get_attribute("string").is_some() {
                        // To treat enums as string-like types
                        let field_ident_str = format!("{table_name}.{field_ident}");
                        quote!(#field_ident: tailwag::orm::queries::filterable_types::FilterableType::<String>::new(
                            tailwag::orm::data_definition::table::Identifier::new_unchecked(format!("{}{}", prefix.to_string(), #field_ident_str)))
                        )
                    } else {
                        quote!(#field_ident: <#orig_type as Filterable>::FilterType::with_prefix(format!("{}{}_",prefix.to_string(),#table_name)))
                    }

                    //     tailwag::orm::queries::filterable_types::FilterableType::<#orig_type>::new(
                    //     tailwag::orm::data_definition::table::Identifier::new_unchecked(#field_ident_str)
                    // ))
                });

            // OUTPUT STARTS HERE
            quote!(
                pub struct #filter_type_struct_ident {
                    prefix: String,
                    #(#new_fields,)*
                    #(#child_fields,)*
                }
                impl tailwag::orm::queries::filterable_types::WithPrefix for #filter_type_struct_ident {
                    fn with_prefix<T: ToString>(prefix: T) -> Self {
                        Self {
                            prefix: prefix.to_string(),
                            #(#default_fields,)*
                            #(#default_child_fields,)*
                        }
                    }
                }
                impl tailwag::orm::queries::filterable_types::Filterable for #ident
                {
                    type FilterType = #filter_type_struct_ident;
                }
            )
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
