use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn build_get_delete_statement(_input: &DeriveInput) -> TokenStream {
    let tokens = quote!(
        fn get_delete_statement(
            &self
        ) -> tailwag::orm::object_management::delete::DeleteStatement<Self> {
            let id_column = tailwag::orm::data_definition::table::TableColumn::uuid("id").unwrap();

            let delete = tailwag::orm::object_management::delete::DeleteStatement::<Self>::new(
                <Self as tailwag::orm::data_manager::GetTableDefinition>::get_table_definition()
                    .clone(),
                tailwag::orm::queries::Filter::Equal(
                    tailwag::orm::queries::FilterComparisonParam::TableColumn(id_column.into()),
                    tailwag::orm::queries::FilterComparisonParam::Uuid(self.id.clone()),
                ),
            );

            delete
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

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };

    match &data.fields {
        syn::Fields::Named(fields) => {
            let _field_names = fields.named.iter().map(|f| &f.ident);
            let functions: Vec<TokenStream> = vec![
                // todo!("Add functions here")
                build_get_delete_statement(input),
            ];

            let parse_args_impl_tokens = quote!(
                impl tailwag::orm::queries::Deleteable for #ident
                where
                    Self: tailwag::orm::data_manager::GetTableDefinition, // To make sure error messages show if we can't use `get_table_definition()`
                {
                    #(#functions)*
                }
            );

            parse_args_impl_tokens
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
