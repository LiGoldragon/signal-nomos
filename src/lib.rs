//! Pure typed wire vocabulary owned by Nomos.
//!
//! This crate names only independently meaningful seating identities and
//! selectors. No request or reply algebra is exported while the current
//! authority-sealed bootstrap transformation has no process boundary.

mod error;

pub use error::Error;

use capsule_content_identity::CapsuleIdentity;
use rkyv::{Archive, Deserialize, Serialize};

/// A wire value whose private invariants remain checkable after restoration.
pub trait WireInvariant {
    /// Revalidate one value restored from untrusted bytes.
    fn validate(&self) -> Result<(), Error>;
}

/// Component-owned opaque Nomos slot identity.
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub struct NomosSlotId(u64);

// Too trivial — pure construction and projection of one opaque wire integer.
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

// Too trivial — pure construction and arithmetic over one generation integer.
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

/// Compare-and-set expectation for a slot binding.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotExpectation {
    Empty,
    Generation(SlotGeneration),
}

/// Ephemeral lowercase hexadecimal Capsule display within one resolver view.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ShortCapsuleDisplay(String);

// Too trivial — construction and immutable projection around one validated string.
impl ShortCapsuleDisplay {
    pub fn try_new(value: impl Into<String>) -> Result<Self, Error> {
        let display = Self(value.into());
        display.validate()?;
        Ok(display)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl WireInvariant for ShortCapsuleDisplay {
    fn validate(&self) -> Result<(), Error> {
        if (4..=64).contains(&self.0.len())
            && self
                .0
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(())
        } else {
            Err(Error::InvalidShortCapsuleDisplay)
        }
    }
}

/// Full persistent Nomos identity or an ephemeral resolver-relative display.
#[derive(Archive, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum CapsuleSelector {
    Full(CapsuleIdentity),
    Short(ShortCapsuleDisplay),
}

impl WireInvariant for CapsuleSelector {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Full(CapsuleIdentity::Nomos(_)) => Ok(()),
            Self::Full(_) => Err(Error::WrongCapsuleKind),
            Self::Short(display) => display.validate(),
        }
    }
}
