use tailwag_web_service::{
    application::WebServiceBuildResponse,
    auth::gateway::{self, extract_session},
    extras::{
        email_alerts::WithEmailQueueTask,
        image_upload::{self, ImageMetadata},
    },
};
mod product;
pub use product::*;
mod order;
pub use order::*;

pub mod stripe_integration;

#[tokio::main]
async fn main() {
    ShopApplication::new().run().await.unwrap();
}

struct ShopApplication;
impl ShopApplication {
    pub fn new() -> WebServiceBuildResponse {
        tailwag_web_service::application::WebService::builder("My Shop Service")
            .with_middleware(extract_session)
            .post_public("/login", gateway::login)
            .post_public("/logout", gateway::logout)
            .post_public("/register", gateway::register)
            .with_resource::<Product>() // TODO- public GET with filtering)
            .with_resource::<ShopOrder>() // TODO - public POST, restricted GET (to specific customer, via email)
            .with_resource::<OrderAmount>() // TODO - Needed to make sure the tables get created. TODO: Auto-create all direct dependent tables automatically in the ORM
            .with_resource::<ImageMetadata>() // TODO - Needed to make sure the tables get created. TODO: Auto-create all direct dependent tables automatically in the ORM
            .post_public("/checkout", checkout::checkout) // TODO
            .get_public("/image/{filename}", image_upload::load_image)
            .get_public("/", || "Hello, world!".to_string())
            .with_server_data(stripe::Client::new(
                std::env::var("STRIPE_API_KEY").expect("STRIPE_API_KEY is missing from env."), // TODO: Move to a 'config' automation / macro.
            ))
            .with_task(stripe_integration::handle_stripe_event)
            .with_email_queue_task()
            .with_static_files()
            .build_service()
    }
}

#[allow(unused)]
macro_rules! test_hurl_file {
    ($filename:literal) => {
        let result = hurl::runner::run(
            include_str!($filename),
            &hurl::runner::RunnerOptionsBuilder::new().build(),
            // &HashMap::default(),
            &vec![(
                "email_address".to_string(),
                hurl::runner::Value::String(format!(
                    "{}@localhost.local",
                    // Generates a unique random email address to verify the entire end_to_end flow
                    uuid::Uuid::new_v4()
                        .to_string()
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                )),
            )]
            .into_iter()
            .collect(),
            &hurl::util::logger::LoggerOptionsBuilder::new().build(),
        );
        assert!(result.unwrap().success);
    };
}

#[test]
fn run_hurl_tests() {
    type KillSignalCell = OnceLock<Sender<AdminActions>>;
    #[tokio::main(flavor = "current_thread")]
    async fn run_service(sender_cell: Arc<KillSignalCell>) {
        let WebServiceBuildResponse {
            service,
            sender,
        } = ShopApplication::new();

        sender_cell.set(sender).unwrap();
        service.run().await.unwrap();
    }

    let kill_signal_cell = Arc::new(OnceLock::new());
    let ksc = kill_signal_cell.clone();
    let thread = std::thread::Builder::new()
        .name("Shop Application".to_string())
        .spawn(move || run_service(ksc))
        .unwrap();

    println!("Starting service... waiting 2 seconds for status");
    sleep(Duration::from_secs(2)); // Wait for service to start up. TODO: Give a way to poll the service.
    println!("Checking server status");

    test_hurl_file!("tests__hurl/login_register_work.hurl");

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
        &hurl::runner::RunnerOptionsBuilder::new().build(),
        &std::collections::HashMap::default(),
        &hurl::util::logger::LoggerOptionsBuilder::new().build(),
    )
    .ok();

    thread.join().unwrap();
}
