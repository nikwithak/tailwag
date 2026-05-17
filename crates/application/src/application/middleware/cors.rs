use std::{collections::HashMap, pin::Pin, sync::Arc};

use crate::application::{
    http::route::{HttpMethod, Request, RequestContext, Response},
    NextFn, WebServiceConfig,
};

#[derive(Debug)]
pub struct CorsHeaders(pub HashMap<String, String>);

mod headers {
    // Ref: https://fetch.spec.whatwg.org/#http-requests 3.2.2 & 3.2.3
    use std::fmt::Display;

    pub enum CorsHeader {
        Origin,
        AccessControlRequestMethod,
        AccessControlRequestHeaders,
        AccessControlAllowOrigin,
        AccessControlAllowCredentials,
        AccessControlAllowMethods,
        AccessControlAllowHeaders,
        AccessControlMaxAge,
        AccessControlExposeHeaders,
    }

    impl Display for CorsHeader {
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            f.write_str(match self {
                CorsHeader::Origin => "Origin",
                CorsHeader::AccessControlRequestMethod => "Access-Control-Request-Method",
                CorsHeader::AccessControlRequestHeaders => "Access-Control-Request-Headers",
                CorsHeader::AccessControlAllowOrigin => "Access-Control-Allow-Origin",
                CorsHeader::AccessControlAllowCredentials => "Access-Control-Allow-Credentials",
                CorsHeader::AccessControlAllowMethods => "Access-Control-Allow-Methods",
                CorsHeader::AccessControlAllowHeaders => "Access-Control-Allow-Headers",
                CorsHeader::AccessControlMaxAge => "Access-Control-Max-Age",
                CorsHeader::AccessControlExposeHeaders => "Access-Control-Expose-Headers",
            })
        }
    }

    impl From<CorsHeader> for String {
        fn from(val: CorsHeader) -> Self {
            val.to_string()
        }
    }
}
pub use headers::*;

/// Implements the CORS specification, as defined by the fetch spec.
/// It is not currently fully compliant.
/// Goal is "common case" with some flexibility
/// ref: https://fetch.spec.whatwg.org/#cors-protocol
pub fn handle_cors(
    req: Request,
    ctx: RequestContext,
    next: Arc<NextFn>,
) -> Pin<Box<dyn Send + std::future::Future<Output = Response>>> {
    Box::pin(async move {
        let Some(allowed_origins) = ctx
            .get_server_data::<Arc<WebServiceConfig>>()
            .map(|config| &config.cors_allowed_origins)
        else {
            log::error!("WebServiceConfig is missing - this is a fatal error.");
            return Response::internal_server_error();
        };

        let request_origin = req
            .headers
            .get(&CorsHeader::Origin.to_string())
            .map(|header| header.to_string().trim().to_string());

        let is_allowed_origin = request_origin
            .as_ref()
            .map(|origin| allowed_origins.contains(origin))
            .map(|allowed| allowed || allowed_origins.contains("*"))
            .unwrap_or(false);

        let response = if matches!(req.method, HttpMethod::Options) {
            // Preflight request - always sends 200. If it is an invalid origin, then we omit the
            // CORS headers in the response. It is the browser's responsibility to contain this this.
            Response::ok()
        } else {
            next(req, ctx).await
        };

        if is_allowed_origin {
            response
                .with_header(
                    CorsHeader::AccessControlAllowOrigin.to_string(),
                    request_origin.unwrap_or_default(),
                )
                .with_header(CorsHeader::AccessControlAllowCredentials, "true")
                .with_header(CorsHeader::AccessControlAllowHeaders, "origin, content-type, accept")
                .with_header(CorsHeader::AccessControlAllowMethods, "GET, POST, DELETE, PATCH")
        } else {
            response
        }
    })
}
