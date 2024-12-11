use actix_web::{
    get, post, web,
    web::{Data, Path},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{
    database::DatabasePool,
    schema::{
        url::ShortUrl,
        user::{CreateUserRequest, User},
    },
};

/// Inserts a new user into the database.
#[post("/")]
pub async fn create_user(
    db_pool: web::Data<DatabasePool>,
    req_body: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Extract request data
    let req = req_body.into_inner();
    let mut user = User::default();
    user.set_password(&req.password).unwrap().set_roles();
    let roles_as_json = serde_json::to_string(&user.roles).unwrap();

    // Insert user into the database
    let query = r#"
        INSERT INTO users (id, username, email, password, created_at, updated_at, is_active, roles)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;
    let result = sqlx::query(query)
        .bind(&user.id)
        .bind(req.username)
        .bind(req.email)
        .bind(&user.password)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .bind(true)
        .bind(roles_as_json)
        .execute(db_pool.get_ref())
        .await;

    // Handle SQL insert result
    match result {
        Ok(_) => HttpResponse::Created().json(json!({
            "message": "User created successfully",
            "userId":user
        })),
        Err(sqlx::Error::Database(e)) if e.constraint().is_some() => {
            HttpResponse::Conflict().json(json!({
                "error": "Username or email already exists"
            }))
        }
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create user"

            }))
        }
    }
}

#[get("/")]

async fn list_users(db_pool: web::Data<DatabasePool>) -> impl Responder {
    match sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(db_pool.get_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            eprintln!("Database query error: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to retrieve users")
        }
    }
}

#[get("/{user_id}/urls")]
pub async fn list_user_urls(path: Path<String>, db: Data<DatabasePool>) -> impl Responder {
    let user_id = path.into_inner();
    match sqlx::query_as::<_, ShortUrl>(
        r#"
        SELECT * FROM short_urls WHERE user_id = ?
        "#,
    )
    .bind(user_id) // Bind the user_id to the query
    .fetch_all(db.as_ref())
    .await
    {
        Ok(urls) => {
            // Log the result (optional)
            HttpResponse::Ok().json(urls) // Return the list of URLs as JSON
        }
        Err(err) => {
            eprintln!("Database query failed: {}", err); // Log the error
            HttpResponse::InternalServerError().json("Internal Server Error") // Return a 500 response
        }
    }
}
