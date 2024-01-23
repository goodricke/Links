use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum LinksError {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal server error")]
    InternalServerError(String),
    #[error("password hash error")]
    PasswordHashError(#[from] argon2::password_hash::Error),
}

impl IntoResponse for LinksError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self}");
        match self {
            LinksError::DatabaseError(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "not found").into_response()
            }
            LinksError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response()
            }
            LinksError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
            LinksError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            LinksError::PasswordHashError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, LinksError>;
