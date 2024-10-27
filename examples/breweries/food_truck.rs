use tailwag_macros::derive_magic;

derive_magic! {
    pub struct FoodTruck {
        id: uuid::Uuid,
        #[display]
        pub name: String,
        #[no_filter]
        pub description: Option<String>,
        #[no_filter]
        pub website_url: Option<String>,
    }
}

#[allow(unused)]
impl FoodTruck {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}
