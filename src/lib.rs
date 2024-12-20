use bcrypt::BcryptError;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Function to generate a default UUID for the `id` field
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_password_hash(plain_password: &str) -> Result<String, BcryptError> {
    let hashed_password = bcrypt::hash(plain_password, bcrypt::DEFAULT_COST)?;
    Ok(hashed_password)
}

/// Generates a short code based on the hash of the original URL.
pub fn generate_short_code_from_url(original_url: &str, length: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(original_url); // Feed the URL into the hash function
    let result = hasher.finalize(); // Finalize the hash

    // Convert the hash result into a hex string and take the first `length` characters
    let short_code = hex::encode(result);
    short_code.chars().take(length).collect()
}
