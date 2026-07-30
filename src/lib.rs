//! Pure typed binary contract for the stateful Nomos daemon.
//!
//! This crate owns wire-safe values and canonical archive validation only. The
//! daemon owns persistence, authorization, evaluation, recovery, and slot
//! policy.

mod error;

pub use error::Error;

use capsule_content_identity::CapsuleIdentity;
use core_nomos::{
    AuthenticatedNameTreeProjection, GenerationClass, NameTreeProjectionVersion,
    SealedNomosCapsule, SealedNomosPopulation,
};
use rkyv::{Archive, Deserialize, Serialize};

/// Component-owned opaque Nomos slot identity.
///
/// The domain and implicit first-deploy creation rule are
/// `[to-be-reviewed-by-psyche]`.
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct NomosSlotId(u64);

impl NomosSlotId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Monotone version of one live slot binding.
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct SlotGeneration(u64);

impl SlotGeneration {
    pub const fn initial() -> Self {
        Self(0)
    }

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn checked_successor(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

/// Compare-and-set expectation for a deployment.
///
/// CAS at the public operation boundary is `[to-be-reviewed-by-psyche]`.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotExpectation {
    Empty,
    Generation(SlotGeneration),
}

/// Ordered, duplicate-preserving temporary generation metadata.
///
/// It is separate from sealed Nomos content and lives per binding
/// `[to-be-reviewed-by-psyche]`; po2.8 owns retiring it.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct GenerationSelection(Vec<GenerationClass>);

impl GenerationSelection {
    pub fn new(classes: Vec<GenerationClass>) -> Self {
        Self(classes)
    }

    pub fn enriched() -> Self {
        Self(GenerationClass::ENRICHED_ORDER.to_vec())
    }

    pub fn classes(&self) -> &[GenerationClass] {
        &self.0
    }
}

/// Canonical immutable Nomos Capsule archive bytes.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NomosCapsuleArchive(Vec<u8>);

impl NomosCapsuleArchive {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
        SealedNomosCapsule::from_archive_bytes(&bytes)?;
        Ok(Self(bytes))
    }

    pub fn from_capsule(capsule: &SealedNomosCapsule) -> Result<Self, Error> {
        Self::try_new(capsule.to_archive_bytes()?)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn restore(&self) -> Result<SealedNomosCapsule, Error> {
        Ok(SealedNomosCapsule::from_archive_bytes(self.bytes())?)
    }
}

/// Canonical authenticated NameTree projection archive bytes.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NomosProjectionArchive(Vec<u8>);

impl NomosProjectionArchive {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
        AuthenticatedNameTreeProjection::from_archive_bytes(&bytes)?;
        Ok(Self(bytes))
    }

    pub fn from_projection(projection: &AuthenticatedNameTreeProjection) -> Result<Self, Error> {
        Self::try_new(projection.to_archive_bytes()?)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn restore(&self) -> Result<AuthenticatedNameTreeProjection, Error> {
        Ok(AuthenticatedNameTreeProjection::from_archive_bytes(
            self.bytes(),
        )?)
    }
}

/// The two independently persisted, canonically authenticated deployment
/// artifacts.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NomosDeploymentArtifacts {
    capsule: NomosCapsuleArchive,
    projection: NomosProjectionArchive,
}

impl NomosDeploymentArtifacts {
    pub fn try_new(
        capsule: NomosCapsuleArchive,
        projection: NomosProjectionArchive,
    ) -> Result<Self, Error> {
        SealedNomosPopulation::from_archive_parts(capsule.bytes(), projection.bytes())?;
        Ok(Self {
            capsule,
            projection,
        })
    }

    pub fn from_population(population: &SealedNomosPopulation) -> Result<Self, Error> {
        Self::try_new(
            NomosCapsuleArchive::from_capsule(population.capsule())?,
            NomosProjectionArchive::from_projection(population.projection())?,
        )
    }

    pub const fn capsule(&self) -> &NomosCapsuleArchive {
        &self.capsule
    }

    pub const fn projection(&self) -> &NomosProjectionArchive {
        &self.projection
    }

    /// Revalidate values restored from an untrusted wire archive.
    pub fn validate(&self) -> Result<SealedNomosPopulation, Error> {
        Ok(SealedNomosPopulation::from_archive_parts(
            self.capsule.bytes(),
            self.projection.bytes(),
        )?)
    }
}

/// Canonical direct encoded Ethos-population/NameTree archive.
///
/// The concrete population is owned and semantically validated by the daemon;
/// the contract prevents empty placeholder payloads
/// `[to-be-reviewed-by-psyche]`.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct EthosPopulationArchive(Vec<u8>);

impl EthosPopulationArchive {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::EmptyEthosPopulationArchive);
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Ephemeral lowercase hexadecimal Capsule prefix.
///
/// It is wire-visible but never persisted. Minimum length four and slot-scoped
/// resolution are `[to-be-reviewed-by-psyche]`.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ShortCapsuleDisplay(String);

impl ShortCapsuleDisplay {
    pub fn try_new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if !(4..=64).contains(&value.len())
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Error::InvalidShortCapsuleDisplay);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Full persistent identity or an ephemeral slot-scoped short display.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum CapsuleSelector {
    Full(CapsuleIdentity),
    Short(ShortCapsuleDisplay),
}

impl CapsuleSelector {
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Full(CapsuleIdentity::Nomos(_)) => Ok(()),
            Self::Full(_) => Err(Error::WrongCapsuleKind),
            Self::Short(display) => {
                ShortCapsuleDisplay::try_new(display.as_str().to_owned()).map(|_| ())
            }
        }
    }
}

/// Transform against the admission-time live binding or a retained seat.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum TransformSelector {
    Live(NomosSlotId),
    Seated {
        slot: NomosSlotId,
        capsule: CapsuleSelector,
    },
}

/// Optional translator rename authority material.
///
/// The concrete translator receipt integration is not yet public. `None`
/// requires an authenticated configured admin Unix peer
/// `[to-be-reviewed-by-psyche]`; arbitrary caller bytes are never sufficient.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TranslatorRenameReceiptArchive(Vec<u8>);

impl TranslatorRenameReceiptArchive {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::EmptyTranslatorRenameReceipt);
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Nomos daemon operations.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Request {
    Deploy {
        slot: NomosSlotId,
        expected: SlotExpectation,
        artifacts: NomosDeploymentArtifacts,
        selection: GenerationSelection,
    },
    Repoint {
        slot: NomosSlotId,
        expected_generation: SlotGeneration,
        capsule: CapsuleSelector,
        selection: Option<GenerationSelection>,
    },
    AdvanceProjection {
        capsule: CapsuleIdentity,
        expected_previous_version: NameTreeProjectionVersion,
        projection: NomosProjectionArchive,
        translator_receipt: Option<TranslatorRenameReceiptArchive>,
    },
    Transform {
        selector: TransformSelector,
        ethos: EthosPopulationArchive,
    },
}

impl Request {
    /// Validate all relationships available without daemon state or authority.
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Deploy { artifacts, .. } => {
                artifacts.validate()?;
                Ok(())
            }
            Self::Repoint { capsule, .. } => capsule.validate(),
            Self::AdvanceProjection {
                capsule,
                expected_previous_version,
                projection,
                translator_receipt,
            } => {
                require_nomos(*capsule)?;
                let projection = projection.restore()?;
                if projection.content_identity() != *capsule {
                    return Err(Error::ProjectionContentMismatch);
                }
                let expected_version = expected_previous_version
                    .checked_successor()
                    .ok_or(Error::ProjectionVersionOverflow)?;
                if projection.version() != expected_version {
                    return Err(Error::ProjectionVersionMismatch);
                }
                if let Some(receipt) = translator_receipt {
                    TranslatorRenameReceiptArchive::try_new(receipt.bytes().to_vec())?;
                }
                Ok(())
            }
            Self::Transform { selector, ethos } => {
                if let TransformSelector::Seated { capsule, .. } = selector {
                    capsule.validate()?;
                }
                EthosPopulationArchive::try_new(ethos.bytes().to_vec())?;
                Ok(())
            }
        }
    }
}

/// Integrity-protected component commit marker projected onto the wire.
///
/// Its two-counter shape is `[to-be-reviewed-by-psyche]`.
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct CommitMarker {
    commit_sequence: u64,
    snapshot: u64,
}

impl CommitMarker {
    pub const fn new(commit_sequence: u64, snapshot: u64) -> Self {
        Self {
            commit_sequence,
            snapshot,
        }
    }

    pub const fn commit_sequence(self) -> u64 {
        self.commit_sequence
    }

    pub const fn snapshot(self) -> u64 {
        self.snapshot
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum DeployOutcome {
    FreshDeployed {
        slot: NomosSlotId,
        identity: CapsuleIdentity,
        generation: SlotGeneration,
        committed_at: CommitMarker,
    },
    Repointed {
        slot: NomosSlotId,
        previous_identity: CapsuleIdentity,
        identity: CapsuleIdentity,
        generation: SlotGeneration,
        committed_at: CommitMarker,
    },
    AlreadyCurrent {
        slot: NomosSlotId,
        identity: CapsuleIdentity,
        generation: SlotGeneration,
        binding_committed_at: CommitMarker,
        observed_at: CommitMarker,
    },
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ProjectionOutcome {
    Advanced {
        identity: CapsuleIdentity,
        previous_version: NameTreeProjectionVersion,
        version: NameTreeProjectionVersion,
        committed_at: CommitMarker,
    },
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AdmissionSnapshot {
    slot: NomosSlotId,
    identity: CapsuleIdentity,
    generation: SlotGeneration,
    projection_version: NameTreeProjectionVersion,
}

impl AdmissionSnapshot {
    pub const fn new(
        slot: NomosSlotId,
        identity: CapsuleIdentity,
        generation: SlotGeneration,
        projection_version: NameTreeProjectionVersion,
    ) -> Self {
        Self {
            slot,
            identity,
            generation,
            projection_version,
        }
    }

    pub const fn slot(&self) -> NomosSlotId {
        self.slot
    }

    pub const fn identity(&self) -> CapsuleIdentity {
        self.identity
    }

    pub const fn generation(&self) -> SlotGeneration {
        self.generation
    }

    pub const fn projection_version(&self) -> NameTreeProjectionVersion {
        self.projection_version
    }
}

/// Checked native Logos-population/NameTree archive bytes.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct LogosPopulationArchive(Vec<u8>);

impl LogosPopulationArchive {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::EmptyLogosPopulationArchive);
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TransformOutcome {
    snapshot: AdmissionSnapshot,
    logos_population: LogosPopulationArchive,
}

impl TransformOutcome {
    pub fn new(snapshot: AdmissionSnapshot, logos_population: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            snapshot,
            logos_population: LogosPopulationArchive::try_new(logos_population)?,
        })
    }

    pub const fn snapshot(&self) -> &AdmissionSnapshot {
        &self.snapshot
    }

    pub fn logos_population(&self) -> &[u8] {
        self.logos_population.bytes()
    }

    pub fn validate(&self) -> Result<(), Error> {
        LogosPopulationArchive::try_new(self.logos_population.bytes().to_vec()).map(|_| ())
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    Deployed(DeployOutcome),
    ProjectionAdvanced(ProjectionOutcome),
    Transformed(TransformOutcome),
    Rejected(Rejection),
}

impl Reply {
    /// Revalidate private-field invariants after untrusted wire restoration.
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Transformed(outcome) => outcome.validate(),
            Self::Deployed(_) | Self::ProjectionAdvanced(_) | Self::Rejected(_) => Ok(()),
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rejection {
    EmptySlot,
    SlotGenerationMismatch,
    CapsuleNotSeated,
    AmbiguousShortCapsuleDisplay,
    CapsuleArchiveInvalid,
    ProjectionInvalid,
    ProjectionStale,
    TranslatorReceiptUnsupported,
    Unauthorized,
    EthosPopulationInvalid,
    ReferenceUniverseInvalid,
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

fn require_nomos(identity: CapsuleIdentity) -> Result<(), Error> {
    match identity {
        CapsuleIdentity::Nomos(_) => Ok(()),
        _ => Err(Error::WrongCapsuleKind),
    }
}
