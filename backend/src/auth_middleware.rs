use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, web};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use crate::models::{AppState, Claims};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let state = req.app_data::<web::Data<AppState>>().expect("AppState not found");
        let jwt_secret = state.jwt_secret.clone();

        // Check for any valid login token. Primary source is the Authorization header
        // (Bearer token). For use-cases like EventSource where browsers cannot set
        // custom headers, allow a `token` query parameter as a fallback. This keeps
        // the route protected while enabling SSE connections from the frontend.
        let headers = req.headers();
        let mut is_authorized = false;
        let mut auth_method: Option<&str> = None; // "header" or "query"

        if let Some(auth_hdr) = headers.get(actix_web::http::header::AUTHORIZATION) {
            if let Ok(s) = auth_hdr.to_str() {
                if let Some(token) = s.strip_prefix("Bearer ") {
                    // Check for admin or API token first (static keys)
                    if crate::auth::verify_admin_token(token) || crate::auth::verify_api_token(token) {
                        is_authorized = true;
                        auth_method = Some("header");
                    } else {
                        // Then try to decode as JWT
                        if decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(jwt_secret.as_bytes()),
                            &Validation::default(),
                        ).is_ok() {
                            is_authorized = true;
                            auth_method = Some("header");
                        }
                    }
                }
            }
        }

        // If Authorization header didn't authenticate, check `token` query param
        if !is_authorized {
            if let Some(q) = req.uri().query() {
                // simple parse for token key: token=... (URL-decoded)
                for pair in q.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next().unwrap_or("");
                    let val = parts.next().unwrap_or("");
                    if key == "token" {
                        // decode percent-encoding if present
                        let token = urlencoding::decode(val).unwrap_or_else(|_| val.into()).into_owned();
                        if crate::auth::verify_admin_token(&token) || crate::auth::verify_api_token(&token) {
                            is_authorized = true;
                            auth_method = Some("query");
                            break;
                        }

                        if decode::<Claims>(
                            &token,
                            &DecodingKey::from_secret(jwt_secret.as_bytes()),
                            &Validation::default(),
                        ).is_ok() {
                            is_authorized = true;
                            auth_method = Some("query");
                            break;
                        }
                    }
                }
            }
        }

        // Log how the request was authenticated for easier debugging of SSE connections.
        // Do NOT log the token value itself.
        if is_authorized {
            log::debug!("AuthMiddleware: request authorized via {:?}", auth_method);
        } else {
            log::debug!("AuthMiddleware: request UNAUTHORIZED (no valid token found)");
            // Return a JSON 401 response so callers get machine-readable errors
            let resp = actix_web::HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "unauthorized", "message": "No valid authentication token provided"}));
            return Box::pin(async { Err(actix_web::error::InternalError::from_response("unauthorized", resp).into()) }) as _;
        }

        Box::pin(async move {
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
