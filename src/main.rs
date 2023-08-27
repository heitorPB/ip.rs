use std::net::{SocketAddr, IpAddr, Ipv6Addr};

use axum::{
    extract::ConnectInfo, http::StatusCode, response::IntoResponse, routing::get, Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct Data {
    ip: String,
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let address = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 3000);

    tracing::info!("Listening on {}", address);
    axum::Server::bind(address)
        .serve(app().into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/json", get(json))
}

async fn root(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    tracing::info!("GET(/) Handling connection from {}", addr);
    format!("Hello, {}", addr.ip())
}

async fn json(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    tracing::info!("GET(/json) Handling connection from {}", addr);

    let connection_data = Data {
        ip: addr.ip().to_string(),
        port: addr.port(),
    };

    (StatusCode::OK, Json(connection_data))
}
