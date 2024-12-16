use actix_web::{
    middleware::{from_fn, Logger},
    web::{self, Data},
    App, HttpServer,
};
use database::init_db;
use dotenv::dotenv;
use middleware::verify_jwt_and_role;
use services::{
    auth_services::{login_user, register_user},
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
mod middleware;
mod schema;
mod services;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = init_db().await.expect("Failed to initialize database");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .wrap(Logger::default()) // Logs requests automatically
            // Public route, no middleware
            .service(redirect_to_original)
            // Routes requiring 'user' role
            .service(
                web::scope("/urls")
                    .service(create_short_url)
                    .service(update_url)
                    .service(delete_url)
                    .service(get_short_url_by_id)
                    .wrap(from_fn(|req, next| verify_jwt_and_role(req, next, "user"))),
            )
            // Routes requiring 'admin' role
            .service(
                web::scope("/users")
                    .service(create_user)
                    .service(list_users)
                    .service(delete_user_by_id)
                    .service(get_user_by_id)
                    .service(delete_user_by_id)
                    .service(list_user_urls)
                    .service(update_user_by_id)
                    .wrap(from_fn(|req, next| verify_jwt_and_role(req, next, "admin"))),
            )
            .service(
                web::scope("/auth")
                    .service(login_user)
                    .service(register_user),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
