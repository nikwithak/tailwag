use sqlx::postgres::PgPoolOptions;
use tailwag::{
    orm::{
        data_definition::database_definition::DatabaseDefinition,
        data_manager::{GetTableDefinition, PostgresDataProvider},
    },
    web::{application::WebServiceApplication, traits::rest_api::BuildRoutes},
};

#[derive(
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    Clone,
    tailwag::macros::Queryable,
    tailwag::macros::GetTableDefinition,
    tailwag::macros::Insertable,
    tailwag::macros::BuildRoutes,
)]
pub struct Item {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
}

#[tokio::main]
async fn main() {
    // TODO: Goal to make functions / logic easy to override
    let data_model: DatabaseDefinition = DatabaseDefinition::new_unchecked("postgres")
        .table(Item::get_table_definition())
        .into();

    /////////////////////////////////////
    // Boilerplate for DB connectivity //
    /////////////////////////////////////
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .await
        .expect("[DATABASE] Unable to obtain connection to database");

    let provider = PostgresDataProvider {
        table_definition: Item::get_table_definition(),
        db_pool: pool,
        _t: Default::default(),
    };

    let app: WebServiceApplication =
        WebServiceApplication::default().add_routes("/item", Item::build_routes(provider).await);

    app.run().await;
    todo!()

    // DataModelRestServiceDefinition::default()
    // .with_resource::<Item>("/item")
    // .run_app();
}

// #[async_trait]
// impl BuildRoutes<Self> for Item {
//     type Resource = Self;

//     async fn build_routes(data_manager: PostgresDataProvider<Self>) -> Router {
//         let _ = data_manager;
//         #[derive(Deserialize)]
//         pub struct ItemCreateRequest {
//             name: String,
//             description: Option<String>,
//         }
//         impl Into<Item> for ItemCreateRequest {
//             fn into(self) -> Item {
//                 Item {
//                     id: Uuid::new_v4(),
//                     name: self.name,
//                     description: self.description,
//                 }
//             }
//         }

//         // TODO: macro for the function to wrap in `Query` or `Json` automatically, so it can just be a decoration on a logic function.
//         pub async fn post_item(
//             State(data_manager): State<PostgresDataProvider<Item>>,
//             axum::extract::Json(request): axum::extract::Json<ItemCreateRequest>,
//         ) -> Json<Item> {
//             let item: Item = request.into();
//             data_manager.create(&item).await.expect("Unable to create object");
//             Json(item)
//         }

//         pub async fn get_items(
//             State(data_manager): State<PostgresDataProvider<Item>>
//         ) -> Json<Vec<Item>> {
//             Json(data_manager.all().execute().await.unwrap())
//         }

//         data_manager.run_migrations().await.expect("Failed to run migrations");
// Router::new()
// .route("/", post(post_item))
// .route("/", get(get_items))
//             .with_state(data_manager)
//     }
// }

// // TODO: macro for the function to wrap in `Query` or `Json` automatically, so it can just be a macro over a logic function.
// pub async fn get_food_trucks(
//     State(data_manager): State<FoodTruckDataManager>,
//     // Query(request): Query<GetFoodTrucksRequest>,
// ) -> Json<Vec<FoodTruck>> {
//     let q = data_manager.all();
//     let results = q.execute().await.unwrap();
//     Json(results)
// // }

// pub async fn form() -> Html<String> {
//     Html(
//         "
//         <form method=\"POST\" encType=\"application/json\" >
//             <label for=\"name\">Name</label>
//             <input type=\"text\" name=\"name\" />
//             <br />
//             <label for=\"style\">Style</label>
//             <input type=\"text\" name=\"style\" />
//             <input type=\"checkbox\" name=\"is_open_late\" value=\"true\" />
//             <br />
//             <button formaction=\"/food_truck\" type=\"submit\">Submit</button>
//         </form>
//         "
//         .into(),
//     )
// }
//         todo!()
//     }
