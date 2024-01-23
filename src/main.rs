use links::errors::Result;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = links::config::load();
    let key = links::config::load_key();

    let pool = PgPoolOptions::new()
        .connect(&config.database.url)
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();

    let state = links::AppState { pool, key };

    let app = links::routes::routes().with_state(state);
    let address = format!(
        "0.0.0.0:{port}",
        port = std::env::var("PORT").unwrap_or_else(|_| "3000".into())
    );
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
