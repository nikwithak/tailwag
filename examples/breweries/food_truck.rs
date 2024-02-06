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

impl FoodTruck {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

impl Default for FoodTruck {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "New Truck".to_string(),
            description: None,
            website_url: None,
        }
    }
}
