use capsule_content_identity::{CapsuleIdentity, CapsuleIdentityPreimage};
use core_nomos::{
    AuthoredTransformerSet, GenerationClass, LoadedNomosPopulation, NameTreeProjectionVersion,
    NomosNameTable,
};
use signal_nomos::{
    AdmissionSnapshot, CapsuleSelector, EthosPopulationArchive, GenerationSelection,
    NomosDeploymentArtifacts, NomosProjectionArchive, NomosSlotId, Reply, Request,
    ShortCapsuleDisplay, SlotExpectation, SlotGeneration, TransformOutcome, TransformSelector,
    encode_reply, encode_request,
};

fn population(version: NameTreeProjectionVersion) -> core_nomos::SealedNomosPopulation {
    LoadedNomosPopulation::from_typed(
        AuthoredTransformerSet::try_new(Vec::new()).expect("empty authored set"),
        NomosNameTable::empty(),
    )
    .seal(version)
    .expect("empty population seals")
}

fn artifacts() -> NomosDeploymentArtifacts {
    let population = population(NameTreeProjectionVersion::initial());
    NomosDeploymentArtifacts::from_population(&population).expect("canonical deployment artifacts")
}

#[test]
fn deploy_round_trips_and_revalidates_canonical_artifacts() {
    let value = Request::Deploy {
        slot: NomosSlotId::new(7),
        expected: SlotExpectation::Empty,
        artifacts: artifacts(),
        selection: GenerationSelection::enriched(),
    };
    value.validate().expect("constructed request validates");
    let bytes = encode_request(&value).expect("request archive");
    let restored =
        rkyv::from_bytes::<Request, rkyv::rancor::Error>(&bytes).expect("typed request restores");
    restored
        .validate()
        .expect("wire-restored request revalidates");
    assert_eq!(restored, value);
    let Request::Deploy { selection, .. } = restored else {
        panic!("deploy request");
    };
    assert_eq!(selection.classes(), GenerationClass::ENRICHED_ORDER);
}

#[test]
fn malformed_artifacts_and_non_nomos_selectors_refuse() {
    assert!(signal_nomos::NomosCapsuleArchive::try_new(vec![1, 2, 3]).is_err());
    assert!(ShortCapsuleDisplay::try_new("ABC0").is_err());
    assert!(ShortCapsuleDisplay::try_new("abc").is_err());
    assert!(EthosPopulationArchive::try_new(Vec::new()).is_err());

    let wrong =
        CapsuleIdentity::ethos(CapsuleIdentityPreimage::new([1]).derive_content_addressed_hash());
    let request = Request::Transform {
        selector: TransformSelector::Seated {
            slot: NomosSlotId::new(1),
            capsule: CapsuleSelector::Full(wrong),
        },
        ethos: EthosPopulationArchive::try_new(vec![1]).expect("non-empty boundary archive"),
    };
    assert!(request.validate().is_err());
}

#[test]
fn wire_tamper_reaches_canonical_capsule_refusal() {
    let original = artifacts();
    let mut capsule = original.capsule().bytes().to_vec();
    let projection = original.projection().clone();
    assert!(
        capsule.iter().any(|byte| *byte != 0),
        "test archive has a mutable byte"
    );
    let index = capsule
        .iter()
        .position(|byte| *byte != 0)
        .expect("nonzero byte");
    capsule[index] ^= 1;
    assert!(signal_nomos::NomosCapsuleArchive::try_new(capsule).is_err());
    assert!(projection.restore().is_ok());
}

#[test]
fn transformed_reply_round_trips_and_revalidates_checked_archive_shape() {
    let identity = artifacts()
        .validate()
        .expect("artifacts validate")
        .capsule()
        .content_identity();
    let reply = Reply::Transformed(
        TransformOutcome::new(
            AdmissionSnapshot::new(
                NomosSlotId::new(2),
                identity,
                SlotGeneration::new(3),
                NameTreeProjectionVersion::new(4),
            ),
            vec![9],
        )
        .expect("non-empty checked archive"),
    );
    let bytes = encode_reply(&reply).expect("reply archive");
    let restored =
        rkyv::from_bytes::<Reply, rkyv::rancor::Error>(&bytes).expect("typed reply restores");
    restored.validate().expect("reply revalidates");
    assert_eq!(restored, reply);
}

#[test]
fn projection_advance_requires_the_exact_successor_version() {
    let initial = population(NameTreeProjectionVersion::initial());
    let successor = population(NameTreeProjectionVersion::new(1));
    assert_eq!(
        initial.capsule().content_identity(),
        successor.capsule().content_identity()
    );

    let request = Request::AdvanceProjection {
        capsule: initial.capsule().content_identity(),
        expected_previous_version: NameTreeProjectionVersion::initial(),
        projection: NomosProjectionArchive::from_projection(successor.projection())
            .expect("successor projection archive"),
        translator_receipt: None,
    };
    request.validate().expect("exact successor validates");

    let stale = Request::AdvanceProjection {
        capsule: initial.capsule().content_identity(),
        expected_previous_version: NameTreeProjectionVersion::initial(),
        projection: NomosProjectionArchive::from_projection(initial.projection())
            .expect("initial projection archive"),
        translator_receipt: None,
    };
    assert!(matches!(
        stale.validate(),
        Err(signal_nomos::Error::ProjectionVersionMismatch)
    ));

    let overflow = Request::AdvanceProjection {
        capsule: initial.capsule().content_identity(),
        expected_previous_version: NameTreeProjectionVersion::new(u64::MAX),
        projection: NomosProjectionArchive::from_projection(initial.projection())
            .expect("initial projection archive"),
        translator_receipt: None,
    };
    assert!(matches!(
        overflow.validate(),
        Err(signal_nomos::Error::ProjectionVersionOverflow)
    ));
    assert_eq!(
        SlotGeneration::new(8).checked_successor(),
        Some(SlotGeneration::new(9))
    );
    assert_eq!(SlotGeneration::new(u64::MAX).checked_successor(), None);
}

#[test]
fn contract_source_has_no_daemon_storage_or_legacy_evaluator_reachability() {
    let source = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");
    for forbidden in [
        "signal_sema_storage",
        "sema_engine",
        "MacroPackage",
        "apply_enriched",
        "kameo",
        "tokio",
    ] {
        assert!(
            !source.contains(forbidden) && !manifest.contains(forbidden),
            "pure wire contract must not contain {forbidden}"
        );
    }
}
