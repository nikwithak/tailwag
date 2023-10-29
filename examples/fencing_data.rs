/// This example is intended to be a "wishlist". Test-Driven development, where I'm building the "what I want" first, and from there I will fill in the blanks to just "Make it work."

mod mocks {}

pub fn main() {
    use super::mocks::*;
    let application = TailwagMagicApplication::new();
    application.register_rest_api::<Brewery>();
    application.register_rest_api::<Brewery>();
}
