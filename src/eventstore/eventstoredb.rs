//! Implementation of Greg Young's Event Store (eventstore.org)

use crate::{EventEnvelope, EventMeta, LID};

use super::super::cloudevents::CloudEvent;
use super::super::Result as EVResult;
use super::super::{Error, Event, Kind};
use super::EventStoreClient;
use eventstore::{Client, ClientSettings, EventData, WriteResult};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Client for the eventstoredb Event Store
#[derive(Clone)]
pub struct EventStoreDBClient {
    client: Client,
}

fn settings(host: &str, port: u16, user: &str, password: &str) -> ClientSettings {
    format!(
            "esdb://{}{}@{}:{}?tls=false&defaultDeadline=100000&tlsVerifyCert=false&secure=false&discoveryInterval=1000&maxDiscoveryAttempts=5",
            user,
            password,
            host,
            port
        )
        .parse()
        .unwrap()
}

impl EventStoreDBClient {
    /// Creates a new event store client with the given host name and port number.
    pub fn new(host: &str, port: u16) -> EventStoreDBClient {
        EventStoreDBClient {
            client: Client::new(settings(host, port, "", "")).unwrap(),
        }
    }
}

impl Default for EventStoreDBClient {
    /// Creates an event store client pointing to localhost:2113, the default address
    fn default() -> Self {
        EventStoreDBClient::new("localhost", 2113)
    }
}

use async_trait::async_trait;

#[async_trait]
impl EventStoreClient for EventStoreDBClient {
    //TODO: fix dup
    async fn get_all_ee<E: Event>(&self, stream: &str) -> EVResult<Vec<EventEnvelope<E>>> {
        let res = self.client.read_stream(stream, &Default::default()).await;
        match res {
            Ok(mut stream) => {
                let mut r: Vec<EventEnvelope<E>> = Vec::new();
                loop {
                    match stream.next().await {
                        Ok(opt) => match opt {
                            Some(e) => {
                                let oe = e.get_original_event();
                                match oe.as_json::<Vec<E>>() {
                                    Ok(event) => {
                                        let mut metadata = oe.metadata.clone();
                                        let lid = if metadata.contains_key("uid") {
                                            LID {
                                                id: Uuid::parse_str(
                                                    &metadata.remove("uid").unwrap(),
                                                )
                                                .unwrap(),
                                                links: if metadata.contains_key("links") {
                                                    let links = metadata.remove("links").unwrap();
                                                    links
                                                        .split(",")
                                                        .map(|x| Uuid::parse_str(x).unwrap())
                                                        .collect()
                                                } else {
                                                    Vec::new()
                                                },
                                            }
                                        } else {
                                            LID::default()
                                        };

                                        let ee = EventEnvelope {
                                            payload: event,
                                            lid,
                                            metadata,
                                        };
                                        r.push(ee);
                                    }
                                    Err(err) => {
                                        return Err(Error {
                                            kind: Kind::StoreFailure(format!(
                                            "Failed to read event {:?} from stream {:?} error {:?}",
                                            e, stream, err
                                        )),
                                        })
                                    }
                                }
                            }
                            None => break,
                        },
                        Err(err) => match err {
                            eventstore::Error::ResourceNotFound => {
                                info!("no events for {:?}", stream)
                            }
                            _ => {
                                return Err(Error {
                                    kind: Kind::StoreFailure(format!(
                                        "Failed to read from stream {:?} error {:?}",
                                        stream, err
                                    )),
                                });
                            }
                        },
                    }
                }
                Ok(r)
            }
            Err(kind) => Err(Error {
                kind: Kind::StoreFailure(format!(
                    "Failed to read from event store stream {:?} error {:?}",
                    stream, kind
                )),
            }),
        }
    }

    async fn get_all_e<E: Event>(&self, stream: &str) -> EVResult<Vec<E>> {
        let res = self.client.read_stream(stream, &Default::default()).await;
        match res {
            Ok(mut stream) => {
                let mut r: Vec<E> = Vec::new();
                loop {
                    match stream.next().await {
                        Ok(opt) => match opt {
                            Some(e) => {
                                let oe = e.get_original_event();
                                match oe.as_json::<E>() {
                                    Ok(event) => {
                                        r.push(event);
                                    }
                                    Err(err) => {
                                        return Err(Error {
                                            kind: Kind::StoreFailure(format!(
                                            "Failed to read event {:?} from stream {:?} error {:?}",
                                            e, stream, err
                                        )),
                                        })
                                    }
                                }
                            }
                            None => break,
                        },
                        Err(err) => match err {
                            eventstore::Error::ResourceNotFound => {
                                info!("no events for {:?}", stream)
                            }
                            _ => {
                                return Err(Error {
                                    kind: Kind::StoreFailure(format!(
                                        "Failed to read from stream {:?} error {:?}",
                                        stream, err
                                    )),
                                });
                            }
                        },
                    }
                }
                Ok(r)
            }
            Err(kind) => Err(Error {
                kind: Kind::StoreFailure(format!(
                    "Failed to read from event store stream {:?} error {:?}",
                    stream, kind
                )),
            }),
        }
    }

    async fn get_all(&self, stream: &str) -> EVResult<Vec<CloudEvent>> {
        let res = self.client.read_stream(stream, &Default::default()).await;
        match res {
            Ok(mut stream) => {
                let mut r: Vec<CloudEvent> = Vec::new();
                loop {
                    match stream.next().await {
                        Ok(opt) => match opt {
                            Some(e) => match e.get_original_event().as_json::<CloudEvent>() {
                                Ok(event) => r.push(event),
                                Err(err) => {
                                    return Err(Error {
                                        kind: Kind::StoreFailure(format!(
                                            "Failed to read event {:?} from stream {:?} error {:?}",
                                            e, stream, err
                                        )),
                                    })
                                }
                            },
                            None => break,
                        },
                        Err(err) => match err {
                            eventstore::Error::ResourceNotFound => {
                                info!("no events for {:?}", stream)
                            }
                            _ => {
                                return Err(Error {
                                    kind: Kind::StoreFailure(format!(
                                        "Failed to read from stream {:?} error {:?}",
                                        stream, err
                                    )),
                                });
                            }
                        },
                    }
                }
                Ok(r)
            }
            Err(kind) => Err(Error {
                kind: Kind::StoreFailure(format!(
                    "Failed to read from event store stream {:?} error {:?}",
                    stream, kind
                )),
            }),
        }
    }

    async fn append(
        &self,
        evt: impl Event,
        stream: &str,
        evt_meta: EventMeta,
    ) -> EVResult<CloudEvent> {
        let ce: CloudEvent = evt.into();

        let e = EventData::json(ce.event_type.to_owned(), &ce)
            .unwrap()
            .metadata_as_json(evt_meta)
            .unwrap();

        let res = self
            .client
            .append_to_stream(stream, &Default::default(), e)
            .await;
        match res {
            Ok(WriteResult {
                next_expected_version: _,
                position: _,
            }) => {
                debug!("wrote {:?} ", ce);
                Ok(ce)
            }
            Err(kind) => Err(Error {
                kind: Kind::StoreFailure(format!(
                    "Failed to post to event store {:?} error {:?}",
                    ce, kind
                )),
            }),
        }
    }

    async fn append_envelope<E: Event>(
        &self,
        evt: EventEnvelope<E>,
        stream: &str,
    ) -> EVResult<EventEnvelope<E>> {
        let lid: HashMap<String, String> = evt.lid.clone().into();
        let mut meta = evt.metadata.clone();
        meta.extend(lid);

        //let raw_data = serde_json::to_string(&source).unwrap();
        let re: Result<Vec<_>, serde_json::Error> = evt
            //let e: Vec<EventData> = evt
            .payload
            .iter()
            .map(|x| {
                let r = EventData::json(x.event_type().to_owned(), &x)
                    .and_then(|e| e.metadata_as_json(meta.clone()));
                r
            })
            .collect();

        let e = re.map_err(|e| Error {
            kind: Kind::StoreFailure(format!("Failed to parse {:?} error {:?}", evt, e,)),
        })?;
        let max_retries = 3;
        let mut retries = 0;
        loop {
            let res = self
                .client
                .append_to_stream(stream, &Default::default(), e.clone())
                .await;
            match res {
                Ok(WriteResult {
                    next_expected_version: _,
                    position: _,
                }) => {
                    debug!("wrote {:?} ", e);
                    return Ok(evt);
                }
                Err(kind) => {
                    let msg = format!("Failed to post to event store {:?} error {:?}", evt, kind);
                    if retries < max_retries {
                        warn!(msg);
                        retries += 1;
                        sleep(Duration::from_millis(1000)).await;
                    } else {
                        return Err(Error {
                            kind: Kind::StoreFailure(msg),
                        });
                    }
                }
            }
        }
    }
}
