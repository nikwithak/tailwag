use tailwag::prelude::*;

#[tokio::main]
async fn main() {
    derive_magic! {
        #[views(("/filter_todos/{val}", filter_todos))]
        struct Todo {
            id: uuid::Uuid,
            title: String,
            description: String,
            due_date: chrono::NaiveDateTime,
            completed: bool,
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
