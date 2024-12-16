use crate::{
    database::DatabasePool,
    schema::{
        auth::Claims,
        url::{CreateUrlRequest, ShortUrl, UpdateUrlRequest},
    },
};
use actix_url_shortener::generate_short_code_from_url;
use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use sqlx::Row; // Import the Row trait to use `get`

#[post("/")]
pub async fn create_short_url(
    req: HttpRequest,
    body: Json<CreateUrlRequest>,
    db: Data<DatabasePool>,
) -> impl Responder {
    // Extract Claims from the request extensions
    let req = req.extensions();
    let claims = req.get::<Claims>();

    if let Some(claims) = claims {
        // Extract the data from the incoming request body
        let CreateUrlRequest { original_url } = body.into_inner();
        let short_code = generate_short_code_from_url(&original_url, 10);

        let short_url = ShortUrl::default();
        let user_id = claims.sub.clone(); // Extract user_id from the 'sub' field of claims

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
                eprintln!(" Error creating short URL: {}", err);
                HttpResponse::InternalServerError().json("Internal Server Error")
                // Return error if failed
            }
        }
    } else {
        HttpResponse::Unauthorized().json("Unauthorized")
    }
}

/// Update Url
#[put("/{url_id}/update")]
pub async fn update_url(
    req: HttpRequest,
    url_id: Path<String>,                // Extract URL ID from the path
    update_data: Json<UpdateUrlRequest>, // The incoming request body containing the update data
    db_pool: Data<DatabasePool>,         // Shared database connection pool
) -> impl Responder {
    let url_id = url_id.into_inner(); // Extract the URL ID from the path
    let update_data = update_data.into_inner(); // Extract the update data from the request body

    let req = req.extensions(); // Extract Claims from the request extensions
    let claims = req.get::<Claims>();

    // Check if the user_id from claims matches the user_id for the URL in the database
    let user_check_query = "SELECT user_id FROM short_urls WHERE id = ?";
    match sqlx::query(user_check_query)
        .bind(&url_id) // Bind the URL ID
        .fetch_one(db_pool.get_ref()) // Execute the query
        .await
    {
        Ok(record) => {
            let db_user_id: String = record.get::<String, _>("user_id"); // Extract user_id from the query result
            if let Some(claims) = claims {
                if db_user_id != claims.sub {
                    return HttpResponse::Unauthorized().json("Unauthorized to update this URL");
                    // User not authorized to update
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching URL user_id: {}", err); // Log the error to the console
            return HttpResponse::InternalServerError().json("Internal Server Error");
            // Return a 500 status if the query fails
        }
    }

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
        Ok(_) => HttpResponse::Ok().json("URL updated successfully"), // Return a 200 OK status if successful
        Err(err) => {
            eprintln!("Error updating URL: {}", err); // Log the error to the console
            HttpResponse::InternalServerError().json("Failed to update URL") // Return a 500 status if the query fails
        }
    }
}

/// Delete Url
#[delete("/{url_id}")]
pub async fn delete_url(
    req: HttpRequest,
    url_id: Path<String>,        // Extract the URL ID from the path
    db_pool: Data<DatabasePool>, // Shared database connection pool
) -> impl Responder {
    let url_id = url_id.into_inner(); // Extract the URL ID from the path

    // Check if the user_id from claims matches the user_id for the URL in the database
    let user_check_query = "SELECT user_id FROM short_urls WHERE id = ?";
    match sqlx::query(user_check_query)
        .bind(&url_id) // Bind the URL ID
        .fetch_one(db_pool.get_ref()) // Execute the query
        .await
    {
        Ok(record) => {
            let req = req.extensions(); // Extract Claims from the request extensions
            let claims = req.get::<Claims>();
            let db_user_id: String = record.get::<String, _>("user_id"); // Extract user_id from the query result
            if let Some(claims) = claims {
                if db_user_id != claims.sub {
                    return HttpResponse::Unauthorized().json("Unauthorized to update this URL");
                    // User not authorized to update
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching URL user_id: {}", err); // Log the error to the console
            return HttpResponse::InternalServerError().json("Internal Server Error");
            // Return a 500 status if the query fails
        }
    }

    // SQL query to delete the URL from the database
    let query = "DELETE FROM short_urls WHERE id = ?";

    match sqlx::query(query)
        .bind(url_id) // Bind the URL ID parameter
        .execute(db_pool.get_ref()) // Execute the query using the DB pool
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json("URL deleted successfully")
            } else {
                HttpResponse::NotFound().json("URL not found")
            }
        }
        Err(err) => {
            eprintln!("Error deleting URL: {}", err);
            HttpResponse::InternalServerError().json("Failed to delete URL")
        }
    }
}

/// Handle redirect from short URL to original URL.
#[get("/s/{short_code}")]
pub async fn redirect_to_original(
    short_code: Path<String>,    // Extract short code from the URL
    db_pool: Data<DatabasePool>, // Inject the database pool
) -> impl Responder {
    // Query the database for the short URL's corresponding original URL
    let short_url = sqlx::query_as::<_, ShortUrl>(
        r#"
        SELECT * FROM short_urls WHERE short_code = ?
        "#,
    )
    .bind(short_code.into_inner()) // Bind the short code to the query
    .fetch_optional(&**db_pool) // Fetch the URL, or return None if not found
    .await;

    match short_url {
        Ok(Some(url)) => {
            // Increment the click_count for the short URL in the database
            let update_result = sqlx::query(
                r#"
                UPDATE short_urls 
                SET click_count = click_count + 1 
                WHERE short_code = ?
                "#,
            )
            .bind(&url.short_code) // Bind the short code to the query
            .execute(&**db_pool)
            .await;

            if let Err(err) = update_result {
                // Log or handle the error if updating click count fails
                eprintln!("Failed to update click count: {:?}", err);
            }

            // Redirect the user to the original URL
            HttpResponse::Found()
                .append_header(("Location", url.original_url))
                .finish()
        }
        Ok(None) => {
            // Return 404 if the short URL does not exist in the database
            HttpResponse::NotFound().json("Short URL not found")
        }
        Err(_) => {
            // Handle database query error
            HttpResponse::InternalServerError().json("Internal Server Error")
        }
    }
}

/// Get URL by ID
#[get("/{url_id}")]
pub async fn get_short_url_by_id(
    req: HttpRequest,
    url_id: Path<String>,
    db_pool: Data<DatabasePool>,
) -> impl Responder {
    // Extract the claims from the request's extensions
    let req = req.extensions();
    let claims = match req.get::<Claims>() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().body("Missing or invalid JWT claims"),
    };

    let url_id = url_id.into_inner();

    // Query the database for the URL and its owner
    match sqlx::query_as::<_, ShortUrl>("SELECT * FROM short_urls WHERE id = ?")
        .bind(&url_id)
        .fetch_one(db_pool.as_ref())
        .await
    {
        Ok(url) => {
            // Check if the URL has an owner and if the user has access to it
            match &url.user_id {
                Some(user_id) if *user_id == claims.sub || claims.roles.contains("admin") => {
                    HttpResponse::Ok().json(url)
                }
                Some(_) => HttpResponse::Forbidden().body("You do not have access to this URL"), // Another user owns it
                None => HttpResponse::Forbidden().body("This URL does not have an owner"), // URL exists but has no owner
            }
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("URL not found"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Database error: {}", e)),
    }
}
