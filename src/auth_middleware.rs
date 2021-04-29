
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use actix_http::{body::{Body, MessageBody, ResponseBody}, error::ErrorUnauthorized};
use actix_service::{Service, Transform};
use futures::{future::LocalBoxFuture, ready};

use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{HttpResponse, Error};
use std::{
    marker::PhantomData,
};

use actix_utils::future::{ready, Ready};

// Potentially we can replace this with actix-web-httpauth
#[derive(Clone)]
pub struct AuthGuard;

impl<S> Transform<S, ServiceRequest> for AuthGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthGuardMiddleware {
            service,
        }))
    }
}

pub struct AuthGuardMiddleware<S> {
    service: S
}

impl<S> Service<ServiceRequest> for AuthGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let x = false;
        if x {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await.unwrap();
                Ok(res)
            })
        } else {
            Box::pin(async move {
                Ok(ServiceResponse::new(
                    req.into_parts().0,
                    HttpResponse::Unauthorized().finish(),
                ))
            })
       }
    }
  
}
