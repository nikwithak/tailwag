use chrono::Duration;
use tailwag_macros::derive_magic;

derive_magic! {
    pub struct Event {
        id: uuid::Uuid,
        #[display]
        pub name: String,
        #[no_filter]
        pub description: Option<String>,
        pub start_time: chrono::NaiveDateTime,
        pub food_truck_id: uuid::Uuid,
        pub brewery_id: uuid::Uuid,
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "New Brewery".to_string(),
            description: None,
            start_time: chrono::Utc::now().naive_utc(),
            ..Default::default()
        }
    }
}
