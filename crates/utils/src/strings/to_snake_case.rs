pub fn to_snake_case(input: &str) -> String {
    let chars = input.chars();
    let mut output = String::new();
    let mut prev: Option<char> = None;
    for char in chars {
        if char.is_uppercase() && prev.map(|prev_char| prev_char.is_lowercase()).unwrap_or(false) {
            output.push('_');
        }
        if char.is_uppercase() {
            for char in char.to_lowercase() {
                output.push(char);
            }
        } else {
            output.push(char);
        }
        prev.replace(char);
    }
    output
}

pub trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for &str {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }
}
impl ToSnakeCase for String {
    fn to_snake_case(&self) -> String {
        to_snake_case(self)
    }
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn to_camel_case_converts() {
        assert_eq!(to_snake_case("FoodTruck"), "food_truck");
        assert_eq!(to_snake_case("foodTruck"), "food_truck");
        assert_eq!(to_snake_case("FOodTRUck"), "food_truck");
        assert_eq!(to_snake_case("foodTruck"), "food_truck");
        assert_eq!(to_snake_case("foodtruck"), "foodtruck");
        assert_eq!(to_snake_case("_food_truck"), "_food_truck");
        assert_eq!(to_snake_case("FOOD_TRUCK"), "food_truck");
        assert_eq!(to_snake_case("_FOOD_TRUCK"), "_food_truck");
    }
}
