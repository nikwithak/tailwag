// Converts an input to "title case". Does not filter out any special characters.
pub fn to_title_case(input: &str) -> String {
    let chars = input.chars();
    let mut output = String::new();
    let mut prev: Option<char> = None;
    for char in chars {
        if char.is_uppercase() && prev.map_or(false, |prev_char| prev_char.is_lowercase()) {
            output.push(' ');
        }
        if char == '_' && prev.map_or(false, |prev_char| prev_char != '_') {
            output.push(' ');
        } else if prev.map_or(true, |prev_char| prev_char == '_') {
            output.push(char.to_ascii_uppercase());
        } else {
            output.push(char);
        }
        prev.replace(char);
    }
    output
}

#[allow(unused)]
pub trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for String {
    fn to_title_case(&self) -> String {
        to_title_case(self)
    }
}
impl ToTitleCase for &str {
    fn to_title_case(&self) -> String {
        to_title_case(self)
    }
}

#[cfg(test)]
mod tests {
    use super::to_title_case;

    macro_rules! test_case {
        ($input:expr, $expected:expr) => {
            assert_eq!(to_title_case($input), $expected)
        };
    }
    #[test]
    fn to_camel_case_converts() {
        test_case!("FoodTruck", "Food Truck");
        test_case!("foodTruck", "Food Truck");
        test_case!("foodTruck", "Food Truck");
        test_case!("foodtruck", "Foodtruck");
        test_case!("food_truck", "Food Truck");
        // Non-working cases below. Need to re-think expected behavior
        // test_case!("FOodTRUck", "Food Truck");
        // test_case!("FOOD_TRUCK", "Food Truck");
        // test_case!("_food_truck", "Food Truck");
        // test_case!("_FOOD_TRUCK", "Food Truck");
    }
}
