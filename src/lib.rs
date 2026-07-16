//! Typed binary contract for the Nomos daemon.
use rkyv::{Archive,Deserialize,Serialize};
use signal_sema_storage::{ContentHash,FixtureScope,SlotIdentifier,SlotSummary};
#[derive(Archive,Serialize,Deserialize,Clone,Debug,PartialEq,Eq)] pub enum Request { Transform { scope: FixtureScope, schema: ContentHash, output_slot: SlotIdentifier }, List { scope: FixtureScope }, Subscribe { scope: FixtureScope } }
#[derive(Archive,Serialize,Deserialize,Clone,Debug,PartialEq,Eq)] pub enum Reply { Transformed(SlotSummary), Listed(Vec<SlotSummary>), Subscribed, Rejected(Rejection) }
#[derive(Archive,Serialize,Deserialize,Clone,Copy,Debug,PartialEq,Eq)] pub enum Rejection { SchemaNotFound, WrongDocumentKind, LoweringFailed, StorageFailed }
pub fn encode_request(value:&Request)->Result<Vec<u8>,String>{rkyv::to_bytes::<rkyv::rancor::Error>(value).map(|b|b.to_vec()).map_err(|e|e.to_string())}
