use std::net::SocketAddr;

use autometrics::{
    autometrics,
    prometheus_exporter::{self},
};
use axum::{
    extract::{ws::WebSocket, ConnectInfo, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::stream::{SplitStream, StreamExt};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    prometheus_exporter::init();
    init_logging();
    let app = Router::new()
        .route("/", get(root_get))
        .route(
            "/metrics",
            get(|| async {
                prometheus_exporter::encode_to_string()
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            }),
        )
        .route("/ws", get(handle_ws));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let add = listener.local_addr().unwrap();
    info!("Server running on {}", add);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[autometrics]
async fn handle_ws(ConnectInfo(addr): ConnectInfo<SocketAddr>, ws: WebSocketUpgrade) -> Response {
    info!("Client {addr} connected");
    ws.on_upgrade(handle_upgraded)
}

#[autometrics]
async fn handle_upgraded(socket: WebSocket) {
    let (_, receiver) = socket.split();

    //tokio::spawn(write(sender));
    tokio::spawn(read(receiver));
}

#[autometrics]
async fn read(mut receiver: SplitStream<WebSocket>) {
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            info!("Message received {:?}", msg)
        };
    }

    info!("User disconnected");
}

// async fn write(mut sender: SplitSink<WebSocket, Message>) {
//     loop {
//         match sender.send(Message::Text("Hello".to_string())).await {
//             Ok(_) => (),
//             Err(err) => {
//                 error!("Something went wrong {err:?}");
//                 sender.close().await.unwrap();
//                 break;
//             }
//         }
//         tokio::time::sleep(Duration::from_secs(2)).await;
//     }
// }
//
//

async fn root_get() -> impl IntoResponse {
    "Hello"
}

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .without_time();

    let env_filter = EnvFilter::from_default_env().add_directive("push_core=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}
