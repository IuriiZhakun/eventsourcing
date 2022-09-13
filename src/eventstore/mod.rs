//! Event store trait and implementations
#[cfg(feature = "eventstore")]
use super::cloudevents::CloudEvent;
use super::{Dispatcher, Event, Result};

#[cfg(feature = "eventstore")]
pub use self::inmemory::MemoryEventStore;

pub use self::eventstoredb::EventStoreDBClient;
#[cfg(feature = "orgeventstore")]
pub use self::orgeventstore::OrgEventStore;

use async_trait::async_trait;

#[cfg(feature = "eventstore")]
#[async_trait(?Send)]
/// Trait required for event stores. For the moment, event stores are append-only
pub trait EventStoreClient {
    async fn append(&self, evt: impl Event, stream: &str) -> Result<CloudEvent>;
}

mod eventstoredb;
mod inmemory;
#[cfg(feature = "orgeventstore")]
mod orgeventstore;
