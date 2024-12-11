use actix_web::{web::Data, App, HttpServer};
use database::init_db;
use dotenv::dotenv;
use services::{
    url_services::{create_short_url, list_user_urls},
    user_services::{create_user, list_users},
};
use std::io;
mod database;
mod schema;
mod services;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let db = init_db().await.expect("msg");
    HttpServer::new(
        move || {
            App::new()
                .app_data(Data::new(db.clone()))
                .service(create_short_url) // Register the create_short_url handler
                .service(list_user_urls)
                .service(create_user)
                .service(list_users)
        }, // Register the list_user_urls handler
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
