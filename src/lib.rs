mod listener {
    use async_nats::{Connection, Message, Subscription};
    use async_trait::async_trait;
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::str;

    #[async_trait]
    pub trait Listener {
        type Pattern: Serialize + DeserializeOwned;
        type RequestData: DeserializeOwned;

        async fn handler(
            &self,
            connection: &Connection,
            message: &Message,
            data: IncomingRequest<Self::Pattern, Self::RequestData>,
        );

        fn get_pattern(&self) -> Self::Pattern;

        fn serialize_pattern(&self) -> String {
            serde_json::to_string(&self.get_pattern()).unwrap()
        }

        fn deserialize_message_data(
            &self,
            message: &Message,
        ) -> Result<IncomingRequest<Self::Pattern, Self::RequestData>, Box<dyn Error>> {
            let stringify_data = str::from_utf8(&message.data)?;

            Ok(serde_json::from_str(stringify_data).unwrap())
        }
    }

    #[derive(Deserialize)]
    pub struct IncomingRequest<Pattern, Data> {
        pub pattern: Pattern,
        pub id: String,
        pub data: Data,
    }

    pub async fn listen<T: Listener>(connection: &Connection, listeners: Vec<T>) {
        let futures = FuturesUnordered::new();

        for list in listeners {
            let listener = connection
                .subscribe(&*list.serialize_pattern())
                .await
                .unwrap();

            futures.push(run_handler(list, listener, connection));
        }

        futures.collect::<Vec<()>>().await;
    }

    async fn run_handler<T: Listener>(handler: T, listener: Subscription, connection: &Connection) {
        for message in listener.next().await {
            let deserialize = handler.deserialize_message_data(&message).unwrap();

            handler.handler(connection, &message, deserialize).await;
        }
    }
}
