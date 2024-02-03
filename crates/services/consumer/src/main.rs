#![allow(unused)]
use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use lib_core::Message;
use redis::Commands;
use tracing::{debug, error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug)]
struct RabbitConsumer {
    name: String,
    queue: String,
    channel: Channel,
}

impl RabbitConsumer {
    async fn new(queue: String, connection: Connection) -> Self {
        let channel = connection.create_channel().await.unwrap();
        Self {
            name: format!("{}-consumer", queue),
            queue,
            channel,
        }
    }

    async fn consume<F>(&mut self, fun: F)
    where
        F: Fn(&Message),
    {
        let mut consumer = self
            .channel
            .basic_consume(
                "queue",
                "tag_foo",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("consumer");

        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                let message = serde_json::from_slice::<Message>(&delivery.data).unwrap();

                (fun)(&message);

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    let uri = std::env::var("RABBIT_CONN").unwrap();

    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(&uri, options)
        .await
        .expect("get a rabbit connection");

    debug!(state=?connection.status().state());

    let mut consumer = RabbitConsumer::new("queue".into(), connection).await;

    let redis_client =
        redis::Client::open(std::env::var("REDIS_CONN").unwrap()).expect("Open a Redis connection");

    consumer
        .consume(|message| {
            if let Ok(mut con) = redis_client.get_connection() {
                info!(message=?message, "received message");
                match con.publish::<&str, &Message, usize>("events", message) {
                    Ok(_) => info!("Message sent to Redis"),
                    Err(err) => error!(err = ?err, "Cannot publish redis message"),
                };
            } else {
                panic!("Cannot get Redis connection");
            }
        })
        .await;

    std::future::pending::<()>().await;
}

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .without_time();

    let env_filter = EnvFilter::from_default_env().add_directive("consumer=info".parse().unwrap());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();
}
