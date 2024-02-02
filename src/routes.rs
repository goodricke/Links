use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::{
        link::{admin, create_link, delete_link, index, update_link},
        user::{
            change_password, delete_all_users, get_login_form, get_management_page,
            get_password_form, login, logout,
        },
    },
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/admin", get(admin))
        .route("/admin/managemenet", get(get_management_page))
        .route("/admin/login", get(get_login_form).post(login))
        .route("/admin/logout", get(logout))
        .route(
            "/admin/password",
            get(get_password_form).post(change_password),
        )
        .route("/admin/:id", post(update_link))
        .route("/admin/:id/delete", post(delete_link))
        .route("/admin/create", post(create_link))
        .route("/admin/delete_all_users", post(delete_all_users))
}
