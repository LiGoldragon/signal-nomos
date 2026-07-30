#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("archive encoding failed: {0}")]
    Encoding(#[from] rkyv::rancor::Error),
    #[error("Nomos artifact validation failed: {0}")]
    Nomos(#[from] core_nomos::NomosSealError),
    #[error("Capsule identity is not Nomos")]
    WrongCapsuleKind,
    #[error("projection is bound to a different Capsule identity")]
    ProjectionContentMismatch,
    #[error("projection predecessor version cannot advance without overflowing")]
    ProjectionVersionOverflow,
    #[error("projection version is not the exact successor of its expected predecessor")]
    ProjectionVersionMismatch,
    #[error("short Capsule display must be 4..64 lowercase hexadecimal digits")]
    InvalidShortCapsuleDisplay,
    #[error("Ethos population archive is empty")]
    EmptyEthosPopulationArchive,
    #[error("Logos population archive is empty")]
    EmptyLogosPopulationArchive,
    #[error("translator rename receipt archive is empty")]
    EmptyTranslatorRenameReceipt,
}
