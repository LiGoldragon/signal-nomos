//! Typed binary contract for the Nomos daemon.
mod error;

pub use error::Error;
use rkyv::{Archive, Deserialize, Serialize};
use signal_sema_storage::{
    ContentHash, FixtureScope, SlotIdentifier, SlotSummary, SubscriptionIdentifier,
};
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Transform {
        scope: FixtureScope,
        schema: ContentHash,
        output_slot: SlotIdentifier,
    },
    List {
        scope: FixtureScope,
    },
    Subscribe {
        scope: FixtureScope,
    },
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransformEvent {
    pub schema: ContentHash,
    pub logos: SlotSummary,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Transformed(SlotSummary),
    Listed(Vec<SlotSummary>),
    Subscribed {
        identifier: SubscriptionIdentifier,
        initial: Vec<SlotSummary>,
    },
    Event(TransformEvent),
    Rejected(Rejection),
}
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    SchemaNotFound,
    WrongDocumentKind,
    LoweringFailed,
    StorageFailed,
}
pub fn encode_request(value: &Request) -> Result<Vec<u8>, Error> {
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map(|bytes| bytes.to_vec())
        .map_err(Error::from)
}
pub fn encode_reply(value: &Reply) -> Result<Vec<u8>, Error> {
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map(|bytes| bytes.to_vec())
        .map_err(Error::from)
}
