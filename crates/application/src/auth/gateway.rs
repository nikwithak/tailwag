use rand::Rng;
use std::{pin::Pin, sync::Arc, time::Duration};
use tailwag_orm::{
    data_definition::data_system::DataSystem, data_manager::traits::WithFilter,
    queries::filterable_types::FilterEq, OrmResult,
};
use totp_rs::TOTP;

use crate::{
    application::http::route::{FromRequest, RoutePolicy},
    option_utils::OrError,
    HttpResult,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tailwag_macros::BuildRoutes;
use tailwag_orm::data_manager::{traits::DataProvider, PostgresDataProvider};
use tailwag_orm_macros::Filterable;
use uuid::Uuid;

use crate::application::{
    http::route::{IntoResponse, Request, RequestContext, Response},
    NextFn,
};

mod tailwag {
    pub use crate as web;
    pub use tailwag_forms as forms;
    pub use tailwag_orm as orm;
}

#[derive(Clone)]
pub(crate) struct JwtSecret(String);
impl JwtSecret {
    pub(crate) fn init() -> Self {
        Self(std::env::var("JWT_SECRET").ok().unwrap_or_else(|| {
            rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(64)
                .map(<char as From<_>>::from)
                .collect()
        }))
    }
}

#[derive(
    Clone, // Needed to be able to create an editable version from an Arc<Brewery> without affecting the saved data.
    Debug,
    Default,
    Deserialize, // Needed for API de/serialization
    Serialize,   // Needed for API de/serialization
    // sqlx::FromRow,                      // Needed for DB connectivity
    tailwag_orm_macros::GetTableDefinition, // Creates the data structure needed for the ORM to work.
    tailwag_orm_macros::Insertable,
    tailwag_orm_macros::Updateable,
    tailwag_orm_macros::Deleteable,
    Filterable,
    BuildRoutes,
    tailwag::forms::macros::GetForm,
)]
#[views(("/current", get_current_user, RoutePolicy::RequireAuthentication))]
#[actions(("/totp/enable", enable_totp, RoutePolicy::RequireAuthentication),("/totp/confirm", confirm_totp, RoutePolicy::RequireAuthentication))]
#[policy(RoutePolicy::RequireRole("Admin".to_string()))]
#[no_default_routes]
#[create_type(AppUserCreateRequest)]
pub struct AppUser {
    id: uuid::Uuid,
    email_address: String,
    #[serde(skip_serializing)]
    passhash: String,
    // TODO: - this flag should later be replaced with an actual RBAC / ABAC system.
    is_admin: bool,
    #[serde(skip_serializing)]
    // TODO: Encrypt this with an application-specific encryption secret loaded at runtime.
    totp_secret: Option<String>,
    totp_enabled: Option<bool>,
}

impl AppUser {
    fn generate_totp_secret(&mut self) {
        self.totp_secret = Some(totp_rs::TOTP::default().get_secret_base32());
    }

    fn verify_totp_code(
        &self,
        user_code: String,
    ) -> bool {
        if let Some(secret) = &self.totp_secret {
            let mut totp = TOTP::default();
            let Ok(secret) = totp_rs::Secret::Encoded(secret.clone()).to_bytes() else {
                return false;
            };
            totp.secret = secret;
            let Ok(code) = totp.generate_current() else {
                return false;
            };
            code == user_code
        } else {
            // Returns *false* if TOTP is disabled - user shouldn't be giving
            // a TOTP code in this case.
            false
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppUserCreateRequest {
    pub email_address: String,
    pub password: String,
    pub is_admin: bool,
}
impl Into<AppUser> for AppUserCreateRequest {
    fn into(self) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            email_address: self.email_address,
            is_admin: self.is_admin,
            passhash: {
                let salt = &SaltString::generate(&mut OsRng);
                let passhash = Argon2::default()
                    .hash_password(self.password.as_bytes(), salt)
                    .expect("Failed to hash password - this should not happen")
                    .to_string();
                passhash
            },
            totp_secret: None,
            totp_enabled: Some(false),
        }
    }
}

impl AppUser {
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

// pub fn get_current_user(req: Request) -> Response {
//     Response::not_implemented()
// }

pub async fn get_current_user(
    _request: Request,
    _users: PostgresDataProvider<AppUser>,
    ctx: RequestContext,
) -> Response {
    let Some(session) = ctx.get_request_data::<Session>() else {
        return Response::not_found();
    };
    // let Some(user) = users.get(|u| u.id.eq(session.)).await.ok().flatten() else {
    //     return Response::not_found();
    // };
    session.account.clone().into_response()
}

impl tailwag::orm::data_manager::rest_api::Id for AppUser {
    fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

#[derive(
    Clone, // Needed to be able to create an editable version from an Arc<Brewery> without affecting the saved data.
    Debug,
    Default,
    Deserialize, // Needed for API de/serialization
    Serialize,   // Needed for API de/serialization
    // sqlx::FromRow, // Needed for DB connectivity
    BuildRoutes,
    tailwag_orm_macros::GetTableDefinition, // Creates the data structure needed for the ORM to work.
    tailwag_orm_macros::Insertable,
    tailwag_orm_macros::Updateable,
    tailwag_orm_macros::Deleteable,
    tailwag_orm_macros::Filterable,
    tailwag::forms::macros::GetForm,
)]
#[policy(RoutePolicy::RequireRole("Admin".to_string()))]
#[no_default_routes]
pub struct Session {
    id: uuid::Uuid,
    #[ref_only]
    #[no_filter]
    account: AppUser,
    start_time: chrono::NaiveDateTime,
    expiry_time: chrono::NaiveDateTime,
}
impl tailwag::orm::data_manager::rest_api::Id for Session {
    fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

impl From<&RequestContext> for Option<Session> {
    fn from(value: &RequestContext) -> Self {
        value.get_request_data().cloned()
    }
}

pub type UserRole = String;

#[derive(Default)]
pub enum AccountType {
    #[default]
    Anonymous, // Public
    Authenticated(AppUser),
}

pub struct AuthorizationGateway;

#[derive(Serialize, Deserialize)]
struct JwtClaims {
    session_id: Uuid,
    exp: usize,
}

pub fn extract_session(
    request: Request,
    mut context: RequestContext,
    next: Arc<NextFn>,
) -> Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
    Box::pin(async move {
        let (Some(sessions), Some(_users)) = (context.get::<Session>(), context.get::<AppUser>())
        else {
            return Response::internal_server_error();
        };

        fn extract_authz_token(request: &Request) -> Option<String> {
            if let Some(header) = request
                .headers
                .get("Authorization")
                .and_then(|header| header.as_str().strip_prefix("Bearer "))
            {
                Some(header.to_owned())
            } else if let Some(cookie) =
                request.headers.get("Cookie").map(|header| header.as_str().to_string())
            {
                let session_cookie = cookie
                    .split(';')
                    .map(|cookie| cookie.trim())
                    .find(|cookie| cookie.starts_with("_id"))
                    .and_then(|cookie| cookie.split('=').last())
                    .map(|cookie| cookie.trim().into());
                session_cookie
            } else {
                None
            }
        }

        let Some(jwt_secret) = context.get_server_data::<JwtSecret>() else {
            return Response::internal_server_error();
        };
        let session_id = extract_authz_token(&request)
            .and_then(|token| {
                jsonwebtoken::decode::<JwtClaims>(
                    &token,
                    &jsonwebtoken::DecodingKey::from_secret(jwt_secret.0.as_ref()),
                    &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
                )
                .ok()
            })
            .map(|token_data| token_data.claims)
            .map(|claims| claims.session_id);

        let session = match session_id {
            Some(session_id) => sessions.get(|sess| sess.id.eq(session_id)).await,
            None => Ok(None),
        };
        match session {
            Ok(Some(session)) => {
                log::debug!("Session found! {:?}", &session);
                log::debug!("Adding session to RequestContext");
                if let Some(user) = session.get_authenticated_user() {
                    context.insert_request_data(user);
                }
                context.insert_request_data(session);
                next(request, context).await
            },
            Ok(None) => {
                log::debug!("No session found for request.");
                next(request, context).await
            },
            Err(e) => {
                log::error!("An error occurred while authorizing the account: {:?}", e);
                Response::unauthorized()
            },
        }
    })
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    email_address: String,
    password: String,
    totp_code: Option<String>,
}

// TODO: Move to config
const SESSION_LENGTH_MS: u64 = 36000000;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    access: String,
    refresh: String,
}
pub async fn login(
    req: Request,
    ctx: RequestContext,
) -> Result<Response, crate::Error> {
    let creds: LoginRequest = <LoginRequest as FromRequest>::from(req)?;
    let providers: DataSystem = DataSystem::try_from(&ctx)?;
    let accounts = providers.get::<AppUser>().ok_or(crate::Error::NotFound)?;
    let sessions = providers.get::<Session>().ok_or(crate::Error::NotFound)?;
    let jwt_secret: &JwtSecret = ctx.get_server_data().or_500("Missing JWT Secrets")?;

    let account = accounts
        .with_filter(|acct| acct.email_address.eq(&creds.email_address))
        .execute()
        .await
        .unwrap()
        // .ok()
        // TODO: Need to update get() to ensure only one exists
        // .and_then(|mut vec| vec.pop())
        .pop()
        .ok_or_else(|| {
            // TODO: Protect against authn timing attacks, by verifying the password against a dummy hash, and writing a dummy session to the store.
            crate::Error::NotFound
        })?;

    argon2::Argon2::default().verify_password(
        creds.password.as_bytes(),
        &argon2::PasswordHash::new(&account.passhash).unwrap(),
    )?;

    if account.totp_enabled.unwrap_or_default() {
        if !account.verify_totp_code(creds.totp_code.or_404()?) {
            crate::Error::not_found()?;
        }
    }

    let account = AppUser {
        passhash: "".into(),
        ..account
    };

    let Ok(new_session) = sessions
        .create(SessionCreateRequest {
            account,
            start_time: Utc::now().naive_utc(),
            expiry_time: Utc::now().naive_utc() + Duration::from_millis(SESSION_LENGTH_MS),
        })
        .await
    else {
        log::error!("Unable to create session for new login");
        todo!("Handle errors with the IntoResponse stuff")
    };
    let jwt = jsonwebtoken::encode(
        &Default::default(),
        &JwtClaims {
            session_id: new_session.id,
            exp: new_session.expiry_time.and_utc().timestamp() as usize,
        },
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.0.as_ref()),
    )
    .expect("Couldn't encode JWT");

    let response = LoginResponse {
        access: jwt.clone(),
        refresh: "".into(),
    };

    #[cfg(debug_assertions)] // Don't set Secure cookies on dev mode.
    let cookie_header_val = format!("_id={}; HttpOnly; SameSite=Strict; Path=/", jwt,);
    #[cfg(not(debug_assertions))] // DO set Secure cookies in Prod / release mode
    let cookie_header_val = format!("_id={}; HttpOnly; Secure; SameSite=Strict; Path=/;", jwt,);
    let response = response.into_response().with_header("Set-Cookie", cookie_header_val);
    Ok(response)
}

pub async fn logout(
    _req: (),
    sessions: PostgresDataProvider<Session>,
    ctx: RequestContext,
) -> Response {
    if let Some(session) = ctx.get_request_data::<Session>() {
        sessions.delete(session.clone()).await.ok();
    }
    Response::ok()
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    email_address: String, // TODO: ValidatedString
    password: String,
}
#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    account_id: Uuid,
}
pub async fn register(
    request: RegisterRequest,
    accounts: PostgresDataProvider<AppUser>,
) -> Option<RegisterResponse> {
    let account = accounts
        .create(AppUserCreateRequest {
            email_address: request.email_address,
            is_admin: false,
            password: request.password,
        })
        .await
        // TODO: Error instead of Option
        .ok()?;

    let response = RegisterResponse {
        account_id: account.id,
    };
    Some(response)
}

pub async fn enable_totp(
    _: Request,
    context: RequestContext,
) -> HttpResult<Option<String>> {
    let session: &Session = context.get_request_data().or_404()?;
    let users: PostgresDataProvider<AppUser> = context.get().or_404()?;
    let user = session.get_authenticated_user().or_404()?;
    // Get the latest from the DB, in case Session is outdated.
    let mut user = users.get(|u| u.id.eq(user.id)).await?.or_404()?;
    user.generate_totp_secret();
    users.update(&user).await?;
    Ok(user.totp_secret)
}

#[derive(Deserialize)]
pub struct ConfirmTotpRequest {
    code: String,
}

pub async fn confirm_totp(
    // request: ConfirmTotpRequest,
    request: Request,
    context: RequestContext,
    // users: PostgresDataProvider<AppUser>,
) -> HttpResult<()> {
    let session: &Session = context.get_request_data().or_404()?;
    let users: PostgresDataProvider<AppUser> = context.get().or_404()?;
    let user = session.get_authenticated_user().or_404()?;
    let request = <ConfirmTotpRequest as FromRequest>::from(request)?;
    // Get the latest from the DB, in case Session is outdated.
    let mut user = users.get(|u| u.id.eq(user.id)).await?.or_404()?;

    if user.verify_totp_code(request.code) {
        user.totp_enabled = Some(true);
        users.update(&user).await?;
        Ok(())
    } else {
        crate::HttpError::bad_request("Invalid TOTP code")
    }
}

impl Session {
    pub fn get_authenticated_user(&self) -> Option<AppUser> {
        // let user_id = &self.account.id;
        // let user = users.get(|user| user.id.eq(*user_id)).await?;
        // Ok(user)
        Some(self.account.clone())
    }
    #[deprecated = "Use get_authenticated_user() instead"]
    pub async fn get_current_user(
        &self,
        // users: PostgresDataProvider<AppUser>,
    ) -> OrmResult<Option<AppUser>> {
        // let user_id = &self.account.id;
        // let user = users.get(|user| user.id.eq(*user_id)).await?;
        // Ok(user)
        Ok(self.get_authenticated_user())
    }
}
