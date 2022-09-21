//! Event store trait and implementations
#[cfg(feature = "eventstore")]
use super::cloudevents::CloudEvent;
use super::{Event, Result};

#[cfg(feature = "eventstore")]
pub use self::inmemory::MemoryEventStore;

pub use self::eventstoredb::EventStoreDBClient;

use async_trait::async_trait;

#[cfg(feature = "eventstore")]
#[async_trait]
//#[async_trait(?Send)]
/// Trait required for event stores. For the moment, event stores are append-only
pub trait EventStoreClient {
    async fn append(&self, evt: impl Event, stream: &str) -> Result<CloudEvent>;
    async fn get_all(&self, stream: &str) -> Result<Vec<CloudEvent>>;
}

mod eventstoredb;
mod inmemory;
