use actix_url_shortener::generate_password_hash;
use actix_web::{
    delete, get, post, put,
    web::{self, Data, Json, Path},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{
    database::DatabasePool,
    schema::{
        url::ShortUrl,
        user::{CreateUserRequest, UpdateUserRequest, User},
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
pub async fn list_users(db_pool: web::Data<DatabasePool>) -> impl Responder {
    println!("run the function");
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

#[delete("/{user_id}")]
pub async fn delete_user_by_id(user_id: Path<String>, db: Data<DatabasePool>) -> impl Responder {
    match sqlx::query("DELETE FROM users WHERE id = ? ")
        .bind(user_id.into_inner())
        .execute(db.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // If rows are affected, return a success response
                HttpResponse::Ok().json("User deleted successfully")
            } else {
                // If no rows were affected (i.e., the URL ID was not found)
                HttpResponse::NotFound().json("User not found")
            }
        }
        Err(err) => {
            // Handle database errors
            eprintln!("Error deleting URL: {}", err);
            HttpResponse::InternalServerError().json("Failed to delete URL")
        }
    }
}

#[get("/{user_id}")]
pub async fn get_user_by_id(user_id: Path<String>, db: Data<DatabasePool>) -> impl Responder {
    match sqlx::query_as::<_, User>("SELECT * FROM user WHERE id = ?")
        .bind(user_id.into_inner())
        .fetch_one(db.as_ref())
        .await
    {
        Ok(user) => HttpResponse::Found().json(user),
        Err(e) => {
            eprint!("{:?}", e);
            HttpResponse::NotFound().json("Fiald to find user")
        }
    }
}

#[put("/{user_id}")]
pub async fn update_user_by_id(
    user_id: Path<String>,
    updated_user: Json<UpdateUserRequest>,
    db: Data<DatabasePool>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let updated_user = updated_user.into_inner();

    let mut query = String::from("UPDATE users SET ");
    let mut params = vec![];

    if let Some(username) = updated_user.username {
        query.push_str("username = ?, ");
        params.push(username);
    }
    if let Some(email) = updated_user.email {
        query.push_str("email = ?, ");
        params.push(email);
    }
    if let Some(password) = updated_user.password {
        query.push_str("password = ?, ");
        let hashed_password = generate_password_hash(&password).unwrap();
        params.push(hashed_password);
    }

    // Remove the trailing comma and space from the query
    if query.ends_with(", ") {
        query.pop();
        query.pop();
    }

    // Add the WHERE clause to target the correct user by ID
    query.push_str(" WHERE id = ?");
    params.push(user_id);

    let mut query_builder = sqlx::query(&query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    match query_builder.execute(db.as_ref()).await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json("User updated successfully")
            } else {
                HttpResponse::NotFound().json("User not found")
            }
        }
        Err(err) => {
            eprintln!("Error updating user: {}", err);
            HttpResponse::InternalServerError().json("Failed to update user")
        }
    }
}
