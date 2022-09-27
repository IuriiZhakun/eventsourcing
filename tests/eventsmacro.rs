extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate eventsourcing;
extern crate serde_json;
#[macro_use]
extern crate eventsourcing_derive;
use eventsourcing::Event;

//use chrono::prelude::*;

const DOMAIN_VERSION: &str = "1.0";

#[derive(Debug, Serialize, Deserialize, Event)]
//#[derive(MyEvent)]
#[event_type_version(DOMAIN_VERSION)]
#[event_source("events://github.com/pholactery/eventsourcing/tests/integration")]
enum TestEvent {
    Sample { val1: u32, val2: u32, val3: String },
}

use tokio;

#[tokio::test]
async fn eventstoredb_client() {
    let se = TestEvent::Sample {
        val1: 1,
        val2: 2,
        val3: "hello".to_owned(),
    };

    println!("save result {:?}", se);
    //println!("save result {:?}", se.event_type());
    assert_eq!(false, true);
}
