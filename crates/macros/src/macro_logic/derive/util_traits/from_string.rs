use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

/// Logic for deriving Deref - only tested with structs using named fields.
///
/// Derefs to the field tagged with the `#[deref]` attribute  - if not found, it defaults to the first declared field in the struct.
pub fn derive_trait(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    let Data::Enum(data) = data else {
        panic!("this macro currently only supports no-field Enums")
    };

    let variants = data.variants.iter().map(|v| {
        if v.fields.len() > 0 {
            panic!("This macro currently only supports no-field Enums")
        } else {
            &v.ident
        }
    });
    let variants_str = variants.clone().map(|v| v.to_string());
    let err_msg = format!("{{}} is not a valid {}", &ident);

    let tokens = quote!(
        impl ::std::str::FromStr for #ident {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#variants_str => Ok(Self::#variants),)*
                    _ => Err(format!(#err_msg, s))
                }
            }
        }
    );
    tokens.into()
}
