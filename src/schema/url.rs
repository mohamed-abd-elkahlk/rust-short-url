use actix_url_shortener::generate_uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortUrl {
    #[serde(default = "generate_uuid")] // Serde will call this function when no `id` is provided
    pub id: String, // Unique identifier for the shortened URL (could be a hash)
    pub original_url: String, // The full URL the short link redirects to
    pub short_code: String,   // The short identifier for the URL (e.g., "abc123")
    #[serde(default = "Utc::now")] // Sets default for `created_at` as the current time
    pub created_at: DateTime<Utc>, // Timestamp when the URL was created
    #[serde(default)] // Sets default as None for Option type
    pub expiration: Option<DateTime<Utc>>, // Optional expiration date for the URL
    #[serde(default)] // Defaults to 0 for unsigned integer
    pub click_count: u64, // Number of times the short URL was clicked
}

impl ShortUrl {
    /// Increment the click count
    pub fn increment_click_count(&mut self) {
        self.click_count += 1;
    }
    /// Check if the URL is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            Utc::now() > expiration
        } else {
            false
        }
    }
}

impl Default for ShortUrl {
    fn default() -> Self {
        Self {
            id: generate_uuid(),
            original_url: String::new(),
            short_code: String::new(),
            created_at: Utc::now(),
            expiration: None,
            click_count: 0,
        }
    }
}
