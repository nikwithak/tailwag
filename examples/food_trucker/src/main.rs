use sqlx::postgres::PgPoolOptions;
use tailwag::{
    orm::data_manager::{GetTableDefinition, PostgresDataProvider},
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
    tailwag::macros::Updateable,
    tailwag::macros::Deleteable,
    tailwag::macros::BuildRoutes,
)]
pub struct Brewery {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
    website_url: Option<String>,
    food_truck_extraction_regex: Option<String>,
    // location: Geometry,
}

#[tokio::main]
async fn run_server() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .await
        .expect("[DATABASE] Unable to obtain connection to database");

    let provider = PostgresDataProvider {
        table_definition: Brewery::get_table_definition(),
        db_pool: pool,
        _t: Default::default(),
    };

    let app: WebServiceApplication =
        WebServiceApplication::default().add_routes("/item", Brewery::build_routes(provider).await);

    app.run().await;

    // DataModelRestServiceDefinition::default()
    // .with_resource::<Item>("/item")
    // .run_app();
}

fn main() {
    run_server()
}

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
