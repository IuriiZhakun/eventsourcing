extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate eventsourcing;
extern crate serde_json;
#[macro_use]
extern crate eventsourcing_derive;
extern crate chrono;
use crate::eventsourcing::eventstore::EventStoreClient;

//use chrono::prelude::*;

const DOMAIN_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Event, Clone, Debug)]
#[event_type_version(DOMAIN_VERSION)]
#[event_source("events://github.com/pholactery/eventsourcing/tests/integration")]
enum TestEvent {
    Sample { val1: u32, val2: u32, val3: String },
}

use eventsourcing::EventMeta;
use tokio;

#[tokio::test]
async fn eventstoredb_client() {
    // ensure that we can produce a cloud event with an arbitrary nested JSON value in the data
    // field and serialize it, then deserialize it, and still be able to convert the serde_json::Value
    // back into the original raw event type.

    use eventsourcing::eventstore::EventStoreDBClient;
    let se = TestEvent::Sample {
        val1: 1,
        val2: 2,
        val3: "hello".to_owned(),
    };

    let edb = EventStoreDBClient::default();
    println!("created edb ");
    let r = edb
        .append(se.clone(), "testEDB", EventMeta::default())
        .await
        .unwrap();
    println!("save result {:?}", r);
}

#[tokio::test]
async fn eventstoredb_client_getall() {
    // ensure that we can produce a cloud event with an arbitrary nested JSON value in the data
    // field and serialize it, then deserialize it, and still be able to convert the serde_json::Value
    // back into the original raw event type.

    use eventsourcing::eventstore::EventStoreDBClient;
    let edb = EventStoreDBClient::default();
    let r = edb.get_all("testEDB").await.unwrap();
    println!("save result {:?}", r);
    let f = r[0].clone().into();
    match f {
        TestEvent::Sample { val1, val2, val3 } => {
            assert_eq!(val1, 1);
            assert_eq!(val2, 2);
            assert_eq!(val3, "hello");
        }
    }
}
