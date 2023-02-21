pub mod request;

use std::io::Write;
use std::sync::Arc;
use actix_http::HttpMessage;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web::middleware::{DefaultHeaders, Logger};
use log::LevelFilter;
use env_logger::Builder;
use env_logger::fmt::Color;
use crate::request::RequestHandler;
use chrono::Local;
use lazy_static::lazy_static;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DatabaseTransaction, Statement, TransactionTrait};



lazy_static! {
    pub static ref DB: DatabaseConnection  = futures::executor::block_on(Database::connect("postgres://root:root@localhost:5432/orca".to_string())).expect("error");
}



/// init_logger - function will initialize log Handler for the application
pub(crate) fn init_logger() {
    Builder::new()
        .format(|buf, record| {
            let mut timestamp_style = buf.style();
            timestamp_style.set_color(Color::Magenta);

            let mut level_style = buf.style();
            level_style.set_color(Color::Red);
            writeln!(buf,
                "[{} {}] {} >>> {}",
                timestamp_style.value(Local::now().format("%d-%m-%Y %H:%M:%S")),
                level_style.value(record.level()),
                record.module_path_static().unwrap_or(""),
                record.args()
            )
        })
        .filter_level(LevelFilter::Debug)
        .init();
    // env_logger::init();
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello(_req: HttpRequest) -> impl Responder {
    let ex = _req.extensions();
    let trx = ex.get::<DatabaseTransaction>().unwrap();
    trx.execute(Statement::from_string(
        DatabaseBackend::Postgres,
        "INSERT INTO checker (name, age) VALUES ('Vasanth', 2);".to_owned(),
    )).await.expect("error");

    trx.commit().await.expect("error");
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(&DB))
            .wrap(DefaultHeaders::new().add(("X-Version", "0.1")))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(RequestHandler::default())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}