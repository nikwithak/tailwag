use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

fn build_get_update_statement(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident: _ident,
        ..
    } = &input;
    let input_table_definition =
        crate::util::database_table_definition::build_table_definition::<()>(input);

    let update_maps = input_table_definition.columns.values().map(|column| {
        let column_name = format_ident!("{}", column.column_name.to_string());
        let mut field_name = column_name.clone();
        let column_name_as_string = column.column_name.to_string();

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;
        let wrapped_type = match &column.column_type {
            E::Boolean => quote!(tailwag::orm::data_definition::table::ColumnValue::Boolean(#column_name.clone())),
            E::Int => quote!(tailwag::orm::data_definition::table::ColumnValue::Int(#column_name.clone())),
            E::Float => quote!(tailwag::orm::data_definition::table::ColumnValue::Float(#column_name.clone())),
            E::String => quote!(tailwag::orm::data_definition::table::ColumnValue::String(#column_name.to_string())),
            E::Timestamp => quote!(tailwag::orm::data_definition::table::ColumnValue::Timestamp(#column_name.clone())),
            E::Uuid => quote!(tailwag::orm::data_definition::table::ColumnValue::Uuid(#column_name.clone())),
            E::Json => quote!(tailwag::orm::data_definition::table::ColumnValue::Json(#column_name.to_string())),
            E::OneToOne{ref_only: false, ..}=>{
                field_name = format_ident!("{}", column.column_name.trim_end_matches("_id").to_string()); // Hack to work around soem ugliness with the DataDefinition / column mapping
                // TODO: This assumes ID.
                quote!(
                    {
                        let stmt = #field_name.get_update_statement();
                        tailwag::orm::data_definition::table::ColumnValue::OneToOne{child_table: stmt.table_name(), value: Box::new(stmt.object_repr().clone())}
                    }
                )
                // todo!()
            },
            E::OneToOne{ref_only: true, ..}=> {
                field_name = format_ident!("{}", column.column_name.trim_end_matches("_id").to_string()); // Hack to work around soem ugliness with the DataDefinition / column mapping
                quote!(
                    {
                        use tailwag::orm::data_manager::rest_api::Id;
                        // TODO: This still assumes ID...
                        tailwag::orm::data_definition::table::ColumnValue::Uuid(#field_name.id().clone())
                    }
                )
            } ,
            E::OneToMany(_child_type, _) => {
                quote!(
                    {
                        let insert_statements = #field_name.iter().map(|child|Box::new(child.get_update_statement()));
                        let child_table = insert_statements.clone().find_map(|stmt|Some(stmt.table_name())).unwrap_or(
                            tailwag::orm::data_definition::table::Identifier::new_unchecked("__nothin_to_insert__")
                        );
                        let values = insert_statements.into_iter().map(|stmt|Box::new(stmt.object_repr().clone())).collect();
                        tailwag::orm::data_definition::table::ColumnValue::OneToMany{child_table, values}
                    }
                )
            },
            tailwag_orm::data_definition::table::DatabaseColumnType::ManyToMany(_, _) => todo!("{:?} is a ManyToMany relationship that isn't yet supported", &column.column_type),
        };

        if column.is_nullable() {
            quote!(
                if let Some(#field_name) = &self.#field_name {
                    update_map.insert(
                        tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string).expect("Invalid column identifier found - this should not happen. Panicking."),
                        #wrapped_type,
                    );
                }
                // TODO: Re-null fields
            )
        } else {
            quote!(
                let #field_name = &self.#field_name;
                update_map.insert(
                    tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string.to_string()).expect("Invalid column identifier found - this should not happen. Panicking."),
                    // TODO: Can't support differently named column/struct_attr names right now
                    // TODO: Only supports string-like types, apparetly?
                    #wrapped_type,
                );
            )
        }
    });

    let tokens = quote!(
        fn get_update_statement(&self) -> tailwag::orm::object_management::update::UpdateStatement {
            let mut update_map = std::collections::HashMap::new();

            #(#update_maps)*

            let update = tailwag::orm::object_management::update::UpdateStatement::new(
                <Self as tailwag::orm::data_manager::GetTableDefinition>::get_table_definition().clone(),
                update_map,
            );

            update
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
                build_get_update_statement(input),
            ];

            let parse_args_impl_tokens = quote!(
                impl tailwag::orm::queries::Updateable for #ident
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
