use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

use axum_extra::extract::PrivateCookieJar;
use cookie::{time::Duration, Cookie};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};

use crate::errors::{LinksError, Result};
use crate::handlers::user::CreateUser;

const USER_COOKIE: &str = "links_user";
const LOGIN_DURATION: i64 = 60; //measured in minutes

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub superuser: bool,
}

impl User {
    fn hash_password(password: &str) -> Result<String> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        Ok(argon.hash_password(password.as_bytes(), &salt)?.to_string())
    }

    pub async fn new(txn: &mut Transaction<'_, Postgres>, user: CreateUser) -> Result<Self> {
        let hash = Self::hash_password(&user.password)?;
        let res = sqlx::query!(
            "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id",
            user.username,
            hash
        )
        .fetch_one(&mut **txn)
        .await?;

        Ok(Self {
            id: res.id,
            username: user.username,
            superuser: false,
        })
    }

    pub async fn get(txn: &mut Transaction<'_, Postgres>, id: i32) -> Result<Self> {
        let user = sqlx::query_as!(
            Self,
            "SELECT id, username, superuser FROM users WHERE id = $1",
            id
        )
        .fetch_one(&mut **txn)
        .await?;
        Ok(user)
    }

    pub async fn get_by_username(
        txn: &mut Transaction<'_, Postgres>,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let user = sqlx::query!(
            "SELECT id, username, password, superuser FROM users WHERE username = $1",
            username
        )
        .fetch_one(&mut **txn)
        .await?;
        let argon = Argon2::default();
        let hash = PasswordHash::new(&user.password).expect("invalid password hash in db");
        let correct = argon.verify_password(password.as_bytes(), &hash).is_ok();

        if !correct {
            Err(sqlx::Error::RowNotFound.into())
        } else {
            Ok(Self {
                id: user.id,
                username: user.username,
                superuser: user.superuser,
            })
        }
    }

    pub fn set_login_cookie(&self, jar: PrivateCookieJar) -> PrivateCookieJar {
        jar.add(
            Cookie::build((USER_COOKIE, self.id.to_string()))
                .max_age(Duration::minutes(LOGIN_DURATION)),
        )
    }
    pub fn remove_login_cookie(&self, jar: PrivateCookieJar) -> PrivateCookieJar {
        jar.remove(USER_COOKIE)
    }

    pub async fn validate_login_cookie(
        jar: PrivateCookieJar,
        txn: &mut Transaction<'_, Postgres>,
    ) -> Result<Self> {
        let Ok(user_id) = jar
            .get(USER_COOKIE)
            .ok_or(LinksError::Unauthorized)?
            .value()
            .parse()
        else {
            return Err(LinksError::Unauthorized);
        };
        let user = Self::get(txn, user_id).await?;
        Ok(user)
    }
    pub async fn update_password(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        old: &str,
        new: &str,
    ) -> Result<()> {
        if Self::get_by_username(txn, &self.username, old)
            .await
            .is_err()
        {
            //maybe this error should be different
            Err(LinksError::Unauthorized)
        } else {
            let hash = Self::hash_password(new)?;
            sqlx::query!(
                "UPDATE users SET password = $1 WHERE id = $2",
                hash,
                self.id,
            )
            .execute(&mut **txn)
            .await?;
            Ok(())
        }
    }
    pub async fn delete_all_users(txn: &mut Transaction<'_, Postgres>) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE permanent=false")
            .execute(&mut **txn)
            .await?;
        Ok(())
    }
}
