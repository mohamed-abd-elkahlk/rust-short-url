use actix_url_shortener::generate_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
/// Represents a user in the system.
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    #[serde(default = "generate_uuid")]
    pub id: String,

    pub username: String,

    pub email: String,

    pub password: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
    pub is_active: bool,

    pub roles: Json<Vec<String>>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: generate_uuid(),
            username: String::new(),
            email: String::new(),
            password: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: false,
            roles: Json(vec![]),
        }
    }
}
impl User {
    /// Hashes a password using bcrypt and updates the `password` field.
    pub fn set_password(&mut self, plain_password: &str) -> Result<&mut Self, bcrypt::BcryptError> {
        let hashed_password = bcrypt::hash(plain_password, bcrypt::DEFAULT_COST)?;
        self.password = hashed_password;
        Ok(self) // Return a mutable reference to the instance
    }

    /// Verifies that a provided password matches the user's stored (hashed) password.
    pub fn verify_password(&self, plain_password: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(plain_password, &self.password)
    }

    /// Updates the `updated_at` timestamp to the current time.
    pub fn touch(&mut self) -> &mut Self {
        self.updated_at = Utc::now();
        self // Return a mutable reference to the instance
    }

    /// Set the default role for the user if no roles are specified.
    pub fn set_roles(&mut self) -> &mut Self {
        if self.roles.is_empty() {
            self.roles.push("user".to_string());
        }
        self // Return a mutable reference to the instance
    }
}
#[derive(Deserialize)]

pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}
