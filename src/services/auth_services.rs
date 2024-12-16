use std::env;

use crate::schema::{
    auth::{Claims, LoginRequest},
    user::{CreateUserRequest, User},
};
use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse, Responder};
use bcrypt::verify;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
#[post("/sign-up")]
pub async fn register_user(
    db_pool: web::Data<sqlx::MySqlPool>,
    req_body: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Store request in a variable to ensure its lifetime
    let req = req_body.into_inner();

    // Store user in a variable to extend its lifetime
    let mut user = User::default();
    user.set_password(&req.password).unwrap();
    user.email = req.email; // Use `clone` to ensure a valid reference
    user.username = req.username; // Use `clone` to ensure a valid reference
    user.is_active = true;
    let query = r#"
        INSERT INTO users (id, username, email, password, is_active, roles)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    let result = sqlx::query(query)
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.is_active)
        .bind(&user.roles)
        .execute(db_pool.get_ref())
        .await;

    match result {
        Ok(_) => {
            let secret_key = env::var("SECRET").expect("msg");
            let claims = Claims::new(user.id.clone(), user.roles.first().unwrap().to_string());
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret_key.as_bytes()),
            )
            .unwrap();

            HttpResponse::Created()
                .cookie(
                    Cookie::build("access_token", token)
                        .path("/")
                        .http_only(true)
                        .finish(),
                )
                .json(json!({
                    "message": "User registered successfully",
                    "data":user
                }))
        }
        Err(_) => HttpResponse::Conflict().json(json!({
            "error": "Username or email already exists"
        })),
    }
}

#[post("/sign-in")]
pub async fn login_user(
    db_pool: web::Data<sqlx::MySqlPool>,
    req_body: web::Json<LoginRequest>,
) -> impl Responder {
    let req = req_body.into_inner();
    let user: User = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(req.email)
        .fetch_one(db_pool.get_ref())
        .await
    {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(json!({ "error": "Invalid email or password" }))
        }
    };

    if verify(req.password, &user.password).unwrap_or(false) {
        let claims = Claims::new(user.id, user.roles.first().unwrap().to_string());

        let secret_key = env::var("SECRET").expect("msg");

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_bytes()),
        )
        .unwrap();

        HttpResponse::Ok()
            .cookie(
                Cookie::build("access_token", &token)
                    .path("/")
                    .http_only(true)
                    .finish(),
            )
            .json(json!({ "token": token }))
    } else {
        HttpResponse::Unauthorized().json(json!({ "error": "Invalid email or password" }))
    }
}
