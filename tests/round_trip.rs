use capsule_content_identity::{CapsuleIdentity, CapsuleIdentityPreimage};
use signal_nomos::{
    CapsuleSelector, NomosSlotId, ShortCapsuleDisplay, SlotExpectation, SlotGeneration,
    WireInvariant,
};

#[test]
fn independently_owned_seating_vocabulary_round_trips() {
    let selector = CapsuleSelector::Short(
        ShortCapsuleDisplay::try_new("0123abcd").expect("valid short display"),
    );
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&selector).expect("selector archives");
    let restored = rkyv::from_bytes::<CapsuleSelector, rkyv::rancor::Error>(&bytes)
        .expect("selector restores");

    restored.validate().expect("restored selector validates");
    assert_eq!(restored, selector);
    assert_eq!(NomosSlotId::new(7).value(), 7);
    assert_eq!(SlotExpectation::Empty, SlotExpectation::Empty);
    assert_eq!(
        SlotExpectation::Generation(SlotGeneration::new(3)),
        SlotExpectation::Generation(SlotGeneration::new(3))
    );
}

#[test]
fn short_displays_and_capsule_kind_are_revalidated() {
    let valid = ShortCapsuleDisplay::try_new("deadbeef").expect("lowercase hexadecimal display");
    assert_eq!(valid.as_str(), "deadbeef");
    assert!(ShortCapsuleDisplay::try_new("abc").is_err());
    assert!(ShortCapsuleDisplay::try_new("ABC0").is_err());
    assert!(ShortCapsuleDisplay::try_new("0123_").is_err());

    let wrong =
        CapsuleIdentity::ethos(CapsuleIdentityPreimage::new([1]).derive_content_addressed_hash());
    assert!(CapsuleSelector::Full(wrong).validate().is_err());
}

#[test]
fn slot_generation_successor_is_checked() {
    assert_eq!(SlotGeneration::initial().value(), 0);
    assert_eq!(
        SlotGeneration::new(8).checked_successor(),
        Some(SlotGeneration::new(9))
    );
    assert_eq!(SlotGeneration::new(u64::MAX).checked_successor(), None);
}

#[test]
fn manifest_and_source_have_no_retired_nomos_world() {
    let manifest = include_str!("../Cargo.toml");
    let source = include_str!("../src/lib.rs");
    for forbidden in [
        "core-nomos",
        "d47-core-nomos",
        "SealedNomos",
        "GenerationSelection",
        "NomosDeploymentArtifacts",
        "Request",
        "Reply",
        "Deploy",
        "AdvanceProjection",
        "Transform",
    ] {
        assert!(
            !manifest.contains(forbidden) && !source.contains(forbidden),
            "retired ownership must not contain {forbidden}"
        );
    }
}
