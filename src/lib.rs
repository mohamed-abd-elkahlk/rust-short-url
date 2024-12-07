use uuid::Uuid;

/// Function to generate a default UUID for the `id` field
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}
