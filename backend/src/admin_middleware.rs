use crate::auth;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{error::ErrorUnauthorized, Error};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

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

        // Require Authorization: Bearer <token>
        let headers = req.headers();
        if let Some(auth_hdr) = headers.get(actix_web::http::header::AUTHORIZATION) {
            if let Ok(s) = auth_hdr.to_str() {
                if let Some(token) = s.strip_prefix("Bearer ") {
                    if auth::verify_admin_token(token) {
                        // authorized
                    } else {
                        return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) }) as _;
                    }
                } else {
                    return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) }) as _;
                }
            } else {
                return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) }) as _;
            }
        } else {
            return Box::pin(async { Err(ErrorUnauthorized("unauthorized")) }) as _;
        }

        Box::pin(async move {
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
