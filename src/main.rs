use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use database::init_db;
use dotenv::dotenv;
use std::{io, sync::Mutex};
mod database;
mod schema;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("hellow world")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let db = init_db().await.expect("msg");
    HttpServer::new(move || App::new().app_data(Mutex::new(db.clone())).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
