# nestjs-nats-connector

a simple crate that interacts with nestjs over nats

# usage

```rust
use std::error::Error;
use async_nats::{Connection, Message};
use nestjs_nats_connector::listener;
use nestjs_nats_connector::listener::IncomingRequest;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Serialize, Deserialize)]
pub struct Pattern {
    cmd: String
}

pub struct Handler {

}

#[async_trait]
impl listener::Listener for Handler {
    type Pattern = Pattern;
    type RequestData = u32;

    async fn handler(
        &self,
        connection: &Connection,
        message: &Message,
        data: IncomingRequest<Self::Pattern, Self::RequestData>
    ) {
        println!("{}", data.id)
    }

    fn get_pattern(&self) -> Self::Pattern {
        Pattern {
            cmd: "dashboard.graph".to_string()
        }
    }
}

impl Handler {
    pub fn new() -> Handler {
        Handler { }
    }
}


async fn test(){
    let connection = async_nats::connect("localhost:4222")
        .await
        .expect("cant connect to nats server");

    nestjs_nats_connector::listener::listen(
        &connection,
        vec![
            Handler::new()
        ]
    ).await;
}
```