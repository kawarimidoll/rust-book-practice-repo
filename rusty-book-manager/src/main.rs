use sqlx::{postgres::PgConnectOptions, PgPool};
use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::get, Router};
use tokio::net::TcpListener;

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}
impl From<DatabaseConfig> for PgConnectOptions {
    fn from(cfg: DatabaseConfig) -> Self {
        Self::new()
            .host(&cfg.host)
            .port(cfg.port)
            .username(&cfg.username)
            .password(&cfg.password)
            .database(&cfg.database)
    }
}

fn connect_database_with(cfg: DatabaseConfig) -> PgPool {
    PgPool::connect_lazy_with(cfg.into())
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tokio::test]
async fn health_check_works() {
    let status = health_check().await;
    assert_eq!(status, StatusCode::OK);
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    let connection_result = sqlx::query("SELECT 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[sqlx::test]
async fn health_check_db_works(pool: sqlx::PgPool) {
    let status_code = health_check_db(State(pool)).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let db_cfg = DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        username: "my_user".into(),
        password: "my_password".into(),
        database: "my_db".into(),
    };
    let conn_pool = connect_database_with(db_cfg);

    let app = Router::new()
        .route("/hello", get(hello_world))
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(conn_pool);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);
    Ok(axum::serve(listener, app).await?)
}
