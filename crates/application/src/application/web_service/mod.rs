use std::collections::HashSet;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{collections::HashMap, future::Future, net::TcpListener, pin::Pin};

use crate::application::http::into_route_handler::IntoRouteHandler;
use crate::auth::gateway::{self, extract_session, AppUserCreateRequest, JwtSecret, Session};
use crate::tasks::runner::{IntoTaskHandler, Signal, TaskExecutor};
use env_logger::Env;
use log;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tailwag_forms::{Form, GetForm};
use tailwag_macros::Deref;
use tailwag_orm::data_manager::rest_api::Id;
use tailwag_orm::data_manager::traits::DataProvider;
use tailwag_orm::queries::filterable_types::Filterable;
use tailwag_orm::{
    data_definition::{
        data_system::{DataSystem, DataSystemBuilder, UnconnectedDataSystem},
        table::Identifier,
    },
    data_manager::GetTableDefinition,
    queries::{Deleteable, Updateable},
};
use tailwag_utils::time_exec;
use tailwag_utils::types::generic_type_map::TypeInstanceMap;

use crate::application::http::route::{IntoResponse, RequestContext, ServerContext};
use crate::{auth::gateway::AppUser, traits::rest_api::BuildRoutes};

use super::http::route::{Request, Response};
use super::middleware::cors;
use super::static_files::load_static;
use super::{http::route::Route, stats::RunResult};

#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error("Something went wrong.")]
    Error,
    #[error("Server Resource Not Found")]
    ServerResourceNotFound,
}

pub type Middleware = dyn Send
    + Sync
    + Fn(
        Request,
        RequestContext,
        // fn(Request, RequestContext) -> Pin<Box<dyn Future<Output = Response>>>, // Box<NextFn>, // The function to call when computation is complete
        Arc<NextFn>,
    ) -> Pin<Box<dyn Send + Future<Output = Response>>>;
// ttype NextFn = dyn 'static + n(Request, RequestContext) -> Pin<Box<dyn Future<Output = Response>>>;
pub type NextFn =
    dyn Send + Sync + Fn(Request, RequestContext) -> Pin<Box<dyn Future<Output = Response> + Send>>;

// TODO: Separate definition from config
#[derive(Debug)]
#[allow(unused)]
pub struct WebServiceConfig {
    application_name: String,
    socket_addr: String,
    _max_threads: usize,
    request_timeout_seconds: u64,
    port: i32,
    migrate_on_init: bool,
    database_conn_string: String,
    pub(crate) cors_allowed_origins: HashSet<String>,
}
// What if I do something like
// ```rust
// state: AppState<> and anything wrapped in an Arc<T: Clone> is FromAppState<>
// ````
pub enum AdminActions {
    KillServer,
}

type HandlerFn =
    dyn Send + Sync + Fn(Request, RequestContext) -> Pin<Box<dyn Send + Future<Output = Response>>>;
#[allow(private_bounds)]
#[derive(Clone)]
pub struct WebServiceInner {
    config: Arc<WebServiceConfig>,
    consolidated_handler: Arc<HandlerFn>, // TODO / IDEA: Maybe make this just a "RequestHandler" fn, instead of "handle request"?
    // routes: Arc<Route>,
    resources: UnconnectedDataSystem,
    server_data: Arc<TypeInstanceMap>,
}

#[derive(tailwag_macros::Deref)]
pub struct WebService {
    #[deref]
    inner: WebServiceInner,
    task_executor: Option<TaskExecutor>,
    admin_rx: Receiver<AdminActions>,
}

// TODO: Separate definition from config
#[allow(private_bounds)]
pub struct WebServiceBuilder {
    config: WebServiceConfig,
    root_route: Route,
    forms: HashMap<Identifier, Form>,
    _exp_middleware: Vec<Arc<Middleware>>,
    resources: DataSystemBuilder,
    server_data: TypeInstanceMap,
    task_executor: TaskExecutor,
}

// #[cfg(debug_assertions)]
impl Default for WebServiceBuilder {
    /// Initializes a web service. The service configuration is pulled from Environment variables, with sensible defaults for *debug development*
    /// for anything not specified.
    ///
    /// The default Tailwag Application includes the authentication module, and the CORS module.
    fn default() -> Self {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
        // Load in the current `.env` file, if it exists. If it fails, who cares, the rest of the ENV should be set.
        dotenv::dotenv().ok();
        let database_conn_string = match std::env::var("DATABASE_CONN_STRING") {
            Ok(s) => s,
            Err(_) => {
                let db_name = std::env::var("POSTGRES_DB").unwrap_or("postgres".into());
                let user = std::env::var("POSTGRES_USER").unwrap_or("postgres".into());
                let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("postgres".into());
                let endpoint = std::env::var("POSTGRES_ENDPOINT").unwrap_or("127.0.0.1".into());
                let port = std::env::var("POSTGRES_PORT").unwrap_or("5432".into());
                format!("postgres://{user}:{password}@{endpoint}:{port}/{db_name}")
            },
        };
        let socket_addr = std::env::var("LISTEN_ADDRESS").unwrap_or("127.0.0.1".into()); // Only listens on localhost, unless configured otherwise.
        let port = std::env::var("LISTEN_PORT").map_or(8081, |port| {
            port.parse().expect("Invalid port provided - must be an integer.")
        });
        let max_threads = std::env::var("MAX_THREADS").map_or(4, |num_threads| {
            num_threads
                .parse()
                .expect("Invalid thread count provided - must be an integer.")
        });
        let application_name =
            std::env::var("APPLICATION_NAME").unwrap_or("Tailwag Default Application".into());
        let migrate_on_init = std::env::var("MIGRATE_ON_INIT").map_or(true, |val| {
            val.parse().expect("MIGRATE_ON_INIT must be parseable to a boolean")
        });
        let request_timeout_seconds = std::env::var("REQUEST_TIMEOUT_MS") // Defaults to 30 seconds
            .map_or(30000, |val| {
                val.parse().expect("REQUEST_TIMEOUT_SECONDS must be a valid integer.")
            });

        // TODO: Handle these.
        let _allowed_domains: HashSet<String> = std::env::var("ALLOWED_DOMAINS")
            .unwrap_or_default()
            .split(",")
            .map(|s| s.trim().to_string())
            .filter(|s| s.is_empty())
            .collect();

        let cors_allowed_origins: HashSet<String> = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or("".into())
            .split(",")
            .map(String::from)
            .collect();

        Self {
            config: WebServiceConfig {
                socket_addr,
                port,
                _max_threads: max_threads,
                application_name,
                migrate_on_init,
                database_conn_string,
                request_timeout_seconds,
                cors_allowed_origins,
            },
            resources: DataSystem::builder(),
            root_route: Route::default(),
            forms: HashMap::new(),
            server_data: Default::default(),
            task_executor: Default::default(),
            _exp_middleware: Vec::new(),
        }
        // .with_authentication()
        .with_cors()
    }
}

macro_rules! build_route_method {
    ($method:ident:$variant:ident) => {
        pub fn $method<F, I, O>(
            mut self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
        ) -> Self {
            self.root_route = self.root_route.$method(path, handler);
            self
        }
    };
    ($method:ident:$variant:ident) => {
        pub fn $method<F, I, O>(
            mut self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
        ) -> Self {
            self.root_route = self.root_route.$method(path, handler);
            self
        }
    };
}

/// Builder methods for easy route building. These allow you to chain together .get("/path", || ()) calls,
/// quickly building a service.
impl WebServiceBuilder {
    build_route_method!(get:Get);
    build_route_method!(post:Post);
    build_route_method!(delete:Delete);
    build_route_method!(patch:Patch);
    build_route_method!(get_public:Get);
    build_route_method!(post_public:Post);
    build_route_method!(delete_public:Delete);
    build_route_method!(patch_public:Patch);
}

impl WebServiceBuilder {
    // Builder functions
    pub fn new(app_name: &str) -> Self {
        let mut builder = Self::default();
        builder.config.application_name = app_name.to_string();
        builder
    }

    // Adds an endpoint at `/static/{}`, which will serve the static content of all files in the `static` directory.
    pub fn with_static_files(self) -> Self {
        // TODO: Move this to its own module
        self.get("/static/{path}", load_static)
    }

    // Adds the CRUD endpoints for the specified type, `T`. The routes that get created are determined by `T`'s implementation of the trait  `BuidlRoutes`.
    pub fn with_resource<T>(mut self) -> Self
    where
        // Gross collection of required traits. Need to clean this up.
        T: GetTableDefinition
            + BuildRoutes<T>
            + tailwag_orm::queries::Insertable
            + 'static
            + Send
            + Id
            + Filterable
            + Clone
            + std::fmt::Debug
            + Unpin
            + Updateable
            + Default
            + GetForm
            + for<'a> Deserialize<'a>
            + Serialize
            // + for<'r> sqlx::FromRow<'r, PgRow>
            + Deleteable,
    {
        let resource_name = T::get_table_definition().table_name.clone();
        self.resources.add_resource::<T>();
        self.forms.insert(resource_name.clone(), T::get_form());

        {
            //  BUILD THE ROUTES
            let route = T::build_routes();
            self.root_route.route(format!("{}", &resource_name), route);
        }

        {
            //  EXPORT THE FORMS
            let form = T::get_form();
            form.save_json(&format!("out/forms/{resource_name}.json")).unwrap();
        }

        self
    }

    pub fn with_task<F, T, Req>(
        mut self,
        task_handler: F,
    ) -> Self
    where
        F: IntoTaskHandler<F, T, Req> + Sized + Send + 'static,
        Req: 'static,
    {
        self.task_executor.add_handler(task_handler);
        self
    }

    pub fn with_middleware(
        mut self,
        func: impl 'static
            + Send
            + Sync
            + Fn(
                Request,
                RequestContext,
                // The function to call when computation is complete
                Arc<NextFn>, // fn(Request, RequestContext) -> Pin<Box<dyn Future<Output = Response>>>, // Box<NextFn>
            ) -> Pin<Box<dyn Send + Future<Output = Response>>>,
    ) -> Self {
        self._exp_middleware.push(Arc::new(func));
        self
    }

    pub fn with_server_data<T: Clone + Send + Sync + 'static>(
        mut self,
        data: T,
    ) -> Self {
        self.server_data.insert(data);
        self
    }

    pub fn build_service(self) -> WebServiceBuildResponse {
        let (admin_tx, admin_rx) = channel();
        // let WebServiceBuilder { config, root_route, migrations, forms, middleware_before, middleware_after, resources, server_data, task_executor } = self;
        let mut server_data = self.server_data;
        let config = Arc::new(self.config);
        server_data.insert(self.task_executor.scheduler());
        server_data.insert(config.clone());

        fn build_middleware(
            routes: Route,
            middleware: Vec<Arc<Middleware>>,
        ) -> Arc<HandlerFn> {
            let routes = Arc::new(routes);
            let mut consolidated_fn: Arc<HandlerFn> =
                // Just calls the request. This is our end state.
                Arc::new(move |req: Request, ctx: RequestContext| {
                    // Box::pin(async move { orig_req(req, ctx).await })
                    let routes = routes.clone();
                    Box::pin(async move { routes.clone().handle(req, ctx).await })
                });

            for mw_step in middleware.into_iter().rev() {
                // Wrap each middleware function with the one before it. This allows for a "bounce" in the middleware - requests will go top-down, so the first thing it hits is the first middleware added.
                consolidated_fn = Arc::new(move |req: Request, ctx: RequestContext| {
                    let next = consolidated_fn.clone();
                    mw_step(
                        req,
                        ctx,
                        Arc::new(move |req: Request, ctx: RequestContext| next(req, ctx)),
                    )
                });
            }
            consolidated_fn
        }

        // Print all the configured routes before building.
        // TODO: Move this to on start.
        self.root_route.print_all_routes();

        let service = WebService {
            inner: WebServiceInner {
                config,
                resources: self.resources.build().unwrap(),
                // routes: Arc::new(self.root_route), // No longer stored in Webservice - it's now moved to Middleware when running.
                server_data: Arc::new(server_data),
                consolidated_handler: build_middleware(self.root_route, self._exp_middleware),
            },
            admin_rx,
            task_executor: Some(self.task_executor),
        };

        WebServiceBuildResponse {
            service,
            sender: admin_tx,
        }
    }
}

impl WebServiceBuilder {
    pub fn with_authentication(self) -> Self {
        self.with_middleware(extract_session)
            .with_resource::<AppUser>()
            .with_resource::<Session>()
            .with_server_data(JwtSecret::init())
            .post_public("/login", gateway::login)
            .post_public("/register", gateway::register)
    }

    pub fn with_cors(self) -> Self {
        self.with_middleware(cors::handle_cors)
    }
}

#[derive(Deref)]
pub struct WebServiceBuildResponse {
    #[deref]
    pub service: WebService,
    pub sender: Sender<AdminActions>,
}

impl WebServiceBuildResponse {
    pub async fn run(self) -> Result<RunResult, crate::Error> {
        self.service.run().await
    }
}

type RequestMetrics = ();
impl WebService {
    fn print_welcome_message(&self) {
        let WebServiceConfig {
            application_name,
            ..
        } = &*self.config;
        // TODO: Bring this into a template file (.txt or .md)
        log::info!(
            r#"
=============================================
   Starting Web Application {application_name}
=============================================
"#,
        );
        log::debug!("CONFIGURED ENVIRONMENT: {:?}", &self.config);
        #[cfg(debug_assertions)]
        {
            log::warn!(
                r#"
============================================================================="
    ++ {application_name} IS RUNNING IN DEVELOPMENT MODE ++
    ++      DO NOT USE IN PRODUCTION WITH THIS CONFIG ++
============================================================================="
"#
            );
        }
    }

    pub fn builder(name: &str) -> WebServiceBuilder {
        WebServiceBuilder::new(name)
    }

    async fn build_context(
        &self,
        db_pool: &PgPool,
    ) -> ServerContext {
        let data_providers = self.resources.connect(db_pool.clone()).await;
        let server_data = self.server_data.clone();

        ServerContext {
            data_providers,
            server_data,
        }
    }

    async fn connect_postgres(&self) -> Result<PgPool, crate::Error> {
        Ok(PgPoolOptions::new()
            .max_connections(4)
            .connect(&self.config.database_conn_string)
            .await?)
    }

    async fn start_service(
        self,
        context: ServerContext,
    ) -> Result<RunResult, crate::Error> {
        let bind_addr = format!("{}:{}", &self.config.socket_addr, self.config.port);
        log::info!("Starting service on {}", &bind_addr);
        let listener = TcpListener::bind(&bind_addr).unwrap();
        log::info!("Waiting for connection....");
        while let Ok((stream, _addr)) = listener.accept() {
            if let Ok(AdminActions::KillServer) = self.admin_rx.try_recv() {
                // If we've gotten a kill signal, then stop the server.
                break;
            }

            log::info!("Received connection from {}!", _addr.ip());
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(
                self.config.request_timeout_seconds,
            )));
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(
                self.config.request_timeout_seconds,
            )));
            // TODO: Rate-limiting & failtoban to block malicious actors

            let inner = self.inner.clone();
            tokio::spawn(inner.handle_request(stream, context.clone()));

            log::info!("Waiting for connection....");
        }
        Ok(RunResult::default())
    }

    fn start_task_executor(
        &mut self,
        context: ServerContext,
    ) -> Option<JoinHandle<()>> {
        self.task_executor.take().map(|exec| std::thread::spawn(|| exec.run(context)))
    }

    pub async fn run(mut self) -> Result<RunResult, crate::Error> {
        self.print_welcome_message();

        let db_pool = self.connect_postgres().await?;
        let context = self.build_context(&db_pool).await;

        context.data_providers.run_migrations().await?;
        // Create root user, if none exits & one is configured.
        if let Some(users) = context.data_providers.get::<AppUser>() {
            if let None = users.all().await?.next() {
                // No users exist. Create a default user.
                let email_address =
                    std::env::var("CREATE_ADMIN_USER_EMAIL").unwrap_or("root@localhost".into());
                log::warn!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                log::warn!("!!! CREATING ADMIN USER WITH EMAIL {}", &email_address);
                let password = std::env::var("CREATE_ADMIN_USER_PASSWORD").unwrap_or_else(|_| {
                    let mut rng = rand::thread_rng();
                    let password = (0..24).map(|_| rng.sample(&Alphanumeric) as char).collect();
                    log::warn!("!!! CREATING ADMIN USER WITH PASSWORD {}", &password);
                    log::warn!("!!! CHANGE THIS PASSWORD !!!");
                    log::warn!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                    password
                });
                users
                    .create(AppUserCreateRequest {
                        email_address,
                        password,
                        is_admin: true,
                    })
                    .await?;
            }
        }

        let mut task_scheduler = self
            .task_executor
            .as_ref()
            .map(|te| te.scheduler())
            .ok_or("Unable to get task scheduler.".to_string())?;
        let tasks_thread = self.start_task_executor(context.clone());
        let result = self.start_service(context.clone()).await;

        // Let the tasks_thread die
        task_scheduler
            .enqueue(Signal::Kill)
            .map_err(|err| format!("Unable to schedule task: {:?}", err))?;
        tasks_thread.map(|thread| thread.join());
        result
    }
}

/// This mod adds QueuedTask support to the WebApplication, running in a separate thread.
/// #[cfg(feature = "tasks")]
impl WebServiceInner {
    pub async fn handle_request(
        self,
        mut stream: std::net::TcpStream,
        server_context: ServerContext,
    ) -> Result<RequestMetrics, crate::Error> {
        let request_id = uuid::Uuid::new_v4();
        log::info!("[REQ_ID {request_id}] Connection received from {}", stream.peer_addr()?);

        let request = time_exec!(
            "[REQ_ID {request_id}] Request Destructuring",
            crate::application::http::route::Request::try_from(&stream)
        );
        let response = match request {
            Ok(request) => {
                let context = time_exec!(
                    "[REQ_ID {request_id}] Build Context",
                    RequestContext::from_server_context(server_context)
                );

                let handler = self.consolidated_handler.clone();
                handler(request, context).await
            },
            Err(err) => {
                match &err {
                    crate::Error::InternalServerError(e) => {
                        log::error!("INTERNAL SERVER ERROR: {e}");
                    },
                    crate::Error::TaskSchedculingError(e) => {
                        log::error!(
                            "TASK SCHEDULING ERROR: {}",
                            match &e {
                                crate::tasks::runner::TaskError::TaskNotFound => "Task Not Found",
                                crate::tasks::runner::TaskError::Unknown(e) => e,
                            }
                        );
                    },
                    _ => {},
                }
                err.into_response()
            },
        };

        stream.write_all(&response.as_bytes())?;

        Ok(())
    }
}
