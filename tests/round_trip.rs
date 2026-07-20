use signal_nomos::{Error, Reply, Request, encode_reply, encode_request};
use signal_sema_storage::{
    ContentHash, DocumentKey, DocumentKind, FixtureScope, SlotIdentifier, SlotSummary, Version,
};

#[test]
fn request_and_reply_round_trip_through_typed_codec_boundary() {
    let value = Request::Transform {
        scope: FixtureScope(1),
        schema: ContentHash([1; 32]),
        output_slot: SlotIdentifier(2),
    };
    let bytes = match encode_request(&value) {
        Ok(bytes) => bytes,
        Err(Error::Encoding(source)) => panic!("typed archive error: {source}"),
    };
    assert_eq!(
        rkyv::from_bytes::<Request, rkyv::rancor::Error>(&bytes).unwrap(),
        value
    );

    let reply = Reply::Subscribed {
        identifier: signal_sema_storage::SubscriptionIdentifier(3),
        initial: vec![SlotSummary {
            key: DocumentKey {
                scope: FixtureScope(1),
                kind: DocumentKind::Logos,
                slot: SlotIdentifier(2),
            },
            version: Version(4),
            hash: ContentHash([5; 32]),
        }],
    };
    let bytes = match encode_reply(&reply) {
        Ok(bytes) => bytes,
        Err(Error::Encoding(source)) => panic!("typed archive error: {source}"),
    };
    assert_eq!(
        rkyv::from_bytes::<Reply, rkyv::rancor::Error>(&bytes).unwrap(),
        reply
    );
}
