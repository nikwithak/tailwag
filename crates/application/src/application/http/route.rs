use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, Read},
    ops::Deref,
    pin::Pin,
    sync::Arc,
};
use tailwag_macros::{Deref, Display};
use tailwag_orm::{
    data_definition::data_system::DataSystem, data_manager::PostgresDataProvider,
    queries::Insertable,
};
use tailwag_utils::{
    data_strutures::hashmap_utils::GetOrDefault, types::generic_type_map::TypeInstanceMap,
};

use crate::{
    application::http::{headers::Headers, multipart::parse_multipart_request},
    auth::gateway::Session,
};
use crate::{
    application::{http::into_route_handler::IntoRouteHandler, ConfigConstants},
    auth::gateway::AppUser,
};

/// TODO: This file has gotten huge, and contains WAY more than just route logic. Factor a bunch of this out to smaller files in more logical groupings.

pub type RoutePath = String;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoutePolicy {
    #[default]
    Public,
    #[allow(unused)]
    RequireAuthentication,
    RequireRole(String),
}

impl Display for RoutePolicy {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            RoutePolicy::Public => write!(f, "PUBLIC"),
            RoutePolicy::RequireAuthentication => write!(f, "AUTHENTICATED"),
            RoutePolicy::RequireRole(role) => write!(f, "ROLE: {role}"),
        }
    }
}

pub trait RouteAuthorizationPolicy {
    fn is_authorized(session: &Session) -> bool;
}

/// An extractor used to specify that the inputs should come from a PathVariable.
/// This wrapper is needed to prevent the extraction coming from the RequestBody
pub struct PathVariable<T>(pub T);
impl<T> Deref for PathVariable<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

/// Alias type for a path variable string extractor
pub type PathString = PathVariable<String>;
/// Alias type for path variable extractor shorthand
pub type PathVar<T> = PathVariable<T>;

#[derive(Deref)]
struct PoliciedRouteHandler {
    #[deref]
    handler: Box<RouteHandler>,
    _policy: RoutePolicy,
}

// I'll probably end up ditching this for something... better.
// I haven't quite wrapped my head around how I want to structure policies.
#[allow(unused)]
impl PoliciedRouteHandler {
    pub fn public(handler: RouteHandler) -> Self {
        Self {
            handler: Box::new(handler),
            _policy: RoutePolicy::Public,
        }
    }
    #[allow(unused)]
    pub fn protected(handler: RouteHandler) -> Self {
        Self {
            handler: Box::new(handler),
            _policy: RoutePolicy::RequireAuthentication,
        }
    }
}

#[allow(unused)]
#[derive(Default)]
pub struct Route {
    handlers: HashMap<HttpMethod, PoliciedRouteHandler>,
    children: HashMap<RoutePath, Route>,
    dynamic_child: Option<(String, Box<Route>)>, // String = the given name. Only one supported for now, and it isn't actually extracted properly.
}
impl std::fmt::Debug for Route {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("handlers", &self.handlers.keys())
            .field("children", &self.children)
            .finish()
    }
}

impl Route {
    /// Prints to stdout the route tree for all configured routes.
    fn print_routes(
        &self,
        prefix: &str,
    ) {
        for (method, handler) in &self.handlers {
            let method = method.to_string().to_uppercase();
            let padding = " ".repeat(8 - method.len());

            let policy = format!("{}", &handler._policy);
            let policy_padding = " ".repeat(20 - policy.len());
            log::info!("   {policy}{policy_padding}{method}{padding}{prefix}/");
        }
        for (path, route) in &self.children {
            let new_prefix = format!("{prefix}/{path}");
            route.print_routes(&new_prefix);
        }
        if let Some((dyn_string, route)) = &self.dynamic_child {
            let new_prefix = format!("{prefix}/{{{dyn_string}}}");
            route.print_routes(&new_prefix);
        }
    }
    pub fn print_all_routes(&self) {
        log::info!("\n\n");
        log::info!("====================================");
        log::info!("         ALL CONFIGURED ROUTES");
        log::info!("====================================");
        self.print_routes("");
        log::info!("====================================");
        log::info!("      END OF CONFIGURED ROUTES");
        log::info!("====================================");
        log::info!("\n\n");
    }
}

impl Route {
    fn parse_query_params(query_string: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let params = query_string.split("&");
        for param in params {
            if let Some((key, val)) = param.split_once('=') {
                map.insert(key.to_string(), val.to_string());
            } // TODO [Feature] work in parsing for < > <= >= etc
        }

        map
    }

    pub async fn handle(
        &self,
        mut request: Request,
        context: RequestContext,
    ) -> Response {
        let path = &request.path;
        let mut route = self;

        let (path, query_params) = path.split_once('?').unwrap_or((path, ""));
        for segment in path.split('/').filter(|s| !s.is_empty()) {
            match route.children.get(&segment.to_string()) {
                Some(new_route) => route = new_route,
                None => {
                    if let Some((name, new_route)) = &route.dynamic_child {
                        let decoded = match urlencoding::decode(segment) {
                            Ok(s) => s.into_owned(),
                            Err(_e) => return Response::bad_request(),
                        };
                        request.path_params_map.insert(name.to_string(), decoded.clone());
                        // TODO: Remove old path_params eventually (and replace with path_params_map?).
                        request.path_params.push(decoded);
                        route = new_route
                    } else {
                        return Default::default();
                    }
                },
            }
        }

        request.query_params = Self::parse_query_params(query_params);

        if let Some(future) = route.handlers.get(&request.method) {
            //TODO: Verify policy
            if is_authorized(&future._policy, &context).await {
                future.call(request, context).await
            } else {
                Response::default()
            }
        } else {
            Response::default()
        }
    }
}

async fn is_authorized(
    policy: &RoutePolicy,
    ctx: &RequestContext,
) -> bool {
    match policy {
        RoutePolicy::Public => true, // Always allow public routes
        RoutePolicy::RequireAuthentication => ctx
            .get_request_data::<Session>()
            .map_or(false, |sess| sess.get_authenticated_user().is_some()),
        RoutePolicy::RequireRole(role) => {
            // TODO: THis needs to be re-worked when Roles are addressed. For now, this only works for the
            // `Admin` role, and uses a hardcoded boolean flag instead of actual roles.
            let user =
                ctx.get_request_data::<Session>().and_then(|sess| sess.get_authenticated_user()); //map_or(false, |sess| todo!())
                                                                                                  // Hacked together - if "admin" is the requested role, and the user is an admin, then and only then will this succeed.
            user.map_or(false, |u| u.is_admin()) && role.to_lowercase() == "admin"
        },
    }
}

macro_rules! impl_method {
    ($method:ident:$variant:ident) => {
        pub fn $method<F, I, O>(
            self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
        ) -> Self {
            self.with_handler(HttpMethod::$variant, path, handler.into(), RoutePolicy::default())
        }
    };
    ($method:ident:$variant:ident, protected) => {
        pub fn $method<F, I, O>(
            self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
        ) -> Self {
            self.with_handler(
                HttpMethod::$variant,
                path,
                handler.into(),
                RoutePolicy::RequireAuthentication,
            )
        }
    };
    ($method:ident:$variant:ident, public) => {
        pub fn $method<F, I, O>(
            self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
        ) -> Self {
            self.with_handler(HttpMethod::$variant, path, handler.into(), RoutePolicy::Public)
        }
    };
    ($method:ident:$variant:ident, with_policy) => {
        pub fn $method<F, I, O>(
            self,
            path: &str,
            handler: impl IntoRouteHandler<F, I, O>,
            policy: RoutePolicy,
        ) -> Self {
            self.with_handler(HttpMethod::$variant, path, handler.into(), policy)
        }
    };
}
impl Route {
    impl_method!(get:Get);
    impl_method!(post:Post);
    impl_method!(delete:Delete);
    impl_method!(patch:Patch);
    impl_method!(get_with_policy:Get, with_policy);
    impl_method!(post_with_policy:Post, with_policy);
    impl_method!(delete_with_policy:Delete, with_policy);
    impl_method!(patch_with_policy:Patch, with_policy);
    impl_method!(get_protected:Get, protected);
    impl_method!(post_protected:Post, protected);
    impl_method!(delete_protected:Delete, protected);
    impl_method!(patch_protected:Patch, protected);
    impl_method!(get_public:Get, public);
    impl_method!(post_public:Post, public);
    impl_method!(delete_public:Delete, public);
    impl_method!(patch_public:Patch, public);
}

#[allow(unused)]
impl Route {
    // TODO: Impl the RoutePath / ValidatedString trait
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_handler<F, I, O>(
        mut self,
        method: HttpMethod,
        path: &str,
        handler: impl IntoRouteHandler<F, I, O>,
        policy: RoutePolicy,
    ) -> Self {
        self.add_handler(method, path, handler, policy);
        self
    }
    pub fn add_handler<F, I, O>(
        &mut self,
        method: HttpMethod,
        path: &str,
        handler: impl IntoRouteHandler<F, I, O>,
        policy: RoutePolicy,
    ) {
        let parts = path.split('/');
        let mut route = self;
        for part in parts.filter(|p| !p.is_empty()) {
            if let Some(captures) = Regex::new("^\\{(?<name>[a-zA-Z0-9_-]*)\\}$") // /route/{this_part_gets_matched}/
                .expect("Something wrong with regex")
                .captures(part)
            {
                // For now, only one dynamic route is allowed per route.
                // Reduces ambiguity (and lets me get away with this silly hack)
                // In the future, I'll add some regex support (maybe?) or at least a basic extraction syntax
                if route.dynamic_child.is_none() {
                    route.dynamic_child = Some((captures["name"].to_string(), Box::default()));
                }
                let (name, child_route) =
                    route.dynamic_child.as_mut().expect("Missing route that was just added.");
                route = &mut *child_route;
            } else if Regex::new("^[a-zA-Z0-9_]+$").expect("Regex is invalid").is_match(part) {
                route = route.children.get_or_default_mut(&part.to_string());
            } else if part.matches("...").next().is_some() {
                // TODO:
                todo!("... = \"the rest of the input\"");
            } else {
                log::error!("part: {} doesn't match regex", &part);
                panic!("Invalid route");
            }
        }

        // TODO: NEed to indicate that it's extracting something.
        // Static vs Dynamic routes
        if route
            .handlers
            .insert(
                method,
                PoliciedRouteHandler {
                    handler: Box::new(handler.into()),
                    _policy: policy,
                },
            )
            .is_some()
        {
            panic!("This route ({}) already has a handler", &path);
        }
    }

    pub fn with_route(
        mut self,
        path: RoutePath,
        route: Route,
    ) -> Self {
        self.route(path, route);
        self
    }

    /// Nest another Route inside this one.
    /// ```
    pub fn route(
        &mut self,
        path: impl Into<RoutePath>,
        route: Route,
    ) {
        // 1. Parse path (as str) into the parts
        // 2. Go down the request tree to find the Route node where this route would be handled, creating new Route nodes when necessary.
        // 3. Add the Route handler
        self.children.insert(path.into(), route);
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Display)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Options,
}

impl TryFrom<&str> for HttpMethod {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type E = HttpMethod;
        let method = match value.to_uppercase().as_str() {
            "GET" => E::Get,
            "POST" => E::Post,
            "PATCH" => E::Patch,
            "DELETE" => E::Delete,
            "OPTIONS" => E::Options,
            _ => Err(format!("Unsupported HTTP method: {}", value))?,
        };
        Ok(method)
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(usize)]
#[derive(Debug)]
pub enum HttpStatus {
    Ok = 200,
    Accepted = 201,
    SeeOther = 303,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Conflict = 409,
    EntityTooLarge = 413,
    UnsupportedMediaType = 415,
    IAmATeapot = 418,
    NotImplemented = 501,
    InternalServerError = 503,
}

impl Display for HttpStatus {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let var_name = format!(
            "{} {}",
            self.clone() as usize,
            match self {
                HttpStatus::Ok => "OK",
                HttpStatus::Accepted => "Accepted",
                HttpStatus::SeeOther => "See Other",
                HttpStatus::BadRequest => "Bad Request",
                HttpStatus::Unauthorized => "Unauthorized",
                HttpStatus::Forbidden => "Forbidden",
                HttpStatus::NotFound => "Not Found",
                HttpStatus::NotImplemented => "Not Implemented",
                HttpStatus::IAmATeapot => "I Am A Teapot",
                HttpStatus::InternalServerError => "Internal Server Error",
                HttpStatus::Conflict => "Conflict",
                HttpStatus::EntityTooLarge => "Entity Too Large",
                HttpStatus::UnsupportedMediaType => "Unsupported Media Type",
            }
        );
        f.write_str(&var_name)
    }
}

type RouteHandlerInner = Box<
    dyn Send
        + Sync
        + 'static
        + Fn(
            Request,
            RequestContext,
        ) -> Pin<Box<dyn Send + 'static + std::future::Future<Output = Response>>>,
>;
pub struct RouteHandler {
    pub(crate) handler: RouteHandlerInner,
}
impl RouteHandler {
    pub async fn call(
        &self,
        request: Request,
        context: RequestContext,
    ) -> Response {
        (self.handler)(request, context).await
    }
}

#[derive(Debug)]
pub enum HttpVersion {
    V1_1,
    V2_0,
    V3_0,
}
impl TryFrom<&str> for HttpVersion {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        type E = HttpVersion;

        Ok(match value {
            "HTTP/1.1" => E::V1_1,
            "HTTP/2.0" => E::V2_0,
            "HTTP/3.0" => E::V3_0,
            _ => Err(format!("Unsupported HTTP version: {}", value))?,
        })
    }
}
impl Deref for HttpVersion {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            HttpVersion::V1_1 => "HTTP/1.1",
            HttpVersion::V2_0 => "HTTP/2.0",
            HttpVersion::V3_0 => "HTTP/3.0",
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub path_params: Vec<String>,
    pub path_params_map: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub http_version: HttpVersion,
    pub headers: Headers,
    pub body: HttpBody,
}

impl<T: for<'a> Deserialize<'a>> FromRequest for T {
    fn from(req: Request) -> Result<Self, crate::Error> {
        // TODO: Return this as a Result so we can route based on it later
        // ^^^ that didn't work,
        match &req.body {
            HttpBody::Json(body) => {
                let bytes = body.as_bytes();
                let desered = serde_json::from_slice(bytes)?;
                Ok(desered)
                // serde_json::from_slice(body.as_bytes()).unwrap(),
            },
            // HttpBody::Bytes(_) => todo!(),
            // HttpBody::Stream(_) => todo!(),
            // HttpBody::Multipart(_) => todo!(),
            // // HttpBody::Plaintext(String) => todo!(),
            // HttpBody::None => todo!(),
            // HttpBody::Html(_) => todo!(),
            _ => Err("Unsupported content-type provided - the way content-type is handled needs to be rethought.")?
        }
    }
}

#[derive(Debug)]
// TODO: This abstruction turned out to suck. Keep the content-type separate from the data.
pub enum HttpBody {
    // pub bytes: Vec<u8>,
    Json(String),
    Bytes(Vec<u8>),
    Multipart(HashMap<String, MultipartPart>),
    Stream(std::io::BufReader<std::net::TcpStream>),
    Html(String),
    None,
}

const DEFAULT_CONTENT_TYPE: &str = "application/json";

impl TryFrom<&std::net::TcpStream> for Request {
    fn try_from(stream: &std::net::TcpStream) -> Result<Self, Self::Error> {
        let mut stream = std::io::BufReader::new(stream);
        let stream = &mut stream;

        let mut line = String::new();
        stream.take(ConfigConstants::request_line_max_length()).read_line(&mut line)?;

        let mut routing_line = line.split_whitespace();
        let (Some(method), Some(path), Some(http_version)) =
            (routing_line.next(), routing_line.next(), routing_line.next())
        else {
            Err(Self::Error::BadRequest(format!("Invalid routing header found: {}", &line)))?
        };
        let headers = Headers::parse_headers(stream)?;
        let content_length: usize =
            headers.get("content-length").and_then(|c| c.parse().ok()).unwrap_or(0);
        let content_type_header =
            headers.get("content-type").map(|s| s.as_str()).unwrap_or(DEFAULT_CONTENT_TYPE);

        let (content_type, content_type_params) =
            content_type_header.split_once(';').unwrap_or((content_type_header, ""));
        type E = HttpBody;

        if content_length > ConfigConstants::max_content_length() as usize {
            crate::Error::entity_too_large()?;
        }
        let body = if content_length > 0 {
            let mut bytes = vec![0; content_length];
            log::info!("Reading {} bytes", content_length);
            stream.read_exact(&mut bytes)?;
            match content_type.to_lowercase().as_str() {
                "application/json" => E::Json(String::from_utf8(bytes)?),
                "multipart/form-data" => parse_multipart_request(content_type_params, bytes)?,
                _ => crate::Error::unsupported_media_type()?,
            }
        } else {
            HttpBody::None
        };

        Ok(Request {
            method: method.try_into()?,
            path: path.to_string(), // TODO: Validate it
            // DEPRECATED: Use path_params_map instead.
            path_params: Default::default(),
            path_params_map: Default::default(),
            query_params: Default::default(),
            http_version: http_version.try_into()?,
            headers,
            body,
        })
    }

    type Error = crate::Error;
}

#[derive(Debug)]
pub struct Response {
    pub http_version: HttpVersion,
    pub status: HttpStatus,
    pub headers: Headers,
    pub body: Vec<u8>,
}
// impl<E: std::error::Error> From<E> for Response {
//     fn from(value: E) -> Self {
//         Response::internal_server_error()
//     }
// }

impl From<tailwag_orm::Error> for Response {
    fn from(value: tailwag_orm::Error) -> Self {
        log::error!("Request errored out: {:?}", &value);
        Response::internal_server_error()
    }
}

macro_rules! default_response {
    ($fnname:ident, $enumname:ident) => {
        /// Creates a default request with the given status codes.
        pub fn $fnname() -> Self {
            Self {
                http_version: HttpVersion::V1_1,
                status: HttpStatus::$enumname,
                headers: Headers::default(),
                body: Vec::new(),
            }
        }
    };
}

/// Factory Methods
impl Response {
    default_response!(bad_request, BadRequest);
    default_response!(not_found, NotFound);
    default_response!(not_implemented, NotImplemented);
    default_response!(internal_server_error, InternalServerError);
    default_response!(entity_too_large, EntityTooLarge);
    default_response!(unauthorized, Unauthorized);
    default_response!(unsupported_media_type, UnsupportedMediaType);
    default_response!(conflict, Conflict);
    default_response!(ok, Ok);
    pub fn redirect_see_other(redirect_url: &str) -> Self {
        let mut headers = Headers::default();
        headers.insert("Location".into(), redirect_url.into());

        Self {
            http_version: HttpVersion::V1_1,
            status: HttpStatus::SeeOther,
            headers,
            body: Vec::new(),
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::not_found()
    }
}

impl Response {
    pub fn with_body(
        mut self,
        bytes: Vec<u8>,
    ) -> Self {
        self.body = bytes;
        self
    }
    pub fn with_header(
        mut self,
        name: impl Into<String>,
        val: impl Into<String>,
    ) -> Self {
        self.headers.insert(name.into().to_lowercase(), val.into().into());
        self
    }
}

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            format!("{} {}", (&self.http_version as &str), self.status).as_bytes(),
        );
        bytes.extend_from_slice(b"\r\n");
        for header in (&self.headers) as &HashMap<_, _> {
            bytes.extend_from_slice(format!("{}: {}", &header.0, &header.1).as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(format!("{}: {}", "Content-Length", self.body.len()).as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&self.body);

        bytes
    }
}

#[derive(Clone, Deref)]
pub struct ServerContext {
    #[deref]
    pub(crate) data_providers: DataSystem,
    pub(crate) server_data: Arc<TypeInstanceMap>,
}

#[derive(Deref)]
pub struct RequestContext {
    #[deref]
    server_context: ServerContext,
    request_data: TypeInstanceMap,
    data_providers: DataSystem,
}

impl RequestContext {
    pub fn from_server_context(server_context: ServerContext) -> Self {
        Self {
            request_data: Default::default(),
            data_providers: server_context.data_providers.clone(),
            server_context,
        }
    }
    pub(crate) fn server_context(&self) -> ServerContext {
        self.server_context.clone()
    }
}

impl RequestContext {
    /// Gets the requested data type from the request context, if it exists.
    /// Useful for maintaining data state between beforeware & afterware (e.g. wrapping with middleware)
    pub fn get_request_data<T: 'static + Sync + Send>(&self) -> Option<&T> {
        self.request_data.get::<T>()
    }
    pub fn get_request_data_mut<T: 'static + Sync + Send>(&mut self) -> Option<&mut T> {
        self.request_data.get_mut::<T>()
    }
    pub fn insert_request_data<T: 'static + Sync + Send>(
        &mut self,
        t: T,
    ) {
        self.request_data.insert(t);
    }
    pub fn get_authenticated_user(&self) -> Option<&AppUser> {
        self.get_request_data()
    }

    pub fn get_server_data<T: 'static + Sync + Send>(&self) -> Option<&T> {
        self.server_data.get()
    }
}

impl From<RequestContext> for ServerContext {
    fn from(val: RequestContext) -> Self {
        val.server_context.clone()
    }
}

impl<T: Insertable + Clone + Send + 'static> From<ServerContext> for PostgresDataProvider<T> {
    fn from(value: ServerContext) -> Self {
        value.data_providers.get::<T>().unwrap()
    }
}

impl From<&RequestContext> for DataSystem {
    fn from(ctx: &RequestContext) -> Self {
        ctx.data_providers.clone()
    }
}

pub struct ServerData<T: Clone + Send + Sync + 'static>(pub T);

impl<T: Clone + Send + Sync + 'static> Deref for ServerData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone + Send + Sync + 'static> From<&RequestContext> for ServerData<T> {
    fn from(ctx: &RequestContext) -> Self {
        // TODO: Use TryFrom instead
        Self(ctx.server_context.server_data.get::<T>().unwrap().clone())
    }
}

// impl<T: Clone + Send + Sync + 'static> TryFrom<&RequestContext> for ServerData<T> {
//     type Error = ApplicationError;
//     fn try_from(ctx: &RequestContext) -> Result<Self, Self::Error> {
//         // TODO: Use TryFrom instead
//         Ok(Self(
//             ctx.server_context
//                 .server_data
//                 .get::<T>()
//                 .cloned()
//                 .ok_or(ApplicationError::ServerResourceNotFound)?,
//         ))
//     }
// }

impl<T: Clone + Send + Sync + 'static> From<ServerContext> for ServerData<T> {
    fn from(ctx: ServerContext) -> Self {
        // TODO: Use TryFrom instead
        Self(ctx.server_data.get::<T>().unwrap().clone())
    }
}

impl<T: Insertable + Clone + Send + Sync + 'static> From<&RequestContext>
    for PostgresDataProvider<T>
{
    fn from(ctx: &RequestContext) -> Self {
        ctx.data_providers
            .get::<T>()
            .clone()
            .expect("Attempted to use DataProvider that does not exist.")
    }
}

pub trait IntoResponse
where
    Self: Sized,
{
    fn into_response(self) -> Response;
}

impl<T: Serialize> IntoResponse for T {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(body) => crate::application::http::route::Response {
                status: crate::application::http::route::HttpStatus::Ok,
                headers: Headers::from(vec![("Content-Type", "application/json")]), // TODO: Make this dynamic
                http_version: crate::application::http::route::HttpVersion::V1_1,
                body: body.into_bytes(),
            },
            Err(_) => crate::application::http::route::Response {
                status: crate::application::http::route::HttpStatus::InternalServerError,
                headers: Headers::from(vec![]),
                http_version: crate::application::http::route::HttpVersion::V1_1,
                body: Vec::new(),
            },
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

pub trait FromRequest
where
    Self: Sized,
{
    fn from(req: Request) -> Result<Self, crate::Error>;
}

impl FromRequest for Request {
    fn from(req: Request) -> Result<Self, crate::Error> {
        Ok(req)
    }
}
impl<T> FromRequest for PathVariable<T>
where
    T: From<String> + Display,
{
    fn from(req: Request) -> Result<Self, crate::Error> {
        // TODO: Not robust
        match req.path_params.first() {
            Some(val) => Ok(PathVariable(val.to_owned().into())),
            None => Err("Unable to extract path variable".into()),
        }
    }
}

use super::multipart::MultipartPart;
