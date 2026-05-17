use std::{
    sync::{mpsc::Sender, Arc, OnceLock},
    thread::sleep,
    time::Duration,
};

use tailwag_web_service::{
    application::{AdminActions, WebService, WebServiceBuildResponse},
    auth::gateway,
};

mod tailwag {
    pub use tailwag_forms as forms;
    pub use tailwag_macros as macros;
    pub use tailwag_orm as orm;
    pub use tailwag_web_service as web;
}

#[derive(
    Clone,
    Debug,
    Default,
    serde::Deserialize,
    serde::Serialize,
    sqlx::FromRow,
    tailwag_orm_macros::GetTableDefinition,
    tailwag_orm_macros::Insertable,
    tailwag_orm_macros::Updateable,
    tailwag_orm_macros::Deleteable,
    tailwag_orm_macros::Filterable,
    tailwag::macros::BuildRoutes,
    tailwag_orm_macros::Id,
    tailwag::macros::Display,
    tailwag::forms::macros::GetForm,
)]
pub struct Event {
    id: uuid::Uuid,
}

type KillSignalCell = OnceLock<Sender<AdminActions>>;

#[tokio::main(flavor = "current_thread")]
async fn run_service(sender_cell: Arc<KillSignalCell>) {
    let WebServiceBuildResponse {
        service,
        sender,
    } = WebService::builder("Hello World works")
        .get("/login", || "Login form goes here".to_string())
        .post("/login", gateway::login)
        .post("/register", gateway::register)
        .get("/", || "Hello, world!".to_string())
        .with_resource::<Event>()
        .build_service();

    sender_cell.set(sender).unwrap();
    service.run().await.unwrap();
}

macro_rules! test_hurl_file {
    ($filename:literal) => {
        let result = hurl::runner::run(
            include_str!($filename),
            None,
            &hurl::runner::RunnerOptionsBuilder::new().build(),
            // &HashMap::default(),
            &Default::default(),
            // &vec![(
            //     "email_address".to_string(),
            //     hurl::runner::Value::String(format!(
            //         "{}@localhost.local",
            //         // Generates a unique random email address to verify the entire end_to_end flow
            //         uuid::Uuid::new_v4()
            //             .to_string()
            //             .chars()
            //             .filter(|c| c.is_alphanumeric())
            //             .collect::<String>()
            //     )),
            // )]
            // .into_iter()
            // .collect(),
            &hurl::util::logger::LoggerOptionsBuilder::new().build(),
        );
        assert!(result.unwrap().success);
    };
}

#[test]
fn run_hurl_tests() {
    // I did a quick hack-through to add a signal we can use to kill the server gracefully.
    // It's a condition on the while loop listneing for new requests - will only fire when another request
    // is received after sending the kill switch.
    let kill_signal_cell = Arc::new(OnceLock::new());
    let ksc = kill_signal_cell.clone();
    let thread = std::thread::Builder::new()
        .name("Hello World App".to_string())
        .spawn(move || run_service(ksc))
        .unwrap();

    println!("Starting service... waiting 2 seconds for status");
    sleep(Duration::from_secs(1)); // Wait for service to start up. TODO: Give a way to poll the service.
    println!("Checking server status");

    test_hurl_file!("login_register_work.hurl");

    // Tell the server to shut up now
    let signal = kill_signal_cell.get().unwrap();
    signal.send(AdminActions::KillServer).unwrap();
    println!("Sent kill signal to service");

    // The kill signal doesn't fire until another request comes in...
    // definitely a bug but not worth fixing rn, the kill signal was hacked together
    // for these tests anyway, and the replumbing would be a bit of a headache.
    // Smooth killing of service may be needed later though - e.g. I plan to
    // intercept SIGKILL signal so I can cleanly shut down when deploying updates.
    hurl::runner::run(
        r#"GET http://localhost:8081/"#,
        None,
        &hurl::runner::RunnerOptionsBuilder::new().build(),
        &Default::default(),
        &hurl::util::logger::LoggerOptionsBuilder::new().build(),
    )
    .ok();

    thread.join().unwrap();
}
