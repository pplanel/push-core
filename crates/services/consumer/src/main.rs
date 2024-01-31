use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    init_logging();
    let uri = "amqp://localhost:5673";
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(uri, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    let mut consumer = channel
        .basic_consume(
            "queue",
            "tag_foo",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("consumer");
    info!(state=?connection.status().state());

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let message = serde_json::from_slice::<Message>(&delivery.data).unwrap();
            info!(message=?message, "received message");
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack send_webhook_event message");
        }
    }
    channel
        .basic_publish(
            "",
            "queue_test",
            BasicPublishOptions::default(),
            b"Hello world!",
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();

    std::future::pending::<()>().await;
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
