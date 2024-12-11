use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use database::init_db;
use dotenv::dotenv;
use services::{
    url_services::{create_short_url, delete_url, list_user_urls, update_url},
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
                .service(
                    web::scope("/urls")
                        .service(create_short_url) // Route for creating short URLs
                        .service(list_user_urls) // Route for listing URLs for a user
                        .service(update_url) // Route for updating a URL
                        .service(delete_url), // Route for deleting a URL
                )
                // Group user-related routes under `/users`
                .service(
                    web::scope("/users")
                        .service(create_user) // Route for creating a user
                        .service(list_users), // Route for listing users
                )
        }, // Register the list_user_urls handler
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
