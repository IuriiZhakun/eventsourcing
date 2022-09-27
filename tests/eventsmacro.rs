extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate eventsourcing;
extern crate serde_json;
#[macro_use]
extern crate eventsourcing_derive;
use eventsourcing::Event;
use eventsourcing::{Aggregate, AggregateState, Dispatcher, Result};

//use chrono::prelude::*;

const DOMAIN_VERSION: &str = "1.0";

#[derive(Debug, Serialize, Deserialize, Event)]
//#[derive(MyEvent)]
#[event_type_version(DOMAIN_VERSION)]
#[event_source("events://github.com/pholactery/eventsourcing/tests/integration")]
enum TestEvent {
    Sample { val1: u32, val2: u32, val3: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestState {
    pub generation: u64,
}

impl AggregateState for TestState {
    fn generation(&self) -> u64 {
        self.generation
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct TestAgg;

pub struct TestServices;

#[derive(Debug)]
pub enum TestCommand {
    Hello(String),
}

use async_trait::async_trait;

#[async_trait]
impl Aggregate for TestAgg {
    type Event = TestEvent;
    type Command = TestCommand;
    type State = TestState;
    type Services = TestServices;

    fn apply_event(_state: &Self::State, _evt: &Self::Event) -> Result<Self::State> {
        unimplemented!()
    }

    async fn handle_command(
        _state: &Self::State,
        cmd: &Self::Command,
        _svc: &Self::Services,
    ) -> Result<Vec<Self::Event>> {
        println!("Command handled: {:#?}", cmd);
        // SHOULD DO: validate state and command
        let evt = match *cmd {
            TestCommand::Hello(ref entity_id) => TestEvent::Sample {
                val1: 0,
                val2: 0,
                val3: entity_id.clone(),
            },
        };

        // if validation passes...
        Ok(vec![evt])
    }
}

#[derive(Dispatcher)]
#[aggregate(TestAgg)]
struct TestDispatcher;

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
