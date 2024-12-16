use crate::schema::auth::Claims;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, middleware::Next, Error};
use jsonwebtoken::{decode, DecodingKey, Validation};

/// Middleware to verify the JWT and check for required roles
pub async fn verify_jwt_and_role(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
    required_roles: &str,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if let Some(cookie) = req.cookie("access_token") {
        let token = cookie.value().to_string();
        let secret_key = std::env::var("SECRET").unwrap_or_else(|_| "default_secret".to_string());

        // Decode and validate the token
        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret_key.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => {
                if token_data.claims.roles == required_roles {
                    // At least one role matches, continue the request
                    next.call(req).await
                } else {
                    // No roles match, reject the request
                    Err(ErrorUnauthorized(format!(
                        "Access denied. Required roles: {:?}, but found: {:?}",
                        required_roles, token_data.claims.roles
                    )))
                }
            }
            Err(err) => {
                // Token is invalid or expired
                Err(ErrorUnauthorized(format!(
                    "Invalid or expired token:{}",
                    err
                )))
            }
        }
    } else {
        // No access token cookie found, reject the request
        Err(ErrorUnauthorized("Missing access token"))
    }
}
