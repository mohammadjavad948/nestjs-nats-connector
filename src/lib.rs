/// listener module
/// it can subscribe to requests and deserialize it
pub mod listener {
    use async_nats::{Connection, Message, Subscription};
    use async_trait::async_trait;
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::str;

    /// this trait is the core of the entire library
    /// you make listeners by implementing this trait
    /// # example
    /// ```rust
    /// #[derive(Serialize, Deserialize)]
    /// pub struct Pattern {
    ///     cmd: String
    /// }
    ///
    /// pub struct Handler {
    ///
    /// }
    ///
    /// #[async_trait]
    /// impl Listener for Handler {
    ///     type Pattern = Pattern;
    ///     type RequestData = u32;
    ///
    ///     async fn handler(
    ///         &self,
    ///         connection: &Connection,
    ///         message: &Message,
    ///         data: IncomingRequest<Self::Pattern, Self::RequestData>
    ///     ) {
    ///         println!("{}", data.id)
    ///     }
    ///
    ///     fn get_pattern(&self) -> Self::Pattern {
    ///         Pattern {
    ///             cmd: "dashboard.graph".to_string()
    ///         }
    ///     }
    /// }
    ///
    /// impl Handler {
    ///     pub fn new() -> Handler {
    ///         Handler { }
    ///     }
    /// }
    /// ```
    #[async_trait]
    pub trait Listener {
        type Pattern: Serialize + DeserializeOwned;
        type RequestData: DeserializeOwned;

        /// handles incoming request
        async fn handler(
            &self,
            connection: &Connection,
            message: &Message,
            data: IncomingRequest<Self::Pattern, Self::RequestData>,
        );

        /// get the pattern that subscribes to it
        fn get_pattern(&self) -> Self::Pattern;

        /// serialize pattern to string for subject
        fn serialize_pattern(&self) -> String {
            serde_json::to_string(&self.get_pattern()).unwrap()
        }

        /// deserialize incoming request
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
        loop {
            if let Some(message) = listener.next().await {
                let deserialize = handler.deserialize_message_data(&message).unwrap();

                handler.handler(connection, &message, deserialize).await;
            }
        }
    }
}

pub mod requester {
    use std::str;
    use async_nats::{Connection, Message};
    use serde::{Deserialize, Serialize};
    use serde::de::DeserializeOwned;

    #[derive(Serialize, Deserialize)]
    pub struct Request<Pattern, Data> {
        pub pattern: Pattern,
        pub id: String,
        pub data: Data,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Response<Pattern, Data> {
        pub pattern: Pattern,
        pub id: String,
        pub data: Data,
        pub isDisposed: bool,
    }

    pub async fn request<
        Pattern: Serialize + DeserializeOwned,
        RequestData: Serialize,
        ResponseData: DeserializeOwned,
    >(
        pattern: Pattern,
        data: RequestData,
        connection: &Connection,
    ) -> (Message, Response<Pattern, ResponseData>) {
        let subject = serde_json::to_string(&pattern).unwrap();

        let request = Request {
            pattern,
            data,
            id: uuid::Uuid::new_v4().to_hyphenated().to_string(),
        };

        let request = serde_json::to_string(&request).unwrap();

        let message = connection.request(
            &*subject,
            request
        ).await.unwrap();

        let deserialize: Response<Pattern, ResponseData> = serde_json::from_str(
            str::from_utf8(&message.data).unwrap()
        ).unwrap();

        return (message, deserialize);
    }
}
