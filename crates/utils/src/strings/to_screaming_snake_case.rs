use std::fmt::Display;

// Converts an input to SCREAMING_SNAKE_CASE. Does not filter out any special characters.
pub fn to_screaming_snake_case(input: &str) -> String {
    let chars = input.chars();
    let mut output = String::new();
    let mut prev: Option<char> = None;
    for char in chars {
        if char.is_uppercase() && prev.map(|prev_char| prev_char.is_lowercase()).unwrap_or(false) {
            output.push('_');
        }
        if char.is_lowercase() {
            for char in char.to_uppercase() {
                output.push(char);
            }
        } else {
            output.push(char);
        }
        prev.replace(char);
    }
    output
}

pub trait ToScreamingSnakeCase {
    fn to_screaming_snake_case(&self) -> String;
}

impl<T: Display> ToScreamingSnakeCase for T {
    fn to_screaming_snake_case(&self) -> String {
        to_screaming_snake_case(&self.to_string())
    }
}

// impl ToScreamingSnakeCase for &str {
//     fn to_screaming_snake_case(&self) -> String {
//         to_screaming_snake_case(self)
//     }
// }

// impl ToScreamingSnakeCase for String {
//     fn to_screaming_snake_case(&self) -> String {
//         to_screaming_snake_case(self)
//     }
// }

#[cfg(test)]
mod tests {
    use super::to_screaming_snake_case;
    use crate::strings::to_screaming_snake_case::ToScreamingSnakeCase;

    macro_rules! test_case {
        ($input:expr, $expected:expr) => {
            assert_eq!(to_screaming_snake_case($input), $expected);
            assert_eq!($input.to_screaming_snake_case(), $expected);
        };
    }
    #[test]
    fn to_camel_case_converts() {
        assert_eq!("foodTruck".to_screaming_snake_case(), "FOOD_TRUCK");
        test_case!("FoodTruck", "FOOD_TRUCK");
        test_case!("foodTruck", "FOOD_TRUCK");
        test_case!("FOodTRUck", "FOOD_TRUCK");
        test_case!("foodTruck", "FOOD_TRUCK");
        test_case!("FOOD_TRUCK", "FOOD_TRUCK");
        test_case!("foodtruck", "FOODTRUCK");
        test_case!("_food_truck", "_FOOD_TRUCK");
        test_case!("_FOOD_TRUCK", "_FOOD_TRUCK");
    }
}
