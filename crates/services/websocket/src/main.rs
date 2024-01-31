use lib_core::Message;
use redis::PubSubCommands;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    init_logging();
    let redis_client = redis::Client::open("redis://127.0.0.1:6679").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    let _: () = con
        .subscribe("events", |msg| {
            let message: Message = msg.get_payload().unwrap();

            info!(message = ?message, "Message received on Redis");

            redis::ControlFlow::Continue
        })
        .unwrap();
}

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .without_time();

    let env_filter =
        EnvFilter::from_default_env().add_directive("websocket-worker=info".parse().unwrap());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}
