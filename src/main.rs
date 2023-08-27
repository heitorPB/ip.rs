use axum::{extract::ConnectInfo, routing::get, Router};
use std::net::{SocketAddr, IpAddr, Ipv6Addr};

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
    Router::new().route("/", get(root))
}

async fn root(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    tracing::info!("GET(/) Handling connection from {}", addr);
    format!("Hello, {}", addr.ip())
}
