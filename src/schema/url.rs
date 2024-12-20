use actix_url_shortener::generate_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Represents a shortened URL and its metadata.
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ShortUrl {
    #[serde(default = "generate_uuid")]
    pub id: String,

    pub original_url: String,

    pub short_code: String,

    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if it's None
    pub expiration: Option<DateTime<Utc>>,

    pub click_count: u64,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if it's None
    pub user_id: Option<String>, // Added user_id to the struct
}

impl Default for ShortUrl {
    fn default() -> Self {
        Self {
            id: generate_uuid(), // Call generate_uuid for a new unique ID
            original_url: String::new(),
            short_code: String::new(),
            created_at: Utc::now(),
            expiration: None,
            click_count: 0,
            user_id: None, // Added user_id to the default implementation
        }
    }
}

impl ShortUrl {
    /// Check if the URL is expired.
    pub fn is_expired(&self) -> bool {
        self.expiration.map(|exp| Utc::now() > exp).unwrap_or(false)
    }
}

/// Request payload to create a new short URL.
#[derive(Deserialize)]
pub struct CreateUrlRequest {
    #[serde(rename = "originalUrl")]
    pub original_url: String,
}

#[derive(Deserialize)]
pub struct UpdateUrlRequest {
    pub original_url: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
}
