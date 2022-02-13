mod listener {
    use std::error::Error;
    use async_nats::{Connection, Message};
    use serde::{Deserialize, Serialize};
    use std::str;
    use serde::de::DeserializeOwned;

    pub trait Listener {
        type Pattern: Serialize + DeserializeOwned;
        type RequestData: DeserializeOwned;

        fn handler(connection: &Connection, message: &Message);

        fn get_pattern(&self) -> Self::Pattern;

        fn serialize_pattern(&self) -> String {
            serde_json::to_string(&self.get_pattern()).unwrap()
        }

        fn deserialize_message_data(&self, message: Message)
            -> Result<
                IncomingRequest<Self::Pattern, Self::RequestData>,
                Box<dyn Error>
            >
        {
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
}
