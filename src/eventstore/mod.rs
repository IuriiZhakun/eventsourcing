//! Event store trait and implementations
use super::cloudevents::CloudEvent;
use super::{Event, Result};

pub use self::inmemory::MemoryEventStore;

pub use self::eventstoredb::EventStoreDBClient;

use async_trait::async_trait;

#[async_trait]
//#[async_trait(?Send)]
/// Trait required for event stores. For the moment, event stores are append-only
pub trait EventStoreClient: Send + Sync {
    async fn append(&self, evt: impl Event, stream: &str) -> Result<CloudEvent>;
    async fn get_all(&self, stream: &str) -> Result<Vec<CloudEvent>>;
}

mod eventstoredb;
mod inmemory;
