//! # Event Sourcing
//!
//! An eventsourcing library for Rust
//!
//! One of the benefits of [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
//! is that in most cases, embracing this pattern does not require that much code.
//! However, there's still a bit of boilerplate required as well as the discipline for ensuring
//! the events, commands, and aggregates all perform their roles without sharing concerns.
//!
//! The fundamental workflow to remember is that **commands** are applied to **aggregates**,
//! which then emit one or more events. An **aggregate**'s business logic is also responsible
//! for returning a new state from a previous state combined with a new event. Put
//! mathematically, this looks like:
//!
//! ```terminal,ignore
//! f(state1, event) = state2
//! ```
//!
//! There are some event sourcing libraries that allow for, or even encourage, mutation of
//! state in memory. I prefer a more functional approach, and the design of this library
//! reflects that. It encourages you to write unit tests for your aggregate business logic that
//! are predictable and can be executed in isolation without concern for how you receive events
//! or how you persist them in a store.
//!
//! To start, you create an undecorated enum for your **Command** type:
//! ```rust
//! enum LocationCommand {
//!    UpdateLocation { lat: f32, long: f32, alt: f32 },
//!}
//! ```
//!
//! Next, you create an enum for your events and use a derive macro to write some boilerplate
//! on your behalf. Note how the command variants are _imperative_ statements while the
//! event variants are _verbs phrases in the past tense_. While this is by convention and
//! not enforced via code, this is a good practice to adopt.
//! ```rust
//!
//!# extern crate serde;
//!# #[macro_use] extern crate serde_derive;
//!# extern crate eventsourcing;
//!# extern crate serde_json;
//!# #[macro_use] extern crate eventsourcing_derive;
//!const DOMAIN_VERSION: &str = "1.0";
//!#[derive(Serialize, Deserialize, Debug, Clone, Event)]
//!#[event_type_version(DOMAIN_VERSION)]
//!#[event_source("events://github.com/pholactery/eventsourcing/samples/location")]
//!enum LocationEvent {
//!    LocationUpdated { lat: f32, long: f32, alt: f32 },
//!}
//! ```
//!
//! We then define a type that represents the state to be used by an aggregate.
//! With that in place, we write all of our business logic, the core of our event sourcing system,
//! in the aggregate.
//!```rust
//!# extern crate serde;
//!# #[macro_use] extern crate serde_derive;
//!# extern crate eventsourcing;
//!# extern crate serde_json;
//!# #[macro_use] extern crate eventsourcing_derive;
//!# use eventsourcing::{prelude::*, Result};
//!const DOMAIN_VERSION: &str = "1.0";
//!# #[derive(Serialize, Deserialize, Debug, Clone, Event)]
//!# #[event_type_version(DOMAIN_VERSION)]
//!# #[event_source("events://github.com/pholactery/eventsourcing/samples/location")]
//!# enum LocationEvent {
//!#     LocationUpdated { lat: f32, long: f32, alt: f32 },
//!# }
//!# enum LocationCommand {
//!#    UpdateLocation { lat: f32, long: f32, alt: f32 },
//!# }
//!#[derive(Debug, Clone)]
//!struct LocationData {
//!    lat: f32,
//!    long: f32,
//!    alt: f32,
//!    generation: u64,
//!}
//!
//!impl AggregateState for LocationData {
//!    fn generation(&self) -> u64 {
//!        self.generation
//!    }
//!}
//!# #[derive(Serialize, Deserialize, Default)]
//!struct Location;
//!
//!impl Aggregate for Location {
//!   type Event = LocationEvent;
//!   type Command = LocationCommand;
//!   type State = LocationData;
//!
//!   fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State> {
//!       // TODO: validate event
//!       let ld = match evt {
//!           LocationEvent::LocationUpdated { lat, long, alt } => LocationData {
//!               lat: *lat,
//!               long: *long,
//!               alt: *alt,
//!               generation: state.generation + 1,
//!           },
//!       };
//!       Ok(ld)
//!   }
//!
//!   fn handle_command(_state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>> {
//!       // TODO: add code to validate state and command
//!
//!       // if validation passes...
//!       Ok(vec![LocationEvent::LocationUpdated { lat: 10.0, long: 10.0, alt: 10.0 }])
//!   }
//!}
//! ```

extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
extern crate uuid;
use deepsize::{Context, DeepSizeOf};

pub use cloudevents::CloudEvent;
use std::collections::HashMap;
use uuid::Uuid;

use crate::eventstore::EventStoreClient;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// An event sourcing error
#[derive(Debug)]
pub struct Error {
    pub kind: Kind,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "An eventsourcing error ocurred"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::ApplicationFailure(ref s) => fmt::Display::fmt(s, f),
            Kind::CommandFailure(ref s) => fmt::Display::fmt(s, f),
            Kind::StoreFailure(ref s) => fmt::Display::fmt(s, f),
        }
    }
}

/// Indicates the kind of event sourcing error that occurred.
#[derive(Debug)]
pub enum Kind {
    ApplicationFailure(String),
    CommandFailure(String),
    StoreFailure(String),
}

pub type EventMeta = HashMap<String, String>;

/// A Result where failure is an event sourcing error
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct LID {
    pub id: Uuid,
    links: Vec<Uuid>,
}

impl DeepSizeOf for LID {
    fn deep_size_of_children(&self, _context: &mut Context) -> usize {
        (self.links.len() + 1) * std::mem::size_of::<Uuid>()
    }
}

impl LID {
    pub fn get_links(&self) -> &Vec<Uuid> {
        &self.links
    }
    pub fn add_link(&mut self, id: Uuid) {
        self.links.push(id)
    }
    pub fn new() -> Self {
        LID {
            id: Uuid::new_v4(),
            links: Vec::new(),
        }
    }
    pub fn create(id: Uuid) -> Self {
        LID {
            id,
            links: Vec::new(),
        }
    }
    pub fn from(id: &Uuid) -> Self {
        LID {
            id: Uuid::new_v4(),
            links: vec![id.clone()],
        }
    }
    pub fn from_links(links: Vec<Uuid>) -> Self {
        LID {
            id: Uuid::new_v4(),
            links,
        }
    }
    pub fn next(&self) -> Self {
        LID::from(&self.id)
    }

    pub fn has(&self, id: &Uuid) -> bool {
        self.links.contains(id)
    }
}

impl From<LID> for EventMeta {
    fn from(lid: LID) -> Self {
        HashMap::from([
            ("uid".to_string(), lid.id.to_string()),
            (
                "links".to_string(),
                lid.links
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        ])
    }
}

/// All events must be serializable, and they need to expose some basic metadata
/// about the event, including the type name, the type version, and a source
/// to be used when events are emitted. If you use the derive macro for events,
/// you do not have to implement these functions manually.
pub trait Event: Serialize + DeserializeOwned + Send + std::fmt::Debug {
    //pub trait Event: Serialize + Deserialize<'_> + Send + std::fmt::Debug {
    fn event_type_version(&self) -> &str;
    fn event_type(&self) -> &str;
    fn event_source(&self) -> &str;
}

/// Aggregate state only requires that it expose the generation number. State generation
/// can be thought of as a sequential _version_. When a previous state is combined with
/// an event to produce a new state, that new state has a generation 1 higher than the
/// previous.
pub trait AggregateState {
    fn generation(&self) -> u64;
}

/// An aggregate is where the vast majority of business logic for an event sourcing system
/// occurs. They have two roles:
/// 1. Apply events to state, producing new state.
/// 2. Handle commands, producing a vector of outbound events, likely candidates for publication.
///
/// Both of these functions are stateless, as aggregates should also be stateless.
#[async_trait]
pub trait Aggregate: Default + Serialize + DeserializeOwned + Send + Sync {
    type Event: Event;
    type Command;
    type State: AggregateState + Clone;

    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State>;
    fn handle_command(state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>>;
    fn apply_all(state: &Self::State, evts: &[Self::Event]) -> Result<Self::State> {
        Ok(evts.iter().fold(state.clone(), |acc_state, event| {
            Self::apply_event(&acc_state, event).unwrap()
        }))
    }
    fn wrap_events(events: Vec<Self::Event>, lid: LID) -> EventEnvelope<Self::Event> {
        EventEnvelope {
            payload: events,
            lid,
            metadata: HashMap::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventEnvelope<Event> {
    /// The event payload with all business information.
    pub payload: Vec<Event>,
    pub lid: LID,
    /// Additional metadata for use in auditing, logging or debugging purposes.
    pub metadata: HashMap<String, String>,
}

//impl<A: Aggregate> Clone for EventEnvelope<A> {
//    fn clone(&self) -> Self {
//        EventEnvelope {
//            payload: self.payload.clone(),
//            lid: self.lid.clone(),
//            metadata: self.metadata.clone(),
//        }
//    }
//}

/// A dispatcher is a type of pipeline glue that eliminates a certain set of boilerplate
/// code for when you want to emit the events produced through the application of a command
/// immediately to a store, for a given event stream name. You don't have to build a dispatcher
/// yourself, you can use a derive macro to make a placeholder struct your dispatcher.
/// The result of a dispatch is a vector capturing the success of command application. If it
/// succeeded, you will get a CloudEvent, a CloudEvents v0.1 spec-compliant data structure.
#[async_trait]
pub trait Dispatcher {
    type Command;
    type Event: Event;
    type State: AggregateState + Clone;
    type Aggregate: Aggregate<Event = Self::Event, Command = Self::Command, State = Self::State>;

    async fn dispatch(
        state: Self::State,
        cmd: Self::Command,
        store: impl EventStoreClient,
        stream: String,
        evt_meta: EventMeta,
    ) -> Vec<Result<CloudEvent>>;
}

pub mod cloudevents;

pub mod eventstore;
pub mod prelude;
