use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use database::init_db;
use dotenv::dotenv;
use services::{
    url_services::{
        create_short_url, delete_url, get_short_url_by_id, redirect_to_original, update_url,
    },
    user_services::{
        create_user, delete_user_by_id, get_user_by_id, list_user_urls, list_users,
        update_user_by_id,
    },
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
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .service(redirect_to_original)
            .service(
                web::scope("/urls")
                    .service(create_short_url) // Route for creating short URLs
                    .service(update_url) // Route for updating a URL
                    .service(delete_url) // Route for deleting a URL
                    .service(get_short_url_by_id), // Route for get a URL By Id
            )
            // Group user-related routes under `/users`
            .service(
                web::scope("/users")
                    .service(create_user) // Route for creating a user
                    .service(list_user_urls) // Route for listing URLs for a user
                    .service(list_users) // Route for listing users
                    .service(delete_user_by_id) // Route for delete user by id
                    .service(get_user_by_id) // Route for get user by id
                    .service(update_user_by_id), // Route for update user by id
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
