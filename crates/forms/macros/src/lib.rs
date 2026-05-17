mod logic;
use syn::parse_macro_input;

#[proc_macro_derive(GetForm, attributes(no_form))]
pub fn derive_get_form(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::get_form::derive_struct(&input);
    impl_trait_tokens.into()
}
