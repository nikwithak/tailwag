use axum::{
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::{Client, Error, StatusCode};
use tailwag_orm::{
    data_definition::exp_data_system::DataSystem,
    data_manager::{
        rest_api::Id,
        traits::{DataProvider, WithFilter},
        PostgresDataProvider,
    },
    queries::filterable_types::FilterEq,
};

use crate::{event::Event, food_truck::FoodTruck};

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    tailwag::macros::GetTableDefinition,
    tailwag::macros::Insertable,
    tailwag::macros::Updateable,
    tailwag::macros::Deleteable,
    tailwag::macros::Filterable,
    tailwag::macros::BuildRoutes,
    tailwag::macros::Id,
    tailwag::macros::AsEguiForm,
    tailwag::macros::Display,
    tailwag::forms::macros::GetForm,
)]
#[actions(temp_webhook)]
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

pub async fn temp_webhook(
    // params: std::collections::HashMap<String, String>
    id: String,
    data_providers: DataSystem,
) -> Option<Vec<Event>> {
    // let (Some(id),) = (
    let (Some(id), Some(breweries), Some(events), Some(food_trucks)) = (
        // params.get("id"),
        Some(id),
        data_providers.get::<Brewery>(),
        data_providers.get::<Event>(),
        data_providers.get::<FoodTruck>(),
    ) else {
        return None;
        // return reqwest::StatusCode::BAD_REQUEST.into_response();
    };

    let brewery = breweries
        .get(|filter| filter.id.eq(id.parse::<uuid::Uuid>().unwrap()))
        .await
        .expect("Oops");
    if let Some(brewery) = brewery {
        // Json(brewery.fetch_events(events, food_trucks).await.unwrap()).into_response()
        brewery.fetch_events(events, food_trucks).await.ok()
    } else {
        None
    }
}

impl Brewery {
    // ETL job: Pulls the web page, exetracts the regex, builds the events (and potentially food trucks), and saves them to the DB.
    async fn fetch_events(
        &self,
        events: PostgresDataProvider<Event>,
        food_trucks: PostgresDataProvider<FoodTruck>,
    ) -> Result<Vec<Event>, reqwest::Error> {
        let (Some(url), Some(regex_str)) = (&self.website_url, &self.food_truck_extraction_regex)
        else {
            return Ok(Vec::new());
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
                // TODO: This will cause issues towards end of year. Need some fancier logic to really streamline this.
                .unwrap_or(now.year());
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
        Ok(Vec::new())
        // event
    }
}
