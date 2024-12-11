use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{
    database::DatabasePool,
    schema::user::{CreateUserRequest, User},
};

/// Inserts a new user into the database.
#[post("/user")]
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

#[get("/user")]

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
