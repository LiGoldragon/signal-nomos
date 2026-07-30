use core_nomos::{MacroKind, SectionDefault, TemplateFutureKind};
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

#[test]
fn d47_core_readers_refuse_the_appended_recursive_and_insert_at_tags() {
    let recursive = rkyv::to_bytes::<rkyv::rancor::Error>(&MacroKind::Recursive {
        section: SectionDefault::Enumeration,
    })
    .expect("archive current Recursive tag");
    assert!(
        rkyv::from_bytes::<d47_core_nomos::MacroKind, rkyv::rancor::Error>(&recursive).is_err(),
        "d47 MacroKind reader must refuse the appended Recursive tag"
    );

    let insert_at = rkyv::to_bytes::<rkyv::rancor::Error>(&TemplateFutureKind::InsertAt)
        .expect("archive current InsertAt tag");
    assert!(
        rkyv::from_bytes::<d47_core_nomos::TemplateFutureKind, rkyv::rancor::Error>(&insert_at)
            .is_err(),
        "d47 TemplateFutureKind reader must refuse the appended InsertAt tag"
    );
}
