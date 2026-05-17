use tailwag_web_service::{
    application::WebService,
    auth::gateway::{extract_session, login, register},
    Error,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    WebService::builder("AuthN/AuthZ Service")
        .with_middleware(extract_session)
        .post_public("login", login)
        .post_public("register", register)
        .post("echo", echo)
        .build_service()
        .run()
        .await?;

    async fn echo(value: String) -> String {
        log::info!("Your request: {}", &value);
        value
    }
    Ok(())
}
