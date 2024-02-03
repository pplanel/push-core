use std::time::Duration;

use clap::Parser;
use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use lib_core::Message;
use tokio::sync::oneshot;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "queue")]
    queue: String,

    #[arg(short, long, default_value_t = 2)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    init_logging();

    let args = Args::parse();

    let uri = "amqp://localhost:5673";
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(uri, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    let _queue = channel
        .queue_declare(
            &args.queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!(queue = &args.queue, "Queue declared");

    let (tx, mut rx) = oneshot::channel();

    let publisher_handle = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(args.timeout));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let message = Message::fake();
                    let payload = serde_json::to_vec(&message).unwrap();
                    let _ = channel
                        .basic_publish(
                            "",
                            &args.queue,
                            BasicPublishOptions::default(),
                            &payload,
                            BasicProperties::default().with_priority(10),
                            )
                        .await
                        .expect("publish")
                        .await
                        .expect("confirmation");

                    info!("Message published");
                }
                _ = &mut rx => {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("terminating");
            connection.close(0, "").await.unwrap();
            tx.send(true).unwrap();
            let _ = publisher_handle.await;
        }
    }
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
