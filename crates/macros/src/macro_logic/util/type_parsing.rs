use syn::{Field, GenericArgument, PathArguments, Type, TypePath};

pub trait GetQualifiedPath {
    fn get_qualified_path(&self) -> String;
    fn get_qualified_path_for_option(&self) -> String;
}

pub trait IsOption {
    fn is_option(&self) -> bool;
}
impl IsOption for syn::Field {
    fn is_option(&self) -> bool {
        is_option(self)
    }
}

impl GetQualifiedPath for syn::Field {
    fn get_qualified_path(&self) -> String {
        get_type_str(&self.ty)
    }
    fn get_qualified_path_for_option(&self) -> String {
        extract_option_type(self)
    }
}

pub fn get_type_str(ty: &Type) -> String {
    match ty {
        syn::Type::Path(typepath) => get_qualified_path(typepath),
        _ => unimplemented!("{:?}", ty),
    }
}

pub fn get_qualified_path(typepath: &TypePath) -> String {
    let qualified_path = typepath.path.segments.iter().fold(String::new(), |mut acc, p| {
        acc.push_str(&p.ident.to_string());
        acc.push_str("::");
        acc
    });
    qualified_path.trim_end_matches("::").to_string()
}

pub fn is_option(field: &Field) -> bool {
    if let syn::Type::Path(typepath) = &field.ty {
        matches!(
            get_qualified_path(typepath).as_str(),
            "std::option::Option" | "core::option::Option" | "option::Option" | "Option"
        )
    } else {
        false
    }
}

/// Gives you the primary type of the field. If it's an Option, then this will return the qualified path string for the Option's inner type.
/// If not, it returns the qualified path string for the entire type.
pub fn extract_option_type(field: &Field) -> String {
    let syn::Type::Path(typepath) = &field.ty else {
        panic!("No typepath found")
    };
    let qualified_path = field.get_qualified_path();
    match qualified_path.as_str() {
        "std::option::Option" | "core::option::Option" | "option::Option" | "Option" => {
            let type_params = &typepath
                .path
                .segments
                .last()
                .expect("Option should have an inner type")
                .arguments;
            match &type_params {
                PathArguments::AngleBracketed(params) => {
                    let arg = params.args.first().expect("No type T found for Option<T>");
                    match arg {
                        GenericArgument::Type(syn::Type::Path(t)) => Some(get_qualified_path(t)),
                        _ => panic!("no type T found for Option<T>"),
                    }
                },
                _ => panic!("No type T found for Option<T>"),
            }
        },
        _ => None,
    }
    .unwrap_or(qualified_path)
}

// pub fn get_type_from_field(field: &Field) -> DatabaseColumnType {
//     match &field.ty {
//         syn::Type::Path(typepath) => {
//             // Match the type - if it's a supported type, we map it to the DatabaseColumnType. If it's not, we either fail (MVP), or we add support for joins via another trait (must impl DatabaseColumnSubType or something).
//             // TODO: DRY this out using the `is_option` fn above
//             let mut qualified_path = get_qualified_path(typepath);
//             qualified_path = match qualified_path.as_str() {
//                 "std::option::Option" | "core::option::Option" | "option::Option" | "Option" => {
//                     let type_params = &typepath
//                         .path
//                         .segments
//                         .last()
//                         .expect("Option should have an inner type")
//                         .arguments;
//                     match &type_params {
//                         PathArguments::AngleBracketed(params) => {
//                             let arg = params.args.first().expect("No type T found for Option<T>");
//                             match arg {
//                                 GenericArgument::Type(syn::Type::Path(t)) => {
//                                     Some(get_qualified_path(t))
//                                 },
//                                 _ => panic!("no type T found for Option<T>"),
//                             }
//                         },
//                         _ => panic!("No type T found for Option<T>"),
//                     }
//                 },
//                 _ => None,
//             }
//             .unwrap_or(qualified_path);

//             let db_type = match qualified_path.as_str() {
//                 "std::string::String" | "string::String" | "String" => DatabaseColumnType::String,
//                 "bool" => DatabaseColumnType::Boolean,
//                 "u32" | "u64" | "i32" | "i64" | "usize" | "isize" => DatabaseColumnType::Int,
//                 "f32" | "f64" | "fsize" => DatabaseColumnType::Float,
//                 "chrono::_" => DatabaseColumnType::Timestamp, // TODO
//                 "uuid::Uuid" | "Uuid" => DatabaseColumnType::Uuid,
//                 _ => {
//                     // TODO: Impl for joinable tables
//                     unimplemented!("{} not a supported type.", qualified_path)
//                 },
//             };
//             db_type
//         },
//         _ => {
//             unimplemented!("Not a supported data type")
//         },
//     }
// }
