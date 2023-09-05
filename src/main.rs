use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use axum::{
    extract::ConnectInfo,
    http::{
        header::{HeaderMap, USER_AGENT},
        StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Data {
    ip: String,
    port: u16,
    user_agent: String,
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

async fn root(ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap) -> String {
    tracing::info!("GET(/) Handling connection from {}", addr);
    format!("{}", ip(addr.ip(), &headers))
}

async fn json(ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap) -> impl IntoResponse {
    tracing::info!("GET(/json) Handling connection from {}", addr);

    let connection_data = Data {
        ip: ip(addr.ip(), &headers),
        port: addr.port(),
        user_agent: user_agent(&headers),
    };

    (StatusCode::OK, Json(connection_data))
}

fn ip(address: IpAddr, headers: &HeaderMap) -> String {
    let mut ip = address.to_string();

    if headers.contains_key("X-FORWARDED-FOR") {
        // This header can contain multiple values and the origin IP might have
        // the port number, e.g. X-FORWARDED-FOR: client, proxy1, proxy2
        let value = headers
            .get("X-Forwarded-for")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Extract the first IP
        ip = value.split(",").nth(0).unwrap().to_string();

        // Remove port number, if it is there. Beware of IPv6 colons.
        // There are five cases:
        // - 0.0.0.0      - IPv4 without port
        // - 0.0.0.0:0    - IPv4 with port
        // - 0:0:0::0     - IPv6 without port and without brackets
        // - [0:0:0::0]   - IPv6 without port and with brackets
        // - [0:0:0::0]:0 - IPv6 with port and with brackets
        // First deal with [IPv6]:PORT
        ip = match ip.split_once("]:") {
            None => ip,
            Some(value) => value.0.to_string().replace("[", ""),
        };

        // IPv6 does not contains dots
        if ip.contains(".") {
            // IPv4 with or without port
            ip = match ip.split_once(":") {
                None => ip,
                Some(value) => value.0.to_string(),
            }
        };

        ip = ip.replace("[", "").replace("]", "");
    };

    ip
}

fn user_agent(headers: &HeaderMap) -> String {
    match headers.get(USER_AGENT) {
        Some(ua) => ua.to_str().unwrap().to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::{HeaderMap, HeaderValue};
    use std::net::IpAddr;

    #[test]
    fn test_ip_without_headers() {
        // Direct connection to the service
        for address in [
            "1.2.3.4",
            "1:2:3:4:5:6:7:8",
            "2001:db8:85a3:8d3:1319:8a2e:370:7348",
        ] {
            let ip_addr = address.parse::<IpAddr>().unwrap();
            assert_eq!(ip(ip_addr, &HeaderMap::new()), address);
        }
    }

    #[test]
    fn test_ip_with_x_forwarded_for_with_1_value() {
        // There's a reverse proxy at 10.0.0.1
        let priv_address = "10.0.0.1".parse::<IpAddr>().unwrap();
        let expected = "1.2.3.4";

        let mut headers = HeaderMap::new();
        let _ = headers.insert("X-FORWARDED-FOR", HeaderValue::from_str(expected).unwrap());

        assert_eq!(ip(priv_address, &headers), expected.to_string());
    }

    #[test]
    fn test_ip_with_x_forwarded_for_with_2_values() {
        let priv_address = "10.0.0.1";
        let expected = "1.2.3.4";

        let mut headers = HeaderMap::new();
        // X-FORWARDED-FOR: 1.2.3.4, 10.0.0.1
        let header_value = format!("{}, {}", expected, priv_address);
        let _ = headers.insert(
            "X-FORWARDED-FOR",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        assert_eq!(
            ip(priv_address.parse::<IpAddr>().unwrap(), &headers),
            expected.to_string()
        );
    }

    #[test]
    fn test_ip_with_x_forwarded_for_with_2_values_and_port() {
        let priv_address = "10.0.0.1";
        let expected = "1.2.3.4";

        let mut headers = HeaderMap::new();
        // X-FORWARDED-FOR: 1.2.3.4:4567, 10.0.0.1
        let header_value = format!("{}:4567, {}", expected, priv_address);
        let _ = headers.insert(
            "X-FORWARDED-FOR",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        assert_eq!(
            ip(priv_address.parse::<IpAddr>().unwrap(), &headers),
            expected.to_string()
        );
    }

    #[test]
    fn test_ip_with_x_forwarded_for_with_ipv6_example_from_wikipedia() {
        let priv_address = "198.51.100.100";
        let expected = "2001:db8::1a2b:3c4d";

        let mut headers = HeaderMap::new();
        // X-FORWARDED-FOR: 1.2.3.4:41237, 10.0.0.1:26321
        let header_value = format!("[{}]:41237, {}:26321", expected, priv_address);
        let _ = headers.insert(
            "X-FORWARDED-FOR",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        assert_eq!(
            ip(priv_address.parse::<IpAddr>().unwrap(), &headers),
            expected.to_string()
        );
    }

    #[test]
    fn test_user_agent_from_empty_header() {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "".parse().unwrap());

        assert_eq!(user_agent(&headers), "".to_string());
    }

    #[test]
    fn test_user_agent_from_curl() {
        let expected = "curl/8.2.1";
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, expected.parse().unwrap());

        assert_eq!(user_agent(&headers), expected.to_string());
    }
}
