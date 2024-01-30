use sqlx::{Transaction, Postgres};

use crate::{handlers::link::{CreateLink, UpdateLink}, errors::Result};


pub struct Link {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub link: String,
    pub ordr: i32,
}

impl Link {
    //TODO record who created
    pub async fn new(txn: &mut Transaction<'_, Postgres>, link: CreateLink) -> Result<Self> {
        let res = sqlx::query!(
        "INSERT INTO links (title, description, link, ordr) VALUES ($1, $2, $3, (SELECT max(ordr) + 1 FROM links)) RETURNING id, ordr",
        link.title,
        link.description,
        link.link,
    ).fetch_one(&mut **txn)
            .await?;

        Ok(Self {
            id: res.id,
            title: link.title,
            description: link.description,
            link: link.link,
            ordr: res.ordr,
        })
    }

    pub async fn get_all(txn: &mut Transaction<'_, Postgres>) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(Self, "SELECT id, title, description, link, ordr FROM links ORDER BY ordr ASC")
        .fetch_all(&mut **txn)
            .await?;
        Ok(res)
    }

    //TODO audit log
    pub async fn update(txn: &mut Transaction<'_, Postgres>, id: i32, link:UpdateLink) -> Result<()> {
        let res = sqlx::query!("UPDATE links SET title = $1, description = $2, link = $3 WHERE id = $4",
        link.title,
        link.description,
        link.link,
        id).execute(&mut **txn).await?;
        match res.rows_affected() {
            1 => Ok(()),
            0 => Err(crate::errors::LinksError::InternalServerError("link doesnt exist".into())),
            _ => panic!("more than 1 row exists with id {{id}}")
        }
    }

    //TODO audit log
    //TODO this should change ordering
    pub async fn delete(txn: &mut Transaction<'_, Postgres>, id: i32) -> Result<()> {
        let res = sqlx::query!("DELETE FROM links WHERE id = $1",id).execute(&mut **txn).await?;
        match res.rows_affected() {
            1 => Ok(()),
            0 => Err(crate::errors::LinksError::InternalServerError("link doesnt exist".into())),
            _ => panic!("more than 1 row exists with id {{id}}")
        }
    }
}
