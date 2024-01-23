use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::{add_user, get_user, login, logout},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(add_user))
        .route("/users/:id", get(get_user))
        .route("/users/login", post(login))
        .route("/users/logout", post(logout))
}
