use futures::FutureExt;
use futures_util::pin_mut;
use lib_consumer::Subscriber;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug)]
struct WorkerState {
    clients: Mutex<HashMap<String, SocketAddr>>,
    message_count: AtomicUsize,
    broadcast_chan: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    init_logging();
    let port = std::env::var("PORT")
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(3000))
        .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let (tx, _rx) = broadcast::channel(100);
    //let (_s_tx, s_rx) = watch::channel(());
    let token = CancellationToken::new();

    let state = Arc::new(WorkerState {
        clients: Mutex::new(HashMap::new()),
        broadcast_chan: tx.clone(),
        message_count: AtomicUsize::new(0),
    });

    let broadcast_cancelation_token = token.clone();
    let tx_2 = tx.clone();
    let broadcast = tokio::spawn(async move {
        let mut subscriber = Subscriber::new("redis://127.0.0.1:6679", vec!["events"]).unwrap();
        loop {
            tokio::select! {
                _ = broadcast_cancelation_token.cancelled() => {
                    info!("break broadcaster");
                    break;
                }
                s = subscriber.poll_messages() => { // get redis messages
                    let _ = tokio::time::timeout(Duration::from_secs(1), async {
                    info!("polling");
                    pin_mut!(s);

                    while let Some(msg) = s.next().await {
                        let message: lib_core::Message = msg.get_payload().unwrap();
                        let payload = serde_json::to_string(&message).unwrap();
                        let _ = tx_2.send(payload);
                    }
                    }).await;
                }
            }
        }
    });

    let analytics_cancelation_token = token.clone();
    let state_analytics = state.clone();
    let broadcast_to_clients = tx.clone();
    let analytics = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = analytics_cancelation_token.cancelled() => {
                    info!("break analytics");
                    break;
                }
                _ = async {
                    let mut clients = state_analytics.clients.lock().await;
                    let total_count = state_analytics.message_count.load(Ordering::SeqCst);
                    if clients.len() > 0 {
                        info!(capacity=clients.capacity(), messages_sent=total_count, clients=clients.len(), "Clients connected");
                        let _ = broadcast_to_clients.send(format!("Total number of messages {total_count}"));
                    }
                    clients.shrink_to_fit();
                    tokio::time::sleep(Duration::from_secs(2)).await;
                } => {}
            }
        }
    });

    let app = Router::new().route("/ws", get(handler)).with_state(state);

    let server =
        axum::Server::bind(&addr).serve(app.into_make_service_with_connect_info::<SocketAddr>());

    tokio::select! {
        _ = server.fuse() => {}
        _ = tokio::signal::ctrl_c().fuse() => {
            token.cancel();
            analytics.await.unwrap();
            broadcast.await.unwrap();

            debug!("Bye");
        }
    }
}

async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WorkerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, addr))
}

#[tracing::instrument]
async fn handle_socket(mut socket: WebSocket, state: Arc<WorkerState>, who: SocketAddr) {
    let mut user_name = String::default();
    if let Some(Ok(Message::Text(name))) = socket.recv().await {
        if !name.is_empty() && name != "\n" {
            debug!(name = name, "hello");
            user_name = name.clone();
            state.clients.lock().await.insert(name, who);
        } else {
            let _ = socket
                .send(Message::Text(String::from("Wrong username")))
                .await;
            return;
        }
    }

    let (mut sender, mut _receiver) = socket.split();

    let mut rx = state.broadcast_chan.subscribe();

    'main: loop {
        tokio::select! {
            Ok(s) = rx.recv() => {
                if sender.send(Message::Text(s)).await.is_err() {
                    break 'main;
                }
            }
            Some(Ok(Message::Close(_))) = _receiver.next() => {
                break 'main;
            }
        }
    }

    state.clients.lock().await.remove(&user_name);
}

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .without_time();

    let env_filter =
        EnvFilter::from_default_env().add_directive("websocket_worker=debug".parse().unwrap());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}
