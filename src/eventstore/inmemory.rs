//! In-Memory Event Store
//!
//! This module provides an implementation of the event store trait for a simple in-memory
//! cache. This is not an event store you should be using for production and we recommend
//! it is recommended that you only use this for testing/demonstration purposes.

#[cfg(feature = "eventstore")]
use super::super::cloudevents::CloudEvent;
use super::super::Event;
use super::super::EventEnvelope;
use super::super::EventMeta;
use super::super::Result;
#[cfg(feature = "eventstore")]
use super::EventStoreClient;
use chrono::prelude::*;
use std::sync::Mutex;
#[cfg(feature = "eventstore")]
/// An simple, in-memory implementation of the event store trait
pub struct MemoryEventStore {
    evts: Mutex<Vec<CloudEvent>>,
}
#[cfg(feature = "eventstore")]
impl MemoryEventStore {
    /// Creates a new in-memory event store. The resulting store is thread-safe.
    pub fn new() -> MemoryEventStore {
        MemoryEventStore {
            evts: Mutex::new(Vec::<CloudEvent>::new()),
        }
    }
}

use async_trait::async_trait;
#[cfg(feature = "eventstore")]
//#[async_trait(?Send)]
#[async_trait]
impl EventStoreClient for MemoryEventStore {
    async fn append_envelope<E: Event>(
        &self,
        evt: EventEnvelope<E>,
        _stream: &str,
    ) -> Result<EventEnvelope<E>> {
        Ok(evt)
    }

    /// Appends an event to the in-memory store
    async fn append(
        &self,
        evt: impl Event,
        _stream: &str,
        _evt_meta: EventMeta,
    ) -> Result<CloudEvent> {
        let mut guard = self.evts.lock().unwrap();
        let cloud_event = CloudEvent::from(evt);
        guard.push(cloud_event.clone());
        Ok(cloud_event)
    }
    async fn get_all(&self, stream: &str) -> Result<Vec<CloudEvent>> {
        let guard = self.evts.lock().unwrap();
        let matches = guard
            .iter()
            .filter(|evt| evt.event_type == stream)
            .cloned()
            .collect();

        Ok(matches)
    }

    async fn get_all_e<E: Event>(&self, _stream: &str) -> Result<Vec<E>> {
        Ok(Vec::new())
    }
    async fn get_all_ee<E: Event>(&self, _stream: &str) -> Result<Vec<EventEnvelope<E>>> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "eventstore")]
impl MemoryEventStore {
    pub fn get_from(&self, event_type: &str, start: DateTime<Utc>) -> Result<Vec<CloudEvent>> {
        let guard = self.evts.lock().unwrap();
        let matches = guard
            .iter()
            .filter(|evt| evt.event_type == event_type && evt.event_time >= start)
            .cloned()
            .collect();
        Ok(matches)
    }

    pub fn get_range(
        &self,
        event_type: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CloudEvent>> {
        let guard = self.evts.lock().unwrap();
        let matches = guard
            .iter()
            .filter(|evt| {
                evt.event_type == event_type && evt.event_time >= start && evt.event_time <= end
            })
            .cloned()
            .collect();
        Ok(matches)
    }
}
