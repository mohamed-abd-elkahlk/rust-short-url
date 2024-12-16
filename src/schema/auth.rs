use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // Cow can be &str or String
    pub roles: String, // Cow can be &str or String
    pub exp: usize,
}

impl Claims {
    pub fn new(sub: String, roles: String) -> Self {
        let exp_time = chrono::Utc::now().timestamp() + 3600; // Token valid for 1 hour
        Self {
            exp: exp_time as usize,
            roles, // Borrow the roles as Cow<'a, str>
            sub,   // Borrow the sub as Cow<'a, str>
        }
    }
}
