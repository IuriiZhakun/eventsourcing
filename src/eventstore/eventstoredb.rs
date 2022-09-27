//! Implementation of Greg Young's Event Store (eventstore.org)

#[cfg(feature = "orgeventstore")]
use super::super::cloudevents::CloudEvent;
use super::super::{Error, Event, Kind, Result};
#[cfg(feature = "orgeventstore")]
use super::EventStoreClient;
use eventstore::{Client, ClientSettings, EventData, WriteResult};


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

//#[async_trait(?Send)]
#[async_trait]
impl EventStoreClient for EventStoreDBClient {
    async fn get_all(&self, stream: &str) -> Result<Vec<CloudEvent>> {
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
                                println!("no events for {:?}", stream)
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

    async fn append(&self, evt: impl Event, stream: &str) -> Result<CloudEvent> {
        let ce: CloudEvent = evt.into();

        let e = EventData::json(ce.event_type.to_owned(), &ce).unwrap();

        let res = self
            .client
            .append_to_stream(stream, &Default::default(), e)
            .await;
        match res {
            Ok(WriteResult {
                next_expected_version: _,
                position: _,
            }) => {
                println!("wrote {:?} ", ce);
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
}
