extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate eventsourcing;
extern crate serde_json;
#[macro_use]
extern crate eventsourcing_derive;
use eventsourcing::EventMeta;
use eventsourcing::{Aggregate, AggregateState, Result};
//use eventsourcing::{Dispatcher};

//use chrono::prelude::*;

const DOMAIN_VERSION: &str = "1.0";

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
//#[derive(MyEvent)]
#[event_type_version(DOMAIN_VERSION)]
#[event_source("events://github.com/pholactery/eventsourcing/tests/integration")]
pub enum TestEvent {
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

    fn apply_event(_state: &Self::State, _evt: &Self::Event) -> Result<Self::State> {
        unimplemented!()
    }

    fn handle_command(_state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>> {
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

//use futures::future::join_all;
//
//#[async_trait]
//impl ::eventsourcing::Dispatcher for TestDispatcher {
//    type Aggregate = TestAgg;
//    type Event = <TestAgg as Aggregate>::Event;
//    type Command = <TestAgg as Aggregate>::Command;
//    type State = <TestAgg as Aggregate>::State;
//    type Services = <TestAgg as Aggregate>::Services;
//    #[allow(
//        clippy::let_unit_value,
//        clippy::no_effect_underscore_binding,
//        clippy::shadow_same,
//        clippy::type_complexity,
//        clippy::type_repetition_in_bounds,
//        clippy::used_underscore_binding
//    )]
//    async fn dispatch(
//        state: Self::State,
//        cmd: Self::Command,
//        svc: Self::Services,
//        store: impl ::eventsourcing::eventstore::EventStoreClient,
//        stream: String,
//    ) -> Vec<Result<::eventsourcing::cloudevents::CloudEvent>> {
//        let __ret: Vec<Result<::eventsourcing::cloudevents::CloudEvent>> = {
//            match Self::Aggregate::handle_command(&state, &cmd, &svc).await {
//                Ok(evts) => {
//                    join_all(
//                        evts.into_iter()
//                            .map(|evt| store.append(evt, &stream))
//                            .collect::<Vec<_>>(),
//                    )
//                    .await
//                }
//                Err(e) => vec![Err(e)],
//            }
//        };
//        __ret
//    }
//}

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
    //assert_eq!(false, true);
}
