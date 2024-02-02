use askama::Template;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{extract::State, Form};
use axum_extra::extract::PrivateCookieJar;
use serde::de::Error;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::auth::AuthenticatedUser;
use crate::errors::{LinksError, Result};
use crate::models::user::User;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

//TODO make this take a token thingy
/*pub async fn add_user(
    pool: State<Pool<Postgres>>,
    jar: PrivateCookieJar,
    Form(user): Form<CreateUser>,
) -> Result<(PrivateCookieJar, Json<i32>)> {
    //TODO: make password requirements. maybe add some more secure things cus why not
    let mut txn = pool.begin().await?;
    let user = User::new(&mut txn, user).await?;
    let jar = user.set_login_cookie(jar);
    txn.commit().await?;
    Ok((jar, Json(user.id)))
}*/

#[derive(Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

pub async fn login(
    pool: State<Pool<Postgres>>,
    jar: PrivateCookieJar,
    Form(user): Form<LoginUser>,
) -> Result<(PrivateCookieJar, Redirect)> {
    let mut txn = pool.begin().await?;
    let user = User::get_by_username(&mut txn, &user.username, &user.password).await?;
    let jar = user.set_login_cookie(jar);
    txn.commit().await?;
    Ok((jar, Redirect::to("/admin")))
}

pub async fn logout(
    AuthenticatedUser(user): AuthenticatedUser,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect)> {
    let jar = user.remove_login_cookie(jar);
    Ok((jar, Redirect::to("/")))
}
#[derive(Deserialize)]
pub struct ChangePassword {
    pub old: String,
    pub new: String,
    pub confirm: String,
}

pub async fn change_password(
    pool: State<Pool<Postgres>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Form(change_password): Form<ChangePassword>,
) -> Result<Redirect> {
    if change_password.new != change_password.confirm {
        todo!()
    }
    let mut txn = pool.begin().await?;
    user.update_password(&mut txn, &change_password.old, &change_password.new)
        .await?;
    txn.commit().await?;
    Ok(Redirect::to("/admin"))
}
//TODO this should check if they superuser
pub async fn delete_all_users(
    user: AuthenticatedUser,
    pool: State<Pool<Postgres>>,
) -> Result<Redirect> {
    match user.0.superuser {
        false => Err(LinksError::Unauthorized),
        true => {
            let mut txn = pool.begin().await?;
            User::delete_all_users(&mut txn).await?;
            txn.commit().await?;
            Ok(Redirect::to("/admin/management"))
        }
    }
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    error: Option<String>,
}

pub async fn get_login_form() -> LoginTemplate {
    LoginTemplate { error: None }
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct ChangePasswordTemplate;

pub async fn get_password_form(_user: AuthenticatedUser) -> ChangePasswordTemplate {
    ChangePasswordTemplate
}

#[derive(Template)]
#[template(path = "management.html")]
pub struct ManagementTemplate;

pub async fn get_management_page(user: AuthenticatedUser) -> Result<ManagementTemplate> {
    match user.0.superuser {
        true => Ok(ManagementTemplate),
        false => Err(LinksError::Unauthorized),
    }
}
