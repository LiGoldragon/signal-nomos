/// Invariant violation in the independently owned Nomos wire vocabulary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Capsule identity is not Nomos")]
    WrongCapsuleKind,
    #[error("short Capsule display must be 4..64 lowercase hexadecimal digits")]
    InvalidShortCapsuleDisplay,
}
