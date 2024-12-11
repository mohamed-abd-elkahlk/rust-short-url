use actix_web::Error;

pub mod url;
pub mod user;

pub type AppResult<T> = Result<T, Error>;
