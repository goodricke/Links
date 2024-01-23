use axum::extract::Path;
use axum::{extract::State, Form, Json};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::auth::AuthenticatedUser;
use crate::errors::Result;
use crate::models::user::User;

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

pub async fn add_user(
    pool: State<Pool<Postgres>>,
    jar: PrivateCookieJar,
    Form(user): Form<CreateUser>,
) -> Result<(PrivateCookieJar, Json<i32>)> {
    //TODO: implement email validation
    //TODO: make password requirements. maybe add some more secure things cus why not
    let mut txn = pool.begin().await?;
    let user = User::new(&mut txn, user).await?;
    let jar = user.set_login_cookie(jar);
    txn.commit().await?;
    Ok((jar, Json(user.id)))
}

//this should be authed
pub async fn get_user(Path(user_id): Path<i32>, pool: State<Pool<Postgres>>) -> Result<Json<User>> {
    let mut txn = pool.begin().await?;
    let user = User::get(&mut txn, user_id).await?;
    txn.commit().await?;
    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

pub async fn login(
    pool: State<Pool<Postgres>>,
    jar: PrivateCookieJar,
    Form(user): Form<LoginUser>,
) -> Result<(PrivateCookieJar, Json<User>)> {
    let mut txn = pool.begin().await?;
    let user = User::get_by_email(&mut txn, &user.email, &user.password).await?;
    let jar = user.set_login_cookie(jar);
    txn.commit().await?;
    Ok((jar, Json(user)))
}

pub async fn logout(
    AuthenticatedUser(user): AuthenticatedUser,
    jar: PrivateCookieJar,
) -> Result<PrivateCookieJar> {
    let jar = user.remove_login_cookie(jar);
    Ok(jar)
}
#[derive(Deserialize)]
pub struct ChangePassword {
    pub old: String,
    pub new: String,
}

pub async fn change_password(
    pool: State<Pool<Postgres>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(change_password): Json<ChangePassword>,
) -> Result<()> {
    let mut txn = pool.begin().await?;
    user.update_password(&mut txn, &change_password.old, &change_password.new)
        .await?;
    Ok(())
}
// this should be an auth for admin
pub async fn delete_user(
    Path(user_id): Path<i32>,
    pool: State<Pool<Postgres>>,
) -> Result<()> {
    let mut txn = pool.begin().await?;
    User::delete_user(&mut txn, user_id).await?;
    txn.commit().await?;
    Ok(())
    
}
