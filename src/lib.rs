use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

pub mod auth;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod routes;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub key: Key,
}
