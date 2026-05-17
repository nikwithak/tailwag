use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};
use tailwag_utils::macro_utils::attribute_parsing::GetAttribute;

/// Logic for deriving Deref - only tested with structs using named fields.
///
/// Derefs to the field tagged with the `#[deref]` attribute  - if not found, it defaults to the first declared field in the struct.
///
/// Behavior:
///  - For Enums, it implements the name of the attribute as the Display string. Does not work on enums with data inside, only simplee enums
///  - For Structs, it will either echo the Debug output of Self (must also `impl Debug`), OR
///     - tag a field with `#[display]` and it will proxy that Display.
pub fn derive_display(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    match data {
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| {
                if v.fields.len() > 0 {
                    panic!("This macro currently only supports no-field Enums")
                } else {
                    &v.ident
                }
            });
            let variants_str = variants.clone().map(|v| v.to_string());

            // If it's a fieldless struct, we simply deref the name of the struct.
            // let (target_ident, target_type) = target.unwrap_or((quote!(#ident_string), quote!(str)));

            // TODO: Time for generics!
            let tokens = quote!(
                impl ::std::fmt::Display for #ident {
                    fn fmt(
                        &self,
                        f: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        match self {
                            #(Self::#variants => f.write_str(#variants_str),)*
                        }
                    }
                }
            );
            tokens.into()
        },
        Data::Struct(data) => {
            let write_statement = data
                .fields
                .iter()
                .find(|f| f.get_attribute("display").is_some())
                .and_then(|f| f.ident.as_ref())
                .map(|f| quote!(write!(f, "{}", &self.#f)))
                .unwrap_or(quote!(write!(f, "{:?}", &self)));
            quote!(
            // TODO: Derive macro this
            impl ::std::fmt::Display for #ident {
                fn fmt(
                    &self,
                    f: &mut std::fmt::Formatter<'_>,
                ) -> std::fmt::Result {
                    #write_statement
                }
            }
                        )
        },
        _ => unimplemented!(),
    }
}
