use actix_url_shortener::generate_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(default = "generate_uuid")] // Automatically generate a UUID for the user
    pub id: String, // Unique identifier for the user
    pub username: String, // The user's unique username
    pub email: String,    // The user's email address
    pub password: String, // The hashed password (never store raw passwords)
    #[serde(default = "Utc::now")] // Default to the current timestamp for created_at
    pub created_at: DateTime<Utc>, // The time when the user was created
    #[serde(default = "Utc::now")] // Default to the current timestamp for updated_at
    pub updated_at: DateTime<Utc>, // The last time the user information was updated
    #[serde(default)] // Default to false for boolean
    pub is_active: bool, // Whether the user's account is active
    #[serde(default)] // Default to an empty list
    pub roles: Vec<String>, // List of roles/permissions the user has
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
            roles: Vec::new(),
        }
    }
}
