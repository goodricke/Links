use askama::Template;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{extract::State, Form};
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

//TODO make this take a token thingy
/*pub async fn add_user(
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
    let user = User::get_by_email(&mut txn, &user.username , &user.password).await?;
    let jar = user.set_login_cookie(jar);
    txn.commit().await?;
    Ok((jar, Redirect::to("/admin")))
}

pub async fn logout(
    AuthenticatedUser(user): AuthenticatedUser,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar,Redirect)> {
    let jar = user.remove_login_cookie(jar);
    Ok((jar,Redirect::to("/")))
}
#[derive(Deserialize)]
pub struct ChangePassword {
    pub old: String,
    pub new: String,
    pub confirm: String
}

pub async fn change_password(
    pool: State<Pool<Postgres>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Form(change_password): Form<ChangePassword>,
) -> Result<Redirect>{
    if change_password.new != change_password.confirm{
        todo!()
    }
    let mut txn = pool.begin().await?;
    user.update_password(&mut txn, &change_password.old, &change_password.new)
        .await?;
    txn.commit().await?;
    Ok(Redirect::to("/admin"))
}
//TODO this should check if they superuser
pub async fn delete_user(
    _user: AuthenticatedUser,
    Path(user_id): Path<i32>,
    pool: State<Pool<Postgres>>,
) -> Result<()> {
    let mut txn = pool.begin().await?;
    User::delete_user(&mut txn, user_id).await?;
    txn.commit().await?;
    Ok(())
    
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    error: Option<String>
}

pub async fn get_login_form() -> LoginTemplate {
    LoginTemplate{error: None}
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct ChangePasswordTemplate;

pub async fn get_password_form(_user: AuthenticatedUser) -> ChangePasswordTemplate {
    ChangePasswordTemplate
}

