extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate eventsourcing;
extern crate serde_json;
#[macro_use]
extern crate eventsourcing_derive;
use tokio;

use async_trait::async_trait;
use eventsourcing::{eventstore::MemoryEventStore, prelude::*, Result};

const DOMAIN_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq)]
struct LocationData {
    lat: f32,
    long: f32,
    alt: f32,
    generation: u64,
}

impl AggregateState for LocationData {
    fn generation(&self) -> u64 {
        self.generation
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Event)]
#[event_source("events://github.com/pholactery/eventsourcing/samples/location")]
#[event_type_version(DOMAIN_VERSION)]
enum LocationEvent {
    LocationUpdated { lat: f32, long: f32, alt: f32 },
}

#[derive(Serialize, Deserialize, Default)]
struct Location;

enum LocationCommand {
    UpdateLocation { lat: f32, long: f32, alt: f32 },
}

#[async_trait]
impl Aggregate for Location {
    type Event = LocationEvent;
    type Command = LocationCommand;
    type State = LocationData;

    fn apply_event(state: &Self::State, evt: &Self::Event) -> Result<Self::State> {
        let ld = match *evt {
            LocationEvent::LocationUpdated { lat, long, alt } => LocationData {
                lat,
                long,
                alt,
                generation: state.generation + 1,
            },
        };
        Ok(ld)
    }

    fn handle_command(_state: &Self::State, cmd: &Self::Command) -> Result<Vec<Self::Event>> {
        // SHOULD DO: validate state and command
        let evt = match *cmd {
            LocationCommand::UpdateLocation { lat, long, alt } => {
                LocationEvent::LocationUpdated { lat, long, alt }
            }
        };

        // if validation passes...
        Ok(vec![evt])
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let location_store = MemoryEventStore::new();
    let (lat, long, alt) = (10.0, 52.0, 31.0);

    let update = LocationCommand::UpdateLocation { lat, long, alt };
    let old_state = LocationData {
        lat: 57.06,
        long: 36.07,
        alt: 15.0,
        generation: 0,
    };
    // First, handle a command to get an event vector
    let res = Location::handle_command(&old_state, &update)?;
    // Second, apply the events to get a new state
    let state = Location::apply_all(&old_state, &res)?;
    // Third, append to store (can do this alternatively with a dispatcher)
    let store_result = location_store
        .append(res[0].clone(), "locations", EventMeta::default())
        .await?;
    println!("Store result: {:?}", store_result);

    println!("Original state: {:?}", old_state);
    println!("Post-process state: {:?}", state);

    assert_eq!(
        state,
        LocationData {
            lat,
            long,
            alt,
            generation: 1
        }
    );

    println!(
        "all events - {:#?}",
        location_store
            .get_all("locationevent.locationupdated")
            .await?
    );

    Ok(())
}
