extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate eventsourcing_derive;
extern crate eventsourcing;
extern crate serde_json;

mod domain;

use crate::eventsourcing::Dispatcher;
use domain::{CombatCommand, CombatDispatcher, CombatEvent, CombatState};
//use domain::{CombatServices};
use eventsourcing::eventstore::EventStoreDBClient;
use eventsourcing::prelude::*;

#[tokio::main]
async fn main() {
    let combat_store = EventStoreDBClient::new("localhost", 2113);
    let swing = CombatCommand::Attack("ogre".to_owned(), 150);

    let state = CombatState {
        entity_id: "ogre".to_owned(),
        hitpoints: 900,
        generation: 0,
    };

    let rando = CombatEvent::RandomEvent { a: 12, b: 13 };
    println!("{}", rando.event_type());
    let unit = CombatEvent::UnitEvent;
    println!("{}", unit.event_type());
    let res = CombatDispatcher::dispatch(
        state,
        swing,
        combat_store,
        "ogre".to_string(),
        EventMeta::default(),
    )
    .await;
    println!("dispatch results - {:#?}", res);
}
