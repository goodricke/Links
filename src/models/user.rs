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
    pub email: String,
}

impl User {
    pub async fn new(txn: &mut Transaction<'_, Postgres>, user: CreateUser) -> Result<Self> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(user.password.as_bytes(), &salt)?
            .to_string();
        let res = sqlx::query!(
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id",
            user.email,
            hash
        )
        .fetch_one(&mut **txn)
        .await?;

        Ok(Self {
            id: res.id,
            email: user.email,
        })
    }

    pub async fn get(txn: &mut Transaction<'_, Postgres>, id: i32) -> Result<Self> {
        let user = sqlx::query_as!(Self, "SELECT id, email FROM users WHERE id = $1", id)
            .fetch_one(&mut **txn)
            .await?;
        Ok(user)
    }

    pub async fn get_by_email(
        txn: &mut Transaction<'_, Postgres>,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let user = sqlx::query!(
            "SELECT id, email, password FROM users WHERE email = $1",
            email
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
                email: user.email,
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
        if Self::get_by_email(txn, &self.email, old).await.is_err() {
            //maybe this error should be different
            Err(LinksError::Unauthorized)
        } else {
            sqlx::query!("UPDATE users SET password = $1 WHERE id = $2", new, self.id,)
                .execute(&mut **txn)
                .await?;
            Ok(())
        }
    }
    pub async fn delete_user(txn: &mut Transaction<'_, Postgres>, id:i32) -> Result<()>{
        let rows_affected = sqlx::query!("DELETE FROM users WHERE id = $1", id).execute(&mut **txn).await?.rows_affected();
        if rows_affected != 1 {
            //consider creating a 404 type
            Err(sqlx::Error::RowNotFound.into())
        }else {
            Ok(())
        }
    }
}
