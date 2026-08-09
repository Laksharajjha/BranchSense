//! Version and compatibility primitives for adapter contracts.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Version of the language-adapter framework API.
pub const ADAPTER_API_VERSION: Version = Version::new(1, 0, 0);

/// Semantic version used for adapters and framework compatibility.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Version {
    major: u16,
    minor: u16,
    patch: u16,
}

impl Version {
    /// Creates a semantic version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Returns the major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the patch component.
    #[must_use]
    pub const fn patch(self) -> u16 {
        self.patch
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Inclusive lower and optional inclusive upper compatibility bound.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VersionRange {
    minimum: Version,
    maximum: Option<Version>,
}

impl VersionRange {
    /// Creates a compatibility range.
    #[must_use]
    pub const fn new(minimum: Version, maximum: Option<Version>) -> Self {
        Self { minimum, maximum }
    }

    /// Creates a range containing one version only.
    #[must_use]
    pub const fn exact(version: Version) -> Self {
        Self::new(version, Some(version))
    }

    /// Creates an unbounded range starting at `minimum`.
    #[must_use]
    pub const fn from(minimum: Version) -> Self {
        Self::new(minimum, None)
    }

    /// Returns whether a version is in this range.
    #[must_use]
    pub fn contains(self, version: Version) -> bool {
        version >= self.minimum
            && match self.maximum {
                Some(maximum) => version <= maximum,
                None => true,
            }
    }

    /// Returns the inclusive lower bound.
    #[must_use]
    pub const fn minimum(self) -> Version {
        self.minimum
    }

    /// Returns the optional inclusive upper bound.
    #[must_use]
    pub const fn maximum(self) -> Option<Version> {
        self.maximum
    }
}
