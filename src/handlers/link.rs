use askama::Template;
use axum::{
    extract::{Path, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{auth::AuthenticatedUser, errors::Result, models::link::Link};

//TODO add image support
#[derive(Deserialize)]
pub struct CreateLink {
    pub title: String,
    pub description: Option<String>,
    pub link: String,
}

pub async fn create_link(
    _user: AuthenticatedUser,
    pool: State<Pool<Postgres>>,
    Form(link): Form<CreateLink>,
) -> Result<Redirect> {
    let mut txn = pool.begin().await?;
    Link::new(&mut txn, link).await?;
    txn.commit().await?;
    Ok(Redirect::to("/admin"))
}

#[derive(Deserialize)]
pub struct UpdateLink {
    pub title: String,
    pub description: Option<String>,
    pub link: String,
}

pub async fn update_link(
    _user: AuthenticatedUser,
    pool: State<Pool<Postgres>>,
    Path(link_id): Path<i32>,
    Form(link): Form<UpdateLink>,
) -> Result<Redirect> {
    let mut txn = pool.begin().await?;
    Link::update(&mut txn, link_id, link).await?;
    txn.commit().await?;
    Ok(Redirect::to("/admin"))
}

pub async fn delete_link(
    _user: AuthenticatedUser,
    pool: State<Pool<Postgres>>,
    Path(link_id): Path<i32>,
) -> Result<Redirect> {
    let mut txn = pool.begin().await?;
    Link::delete(&mut txn, link_id).await?;
    txn.commit().await?;
    Ok(Redirect::to("/admin"))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    links: Vec<Link>,
}

pub async fn index(pool: State<Pool<Postgres>>) -> Result<IndexTemplate> {
    let mut txn = pool.begin().await?;
    let links = Link::get_all(&mut txn).await?;
    let tmpl = IndexTemplate { links };
    txn.commit().await?;
    Ok(tmpl)
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    links: Vec<Link>,
    superuser: bool,
}

pub async fn admin(user: AuthenticatedUser, pool: State<Pool<Postgres>>) -> Result<AdminTemplate> {
    let mut txn = pool.begin().await?;
    let links = Link::get_all(&mut txn).await?;
    let tmpl = AdminTemplate {
        links,
        superuser: user.0.superuser,
    };
    Ok(tmpl)
}
