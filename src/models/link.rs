use sqlx::{Transaction, Postgres};

use crate::{handlers::link::CreateLink, errors::Result};


pub struct Link {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub ordr: i32,
}

impl Link {
    //TOOD record who created
    pub async fn new(txn: &mut Transaction<'_, Postgres>, link: CreateLink) -> Result<Self> {
        let res = sqlx::query!(
        "INSERT INTO links (title, description, link, ordr) VALUES ($1, $2, $3, $4) RETURNING id",
        link.title,
        link.description,
        link.link,
        link.order
    ).fetch_one(&mut **txn)
            .await?;

        Ok(Self {
            id: res.id,
            title: link.title,
            description: link.description,
            link: link.link,
            ordr: link.order,
        })
    }

    pub async fn get_all(txn: &mut Transaction<'_, Postgres>) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(Self, "SELECT id, title, description, link, ordr FROM links")
        .fetch_all(&mut **txn)
            .await?;
        Ok(res)
    }
}
