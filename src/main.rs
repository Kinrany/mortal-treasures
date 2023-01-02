mod game;

use std::{fmt::Display, io, net::SocketAddr};

use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Router, Server,
};
use futures::{Sink, SinkExt, StreamExt};
use game::{Event, Game};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};
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

fn router(game: Game) -> Router {
    let dir = ServeDir::new("static").not_found_service(ServeFile::new("static/not_found.txt"));
    Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", get_service(dir).handle_error(handle_error))
        .with_state(game)
        .layer(TraceLayer::new_for_http())
}

#[debug_handler]
async fn ws_handler(ws: WebSocketUpgrade, State(game): State<Game>) -> Response {
    ws.on_upgrade(|s| handle_socket(s, game))
}

async fn handle_socket(socket: WebSocket, game: Game) {
    let (mut sender, mut receiver) = socket.split();

    async fn send<S, M>(sink: &mut S, m: M)
    where
        S: Sink<M> + Unpin,
        S::Error: Display,
    {
        if let Err(error) = sink.send(m).await {
            error!(%error, "outgoing websocket error");
        }
    }

    async fn send_event<S>(sink: &mut S, e: &Event)
    where
        S: Sink<Message> + Unpin,
        S::Error: Display,
    {
        let m: Message = serde_json::to_string(e).unwrap().into();
        send(sink, m).await
    }

    send_event(
        &mut sender,
        &Event::World(game.with_first_world(|w| w.clone()).await),
    )
    .await;

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                let message = msg.into_text().unwrap();
                debug!(%message, "incoming message");
                let Ok(event) = serde_json::from_str::<Event>(&message) else {
                    warn!("not an event");
                    continue;
                };
                game.apply_to_first(event.clone()).await;
                send_event(&mut sender, &event).await;
            }
            Err(error) => {
                warn!(%error, "incoming websocket error");
                send(&mut sender, error.to_string().into()).await;
            }
        }
    }
}

async fn serve() {
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
