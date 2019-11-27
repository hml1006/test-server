use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};

use crate::statistic::*;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct ReqStat;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for ReqStat
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ReqStatMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ReqStatMiddleware { service })
    }
}

pub struct ReqStatMiddleware<S> {
    service: S,
}

impl<S, B> Service for ReqStatMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());
        // increase request number
        inc_req_num();

        Box::new(self.service.call(req).and_then(|res| {
            println!("Hi from response");

            //increase response number
            inc_resp_num();

            let status = res.status().as_u16();
            println!("Status: {}", status);

            // statistic response by status code
            match status {
                200 => inc_200_resp_num(),
                301 => inc_301_resp_num(),
                302 => inc_302_resp_num(),
                400 => inc_400_resp_num(),
                403 => inc_403_resp_num(),
                404 => inc_404_resp_num(),
                500 => inc_500_resp_num(),
                501 => inc_501_resp_num(),
                502 => inc_502_resp_num(),
                503 => inc_503_resp_num(),
                _ => ()
            }
            Ok(res)
        }))
    }
}