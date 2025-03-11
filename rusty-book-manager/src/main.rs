use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing::get, Router};
use tokio::net::TcpListener;

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let app = Router::new().route("/hello", get(hello_world));
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
