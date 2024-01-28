use askama::Template;
use axum::{extract::State, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{errors::Result, models::link::Link};

//todo add image support
#[derive(Deserialize)]
pub struct CreateLink{
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub order: i32,
}

pub async fn create_link(pool: State<Pool<Postgres>>, Json(link): Json<CreateLink>) -> Result<Json<i32>>{
    let mut txn = pool.begin().await?;
    let link = Link::new(&mut txn, link).await?;
    txn.commit().await?;
    Ok(Json(link.id))
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    links: Vec<Link>
}

pub async fn index(pool: State<Pool<Postgres>>) -> Result<IndexTemplate> {
    let mut txn = pool.begin().await?;
    let links = Link::get_all(&mut txn).await?;
    let tmpl = IndexTemplate{links};
    txn.commit().await?;
    Ok(tmpl)
}
