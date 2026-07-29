//! Block identity: name, version, content-addressed [`BlockId`].

use std::fmt;
use std::str::FromStr;

use blake3::Hasher;
use semver::{Version as SemVersion, VersionReq as SemVersionReq};
use smol_str::SmolStr;

use crate::genesis::hash::{finalize_u128, update_str};

/// Stable block name (`"structural"`, `"sum"`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockName(pub SmolStr);

impl BlockName {
    /// Construct from any stringy value.
    #[must_use]
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(SmolStr::new(name.as_ref()))
    }

    /// Borrow the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for BlockName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for BlockName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for BlockName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Exact semver for a block pin / identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version(pub SemVersion);

impl Version {
    /// Parse a semver string.
    ///
    /// # Errors
    ///
    /// Returns the parse error when `s` is not valid semver.
    pub fn parse(s: &str) -> Result<Self, semver::Error> {
        Ok(Self(SemVersion::parse(s)?))
    }

    /// Borrow the inner semver.
    #[must_use]
    pub fn as_semver(&self) -> &SemVersion {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Version {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Semver range carried by [`BlockReq`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionRange(pub SemVersionReq);

impl VersionRange {
    /// Parse a semver requirement (`^1.0`, `=1.2.0`, …).
    ///
    /// # Errors
    ///
    /// Returns the parse error when `s` is not a valid requirement.
    pub fn parse(s: &str) -> Result<Self, semver::Error> {
        Ok(Self(SemVersionReq::parse(s)?))
    }

    /// Whether `version` satisfies this range.
    #[must_use]
    pub fn matches(&self, version: &Version) -> bool {
        self.0.matches(version.as_semver())
    }

    /// Borrow the requirement string form.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for VersionRange {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Dependency on another block by name + version range.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockReq {
    /// Required block name.
    pub name: BlockName,
    /// Acceptable version range.
    pub range: VersionRange,
}

impl BlockReq {
    /// Construct a requirement.
    #[must_use]
    pub fn new(name: impl Into<BlockName>, range: VersionRange) -> Self {
        Self {
            name: name.into(),
            range,
        }
    }
}

/// Content-addressed block identity: `BLAKE3₁₂₈(canonical block bytes)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u128);

impl BlockId {
    /// Hash already-canonical block bytes.
    #[must_use]
    pub fn from_canonical(bytes: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"block");
        update_str(&mut hasher, "v1");
        hasher.update(bytes);
        Self(finalize_u128(&hasher))
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}
