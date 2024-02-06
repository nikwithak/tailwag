use axum::http::{HeaderMap, HeaderValue};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::{Client, Error};
use tailwag_macros::derive_magic;
use tailwag_orm::{
    data_manager::{rest_api::Id, traits::DataProvider, PostgresDataProvider},
    queries::filterable_types::FilterEq,
};

use crate::{
    event::Event,
    food_truck::{self, FoodTruck},
};

derive_magic! {
    // #[webhooks(get_truck_events)]
    pub struct Brewery {
        id: uuid::Uuid,
        #[display]
        name: String,
        #[no_filter]
        description: Option<String>,
        #[no_filter]
        website_url: Option<String>,
        #[no_filter]
        food_truck_extraction_regex: Option<String>,
    }
}

impl Default for Brewery {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "New Brewery".to_string(),
            description: None,
            website_url: None,
            food_truck_extraction_regex: None,
        }
    }
}

impl Brewery {
    // ETL job: Pulls the web page, exetracts the regex, builds the events (and potentially food trucks), and saves them to the DB.
    async fn get_truck_events(
        &self,
        events: PostgresDataProvider<Event>,
        food_trucks: PostgresDataProvider<FoodTruck>,
    ) -> Result<Option<Event>, reqwest::Error> {
        let (Some(url), Some(regex_str)) = (&self.website_url, &self.food_truck_extraction_regex)
        else {
            return Ok(None);
        };

        fn client() -> Result<Client, Error> {
            let default_headers = vec![("Cookie", "age_check=1")];
            let default_headers =
                default_headers.into_iter().fold(HeaderMap::new(), |mut map, (h, v)| {
                    map.insert(h, HeaderValue::from_static(v));
                    map
                });
            Client::builder().default_headers(default_headers).build()
        }

        // TODO: START TRANSACTION
        // Build HTTP client
        let client = client()?;
        let response = client.get(url).send().await?.text().await?;
        let regex = regex::Regex::new(regex_str).expect("Regex failed");
        let captures = regex.captures_iter(&response);

        let now = Local::now();
        for capture in captures {
            let month =
                capture.name("month").and_then(|s| s.as_str().parse::<u32>().ok()).unwrap_or(13);
            let day =
                capture.name("day").and_then(|s| s.as_str().parse::<u32>().ok()).unwrap_or(32);
            let year = capture
                .name("year")
                .and_then(|year| year.as_str().parse::<i32>().ok())
                .map(|year| {
                    if year < 100 {
                        year + 2000
                    } else {
                        year
                    }
                })
                // TODO: This could cause issues towards end of year. Need some fancier logic to really streamline this.
                .unwrap_or(now.year());
            // let day_of_week = capture.name("day_of_week").map(|s| s.as_str());
            let mut start_time = capture
                .name("start_time")
                .and_then(|s| s.as_str().parse::<u32>().ok())
                .unwrap_or(25);
            if capture.name("am_or_pm").map(|s| s.as_str().to_lowercase().eq("pm")).is_some() {
                start_time += 12;
            }
            let naive_date = NaiveDate::from_ymd_opt(year, month, day);
            let naive_time = NaiveTime::from_hms_opt(start_time, 0, 0);
            let (Some(naive_date), Some(naive_time)) = (naive_date, naive_time) else {
                log::error!(
                    r#"Failed to extract datetime from capture """{:?}""" using regex """{}""""#,
                    capture,
                    regex
                );
                continue; // TODO: Refactor to avoid continue control flows
            };
            let event_start_time = NaiveDateTime::new(naive_date, naive_time);

            let Some(food_truck_name) = capture.name("food_truck_name").map(|s| s.as_str()) else {
                continue; // TODO: Refactor to avoid continue control flows
            };
            let truck = match food_trucks
                .all()
                .await // TODO: Shouldn't need this .await.unwrap()
                .unwrap()
                .with_filter(|c| c.name.eq(food_truck_name))
                .execute()
                .await
                .unwrap() // TODO: Set up #thiserror so that ? can handle this
                .pop() // TODO: Use .get() instead of .all().with_filter()....pop()
            {
                Some(truck) => {
                    println!("Found food truck: {:?}", &truck);
                    truck
                },
                None => {
                    let truck = food_trucks.create(FoodTruck::new(food_truck_name)).await.unwrap();
                    log::info!("Created new food truck: {}", truck);
                    truck
                }
            };

            if events
                // TODO: all() -> with_filter (directly)
                // TODO: get() -> filter and assert there's only one
                .all()
                .await
                .unwrap()
                .with_filter(|e| {
                    e.start_time.eq(event_start_time)
                        & e.food_truck_id.eq(*truck.id())
                        & e.brewery_id.eq(*self.id())
                })
                .execute()
                .await
                .unwrap()
                .is_empty()
            {
                let mut new_event = Event::default();
                new_event.name = format!("{} at {}", truck.name, self.name);
                new_event.food_truck_id = *truck.id();
                new_event.brewery_id = *self.id();
                new_event.start_time = event_start_time;
                let event = events.create(new_event).await.unwrap();
                log::info!("Created new event: {}", event);
            }
        }
        Ok(todo!())
    }
}
