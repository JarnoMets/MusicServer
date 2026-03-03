use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, web};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use crate::models::{AppState, Claims};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct AdminMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
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

        // Require Authorization: Bearer <token>
        let headers = req.headers();
        let is_admin = if let Some(auth_hdr) = headers.get(actix_web::http::header::AUTHORIZATION) {
            if let Ok(s) = auth_hdr.to_str() {
                if let Some(token) = s.strip_prefix("Bearer ") {
                   // Check for admin token first (legacy/internal)
                   if crate::auth::verify_admin_token(token) {
                       true
                   } else {
                       // Then try to decode as JWT and check is_admin
                       match decode::<Claims>(
                           token,
                           &DecodingKey::from_secret(jwt_secret.as_bytes()),
                           &Validation::default(),
                       ) {
                           Ok(token_data) => token_data.claims.is_admin,
                           Err(_) => false
                       }
                   }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !is_admin {
            // Return a JSON 401 response instead of plain text so clients can parse it easily
            let resp = actix_web::HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "requires_admin", "message": "This action requires admin privileges"}));
            return Box::pin(async { Err(actix_web::error::InternalError::from_response("unauthorized", resp).into()) }) as _;
        }

        Box::pin(async move {
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
