use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use actix_http::header::{HeaderName, HeaderValue};
use actix_http::HttpMessage;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, web};
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header;
use futures_util::future::LocalBoxFuture;
use log::info;
use sea_orm::{DatabaseTransaction, DbConn, DbErr, TransactionTrait};
use crate::DB;


// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middlewares call method gets called with normal request.
#[derive(Debug, Clone, Default)]
pub struct RequestHandler;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for RequestHandler
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestHandlerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestHandlerMiddleware { service }))
    }
}

pub struct RequestHandlerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestHandlerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        info!("Starting the Request");

        let trx : DatabaseTransaction = futures::executor::block_on(DB.begin()).expect("error");
        req.extensions_mut().insert(trx);

        let fut = self.service.call(req);

        //
        // let res = DB.transaction(|trx| {
        //     Box::pin(async move {
        //         req.extensions_mut().insert(trx);
        //         let mut _response = fut.await;
        //         _response
        //     })
        // });
        // Box::pin(async move {
        //     Ok(res.await.expect("error"))
        // })
        // return res;

        // Box::pin(async move {
        //
        //     ///////////////******** Try 1******///////////////
        //     /// this will be before request for any api in the application
        //     let tx = DB.begin().await.expect("error this will raise the panic error");
        //     req.extensions_mut().insert(tx);
        //
        //     let mut _response = fut.await;
        //
        //     /// this is where post request will go for the application
        //     ///
        //     tx.commit().await.expect("TODO: panic message");
        //
        //     ///////////////******** End 1******///////////////
        //
        //     ////////////******** Try 2 ******///////////////
        //     / this will be before request for any api in the application
        //
        //
        //
        //
        //
        //     info!("Completed Request after - {:?}", start_time.elapsed());
        //     _response
        // })


        Box::pin(async move {
            // let db_conn = req.extensions().get::<&DbConn>().unwrap().clone();
            ///////////////******** Try 1******///////////////
            /// this will be before request for any api in the application
            // let tx = DB.begin().await.expect("error this will raise the panic error");

            // let out = DB.transaction::<_, _, DbErr>(|txn| {
            //     Box::pin(async move {
            //         let mut res = fut.await;
            //         res
            //     })
            // })
            // .await;
            // let mut _response = out.expect("error");

            let mut _response = fut.await;

            /// this is where post request will go for the application
            ///
            // tx.commit().await.expect("TODO: panic message");

            ///////////////******** End 1******///////////////

            ////////////******** Try 2 ******///////////////
            // / this will be before request for any api in the application





            info!("Completed Request after - {:?}", start_time.elapsed());
            _response
        })

    }
}