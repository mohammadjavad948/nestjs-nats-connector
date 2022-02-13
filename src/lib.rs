mod listener {
    use async_nats::{Connection, Message};
    use serde::{Deserialize, Serialize};
    use std::str;

    pub trait Listener {
        type Pattern: Serialize;

        fn handler(connection: &Connection, message: &Message);

        fn get_pattern(&self) -> Self::Pattern;

        fn serialize_pattern(&self) -> String {
            serde_json::to_string(&self.get_pattern()).unwrap()
        }

        fn deserialize_message_data<Data>(&self, message: Message){
            let stringify_data = str::from_utf8(&message.data).unwrap();
        }
    }

    #[derive(Deserialize)]
    pub struct IncomingRequest<Pattern, Data> {
        pub pattern: Pattern,
        pub id: String,
        pub data: Data,
    }
}
