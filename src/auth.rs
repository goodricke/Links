use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::PrivateCookieJar;
use cookie::Key;

use crate::{errors::LinksError, AppState};

use crate::models::user::User;

pub struct AuthenticatedUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = LinksError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut txn = state.pool.begin().await?;
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .expect("private cookie jar cannot fail");
        let user = User::validate_login_cookie(jar, &mut txn).await;

        match user {
            Ok(user) => Ok(AuthenticatedUser(user)),
            Err(LinksError::DatabaseError(sqlx::Error::RowNotFound)) => {
                Err(LinksError::Unauthorized)
            }
            Err(e) => Err(e),
        }
    }
}
