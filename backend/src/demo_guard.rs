//! Reject mutating API calls when Traefik sets `X-Demo-Readonly: true`.

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub fn is_demo_readonly_headers(headers: &actix_web::http::header::HeaderMap) -> bool {
    headers
        .get("X-Demo-Readonly")
        .and_then(|v| v.to_str().ok())
        == Some("true")
}

pub struct DemoReadonly;

impl<S, B> Transform<S, ServiceRequest> for DemoReadonly
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = DemoReadonlyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DemoReadonlyMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct DemoReadonlyMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DemoReadonlyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if is_demo_readonly_headers(req.headers()) {
            let method = req.method().clone();
            let is_safe =
                method == Method::GET || method == Method::HEAD || method == Method::OPTIONS;
            if !is_safe {
                let res = req.into_response(
                    HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "demo_readonly",
                            "message": "This demo host is read-only."
                        }))
                        .map_into_right_body(),
                );
                return Box::pin(async move { Ok(res) });
            }
        }

        let service = self.service.clone();
        Box::pin(async move { service.call(req).await.map(|res| res.map_into_left_body()) })
    }
}
