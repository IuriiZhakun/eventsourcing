//! Event store trait and implementations
use super::cloudevents::CloudEvent;
use super::{Event, EventEnvelope, EventMeta, Result};

pub use self::inmemory::MemoryEventStore;

pub use self::eventstoredb::EventStoreDBClient;

use async_trait::async_trait;

#[async_trait]
/// Trait required for event stores. For the moment, event stores are append-only
pub trait EventStoreClient: Send + Sync {
    async fn append(
        &self,
        evt: impl Event,
        stream: &str,
        evt_meta: EventMeta,
    ) -> Result<CloudEvent>;
    async fn get_all(&self, stream: &str) -> Result<Vec<CloudEvent>>;
    async fn get_all_e<E: Event>(&self, stream: &str) -> Result<Vec<E>>;
    async fn get_all_ee<E: Event>(&self, stream: &str) -> Result<Vec<EventEnvelope<E>>>;
    async fn append_envelope<E: Event>(
        &self,
        evt: EventEnvelope<E>,
        stream: &str,
    ) -> Result<EventEnvelope<E>>;
}

mod eventstoredb;
mod inmemory;
