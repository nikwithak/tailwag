#[cfg(feature = "orm")]
#[cfg(feature = "no-orm")]
panic!("Cannot have both orm & no_orm features enabled");

use syn::parse_macro_input;
mod macro_logic;
use macro_logic::util;

// macro_rules! derive_trait {
//     ($TraitName:ident, $function:item) => {
//         #[proc_macro_derive($TraitName)]
//         pub fn derive_$TraitName(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//             let input = parse_macro_input!(input);
//             let impl_trait_tokens = macro_logic::derive::deref::$function(&input);
//             impl_trait_tokens.into()
//         }
//     };
// }
// TODO: Not working yet
// derive_trait!(Deref, derive_deref);

#[allow(unused)]
macro_rules! derive_struct {
    ($struct_name:ident, $lower_name:ident) => {
        #[proc_macro_derive($struct_name)]
        pub fn derive_$lower_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let input = parse_macro_input!(input);
            let impl_trait_tokens = macro_logic::derive::$lower_name::derive_struct(&input);
            impl_trait_tokens.into()
        }
    };
}

#[proc_macro_derive(FromStr)]
pub fn derive_from_str(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::from_string::derive_trait(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(AsEguiForm)]
pub fn derive_as_egui_form(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::gui::as_egui_form::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(IntoForm)]
pub fn derive_to_form(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::gui::into_form::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::display::derive_display(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(
    BuildRoutes,
    attributes(
        no_default_routes,
        actions,
        views,
        get, // Should I rename this to `list`?
        get_id, // Likewise, should this be `get`? Or `get_detail`? Goal is to reduce confusion as much as possible.
        post,
        patch,
        delete,
        policy,
        get_policy,
        post_policy,
        list_policy,
        delete_policy,
        patch_policy,
        delete_policy
    )
)]
pub fn derive_build_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::build_routes::derive_struct(&input);
    impl_trait_tokens.into()
}

/// Wraps a function with inputs/outputs for a `syn` / `quote`
#[proc_macro_derive(DerefMut, attributes(deref))]
pub fn derive_deref_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::deref::derive_deref_mut(&input);
    impl_trait_tokens.into()
}
/// Wraps a function with inputs/outputs for a `syn` / `quote`
#[proc_macro_derive(Deref, attributes(deref))]
pub fn derive_deref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = macro_logic::derive::deref::derive_deref(&input);
    impl_trait_tokens.into()
}
