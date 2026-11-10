use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::services::ServeDir;

mod auth;
mod bootstrap;
mod config;
mod error;
mod models;
mod response;
mod routes;
mod state;
mod storage;

use config::Config;
use state::AppState;
use storage::ImageStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    tracing::info!("开始执行数据库迁移");
    sqlx::migrate!("./migrations").run(&pool).await?;
    bootstrap::bootstrap(&pool, &config).await?;

    if std::env::args().nth(1).as_deref() == Some("migrate") {
        println!("数据库迁移完成");
        return Ok(());
    }

    let images = ImageStore::new(config.image_dir.clone(), config.public_base_url.clone());
    images
        .ensure_dir()
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;

    let state = AppState {
        pool,
        images,
        token_ttl_hours: config.token_ttl_hours,
    };

    let app = routes::router()
        .nest_service("/images", ServeDir::new(config.image_dir.clone()))
        .layer(DefaultBodyLimit::max(12 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
                    "http://localhost:5173",
                )))
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!(
        port = config.port,
        image_dir = ?config.image_dir,
        "订单服务已启动"
    );
    axum::serve(listener, app).await?;
    Ok(())
}
