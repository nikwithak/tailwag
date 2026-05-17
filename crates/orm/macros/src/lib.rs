use syn::parse_macro_input;
mod logic;
mod util;

#[proc_macro_derive(GetTableDefinition, attributes(db_ignore, ref_only))]
pub fn derive_get_table_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::get_table_definition::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Deleteable, attributes(db_ignore))]
pub fn derive_deleteable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::deleteable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Updateable, attributes(db_ignore, string))]
pub fn derive_updateable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::updateable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(
    Insertable,
    attributes(create_type, db_ignore, string, create_ignore, ref_only)
)]
pub fn derive_insertable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::insertable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Id)]
pub fn derive_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::id::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Filterable, attributes(no_filter, db_ignore, string))]
pub fn derive_filterable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::filterable::derive_struct(&input);
    impl_trait_tokens.into()
}
