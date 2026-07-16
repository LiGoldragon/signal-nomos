use signal_nomos::{Error, Rejection, Reply, Request, encode_reply, encode_request};
use signal_sema_storage::{ContentHash, FixtureScope, SlotIdentifier};

#[test]
fn transform_round_trips_through_typed_codec_boundary() {
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

    let reply: Result<Vec<u8>, Error> = encode_reply(&Reply::Rejected(Rejection::LoweringFailed));
    assert!(reply.is_ok());
}
