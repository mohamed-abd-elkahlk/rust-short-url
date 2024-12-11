use crate::{
    database::DatabasePool,
    schema::url::{CreateUrlRequest, ShortUrl},
};
use actix_url_shortener::generate_short_code_from_url;
use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

/// Create Url
#[post("/")]
pub async fn create_short_url(
    body: Json<CreateUrlRequest>,
    db: Data<DatabasePool>,
) -> impl Responder {
    // Extract the data from the incoming request body
    let CreateUrlRequest {
        original_url,
        user_id,
    } = body.into_inner();
    let short_code = generate_short_code_from_url(&original_url, 10);

    let short_url = ShortUrl::default();
    // Create a new ShortUrl in the database
    let query = r#"
    INSERT INTO short_urls (id, original_url, short_code, created_at, user_id)
    VALUES (?, ?, ?, ?, ?)
    "#;

    match sqlx::query(query)
        .bind(short_url.id)
        .bind(original_url)
        .bind(short_code)
        .bind(short_url.created_at)
        .bind(user_id)
        .execute(db.as_ref()) // Execute the query with the DB pool
        .await
    {
        Ok(_) => HttpResponse::Created().json("Short URL created successfully"), // Return success
        Err(err) => {
            eprintln!("❌ Error creating short URL: {}", err);
            HttpResponse::InternalServerError().json("Internal Server Error") // Return error if failed
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
            println!("{:?}", urls);
            HttpResponse::Ok().json(urls) // Return the list of URLs as JSON
        }
        Err(err) => {
            eprintln!("Database query failed: {}", err); // Log the error
            HttpResponse::InternalServerError().json("Internal Server Error") // Return a 500 response
        }
    }
}
