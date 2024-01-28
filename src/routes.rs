use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{
    handlers::{user::{add_user, get_user, login, logout, change_password, delete_user}, link::index},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(add_user))
        .route("/users/:id", get(get_user).delete(delete_user))
        .route("/users/login", post(login))
        .route("/users/logout", post(logout))
        .route("/users/password",put(change_password))
        .route("/", get(index))
}
