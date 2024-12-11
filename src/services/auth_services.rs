use crate::schema::{
    auth::{Claims, LoginRequest},
    user::{CreateUserRequest, User},
};
use actix_web::{post, web, HttpResponse, Responder};
use bcrypt::verify;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
#[post("/register")]
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

    let query = r#"
        INSERT INTO users (id, username, email, password, is_active, roles)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    let result = sqlx::query(query)
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(true)
        .bind(&user.roles)
        .execute(db_pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Created().json(json!({
            "message": "User registered successfully",
            "data":user
        })),
        Err(_) => HttpResponse::Conflict().json(json!({
            "error": "Username or email already exists"
        })),
    }
}

#[post("/login")]
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
        let exp_time = chrono::Utc::now().timestamp() + 3600; // Token valid for 1 hour
        let claims = Claims {
            sub: user.id,
            exp: exp_time as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"secret_key"),
        )
        .unwrap();

        HttpResponse::Ok().json(json!({ "token": token }))
    } else {
        HttpResponse::Unauthorized().json(json!({ "error": "Invalid email or password" }))
    }
}
