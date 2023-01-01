use std::{io, net::SocketAddr};

use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router, Server};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_setup().await;
    serve().await;
}

async fn tracing_setup() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mortal_treasures=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn router() -> Router {
    let dir = ServeDir::new("static").not_found_service(ServeFile::new("static/not_found.txt"));
    Router::new()
        .nest_service("/", get_service(dir).handle_error(handle_error))
        .layer(TraceLayer::new_for_http())
}

async fn serve() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on http://{}", addr);
    Server::bind(&addr)
        .serve(router().into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
