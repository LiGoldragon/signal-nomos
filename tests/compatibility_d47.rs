use signal_nomos::{Request, encode_request};

const DEPLOY: &[u8] = include_bytes!("goldens/d47_deploy_request.bin");

#[test]
fn d47_deploy_request_restores_validates_and_reserializes_byte_exact() {
    let request = rkyv::from_bytes::<Request, rkyv::rancor::Error>(DEPLOY)
        .expect("d47 Deploy request restores");
    request.validate().expect("d47 Deploy request revalidates");
    assert!(matches!(request, Request::Deploy { .. }));
    assert_eq!(
        encode_request(&request).expect("d47 Deploy request reserializes"),
        DEPLOY
    );
}
