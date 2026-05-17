use std::sync::Arc;

use tailwag_web_service::application::{
    http::route::{Request, RequestContext},
    NextFn, WebService,
};

#[tokio::main]
pub async fn main() {
    WebService::builder("Middleware Example Service")
        .with_middleware(|req: Request, ctx: RequestContext, next: Arc<NextFn>| {
            Box::pin(async move {
                log::info!("Receivedc request: {:?}", &req.body);
                let res = next(req, ctx).await;
                log::info!("Finished request");
                res
            })
        })
        .post("echo", echo)
        .build_service()
        .run()
        .await
        .unwrap();

    async fn echo(value: String) -> String {
        log::info!("Your request: {}", &value);
        value
    }
}
