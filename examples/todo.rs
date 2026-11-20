use tailwag::prelude::*;

#[tokio::main]
async fn main() {
    derive_magic! {
        #[views(("/filter_todos/{val}", filter_todos))]
        struct Todo {
            id: uuid::Uuid,
            title: String,
            description: String,
            due_date: time::PrimitiveDateTime,
            completed: bool,
        }
    }

    // `time::PrimitiveDateTime` has no `Default` impl, so derive the default manually.
    impl Default for Todo {
        fn default() -> Self {
            Self {
                id: uuid::Uuid::default(),
                title: String::default(),
                description: String::default(),
                due_date: tailwag::orm::OrDefault::or_default(),
                completed: false,
            }
        }
    }

    WebService::builder("Todo Service")
        .with_resource::<Todo>()
        .build_service()
        .run()
        .await
        .expect("Web service crashed.");

    pub async fn filter_todos(
        val: PathString,
        todos: PostgresDataProvider<Todo>,
    ) -> Vec<Todo> {
        let val = &*val;
        let query = todos
            .with_filter(|todo| todo.completed.eq(true) & todo.title.like(&format!("%{val}%")));
        query.execute().await.unwrap()
    }
}
