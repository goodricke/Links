use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::PrivateCookieJar;
use cookie::Key;

use crate::{errors::LinksError, AppState};

use crate::models::user::User;

pub struct AuthenticatedUser(pub User);

pub enum RedirectOrError {
    Error(LinksError),
    Redirect(Redirect),
}

impl IntoResponse for RedirectOrError {
    fn into_response(self) -> askama_axum::Response {
        match self {
            RedirectOrError::Error(e) => e.into_response(),
            RedirectOrError::Redirect(r) => r.into_response(),
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = RedirectOrError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut txn = state
            .pool
            .begin()
            .await
            .map_err(|e| RedirectOrError::Error(e.into()))?;
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .expect("private cookie jar cannot fail");
        let user = User::validate_login_cookie(jar, &mut txn).await;

        match user {
            Ok(user) => Ok(AuthenticatedUser(user)),
            Err(LinksError::DatabaseError(sqlx::Error::RowNotFound))
            | Err(LinksError::Unauthorized) => {
                Err(RedirectOrError::Redirect(Redirect::to("/admin/login")))
            }
            Err(e) => Err(RedirectOrError::Error(e)),
        }
    }
}
