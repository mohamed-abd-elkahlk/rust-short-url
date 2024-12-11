use crate::{
    database::DatabasePool,
    schema::url::{CreateUrlRequest, ShortUrl, UpdateUrlRequest},
};
use actix_url_shortener::generate_short_code_from_url;
use actix_web::{
    delete, get, post, put,
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

#[get("/{user_id}")]
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

#[put("/{url_id}/update")]
pub async fn update_url(
    url_id: Path<String>,
    update_data: Json<UpdateUrlRequest>,
    db_pool: Data<DatabasePool>,
) -> impl Responder {
    let url_id = url_id.into_inner();
    let update_data = update_data.into_inner();

    // Build the SQL query dynamically based on the fields that are provided
    let mut query = String::from("UPDATE short_urls SET ");
    let mut params = vec![];

    // Check if original_url is provided for update
    if let Some(original_url) = update_data.original_url {
        // Generate new short code if original_url is updated
        let short_code = generate_short_code_from_url(&original_url, 10); // Length of short_code (e.g., 10)
        query.push_str("original_url = ?, short_code = ?, ");
        params.push(original_url);
        params.push(short_code);
    }

    // Update expiration if provided
    if let Some(expiration) = update_data.expiration {
        query.push_str("expiration = ?, ");
        params.push(expiration.to_rfc3339()); // Convert to RFC 3339 string format
    }
    // Remove the trailing comma and space from the query
    query.pop();
    query.pop();

    // Add the WHERE clause to target the correct URL by ID
    query.push_str(" WHERE id = ?");
    params.push(url_id);

    // Execute the query, binding each parameter individually
    let mut query = sqlx::query(&query);
    for param in params {
        query = query.bind(param);
    }

    // Execute the query
    match query.execute(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json("URL updated successfully"),
        Err(err) => {
            eprintln!("Error updating URL: {}", err);
            HttpResponse::InternalServerError().json("Failed to update URL")
        }
    }
}

#[delete("/{url_id}")]
pub async fn delete_url(
    url_id: Path<String>,        // The URL ID to be deleted
    db_pool: Data<DatabasePool>, // The database pool
) -> impl Responder {
    let url_id = url_id.into_inner(); // Extract the URL ID from the path

    // Construct the DELETE query
    let query = r#"
        DELETE FROM short_urls 
        WHERE id = ?
    "#;

    // Execute the query
    match sqlx::query(query)
        .bind(url_id) // Bind the URL ID parameter
        .execute(db_pool.get_ref()) // Execute the query using the DB pool
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // If rows are affected, return a success response
                HttpResponse::Ok().json("URL deleted successfully")
            } else {
                // If no rows were affected (i.e., the URL ID was not found)
                HttpResponse::NotFound().json("URL not found")
            }
        }
        Err(err) => {
            // Handle database errors
            eprintln!("Error deleting URL: {}", err);
            HttpResponse::InternalServerError().json("Failed to delete URL")
        }
    }
}
