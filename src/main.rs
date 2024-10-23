use std::{env, net::SocketAddr};

use anyhow::Result;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use ip_info::{handle_css::axum_handle_css, handle_index::handle_index};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .init();

    let port: i32 = env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8000);

    let app = Router::new()
        .route("/main.css", get(axum_handle_css))
        .route("/", get(handle_index))
        .layer(middleware::from_fn(log))
        .into_make_service_with_connect_info::<SocketAddr>();

    let bind_addr = format!("[::]:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn log(request: Request, next: Next) -> Response {
    let socket_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.ip());

    let headers = &request.headers();
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let ip = if let Some(socket_ip) = socket_ip {
        ip_info::handle_index::real_ip(headers, socket_ip)
    } else {
        "".to_string()
    };

    let method = request.method().clone();
    let path = request.uri().clone();

    let response = next.run(request).await;

    tracing::info!(message = "request", method = %method, src_ip = %ip, user_agent = %user_agent, path = %path.path());

    response
}
