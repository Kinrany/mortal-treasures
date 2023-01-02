use std::{io, net::SocketAddr};

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Router, Server,
};

use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;

use crate::game::Game;

fn router(game: Game) -> Router {
    let dir = ServeDir::new("static").not_found_service(ServeFile::new("static/not_found.txt"));
    Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", get_service(dir).handle_error(handle_error))
        .with_state(game)
        .layer(TraceLayer::new_for_http())
}

async fn ws_handler(ws: WebSocketUpgrade, State(game): State<Game>) -> Response {
    ws.on_upgrade(|s| async { game.add_player(s).await.unwrap() })
}

pub async fn serve() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on http://{}", addr);
    Server::bind(&addr)
        .serve(router(Game::new()).into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
