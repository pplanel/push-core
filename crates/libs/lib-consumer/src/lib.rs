#![allow(unused)]
use async_stream::stream;

use futures_core::stream::Stream;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use redis::{
    aio::{Connection, ConnectionLike},
    IntoConnectionInfo, Msg,
};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

pub struct Subscriber<S> {
    client: redis::Client,
    channels: Vec<S>,
}

impl<S> Subscriber<S>
where
    S: AsRef<str> + redis::ToRedisArgs,
{
    pub fn new<U>(redis_url: U, channels: Vec<S>) -> Result<Self>
    where
        U: IntoConnectionInfo,
    {
        let client = redis::Client::open(redis_url)?;

        Ok(Self { client, channels })
    }

    pub async fn poll_messages(&mut self) -> impl Stream<Item = redis::Msg> {
        let conn = self
            .client
            .get_async_connection()
            .await
            .expect("get a async onncetion");
        let mut pubsub = conn.into_pubsub();
        let _ = pubsub.subscribe(&self.channels).await;

        stream! {
            if let Some(msg )= pubsub.on_message().next().await {
                yield msg
            }
        }
    }
}

#[tokio::test]
async fn test_redis_pubsub() {
    let mut subscriber = Subscriber::new("redis://127.0.0.1:6679", vec!["events"]).unwrap();

    let s = subscriber.poll_messages().await;
    pin_mut!(s);

    let msg = s.next().await;
    assert!(msg.is_some())
}
