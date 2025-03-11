use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{http::StatusCode, routing::get, Router};
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    let app = Router::new()
        .route("/hello", get(hello_world))
        .route("/health", get(health_check));
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);
    Ok(axum::serve(listener, app).await?)
}
