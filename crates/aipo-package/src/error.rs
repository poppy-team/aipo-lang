//! Typed errors and validation problems for package operations.

use std::fmt;

use aipo_host::CapabilityError;

use crate::coordinate::{PackageId, PackageIdError};
use crate::digest::DigestError;

/// A deterministic path/message pair describing one invalid model value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationProblem {
    /// Location of the invalid value in the model.
    pub path: String,
    /// Human-readable reason.
    pub message: String,
}

impl ValidationProblem {
    /// Creates a validation problem.
    #[must_use]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

/// An error produced while loading, decoding, or validating a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// Manifest bytes were not valid UTF-8.
    InvalidUtf8 {
        /// Decoder detail.
        detail: String,
    },
    /// TOML or serde decoding failed.
    Parse {
        /// Decoder detail.
        detail: String,
    },
    /// The decoded model is structurally or semantically invalid.
    Invalid {
        /// All detected validation problems in deterministic order.
        problems: Vec<ValidationProblem>,
    },
    /// TOML or JSON encoding failed.
    Serialize {
        /// Encoder detail.
        detail: String,
    },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { detail } => {
                write!(formatter, "manifest is not valid UTF-8: {detail}")
            }
            Self::Parse { detail } => write!(formatter, "manifest could not be parsed: {detail}"),
            Self::Invalid { problems } => {
                let detail = problems
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ");
                if detail.is_empty() {
                    write!(formatter, "manifest is invalid")
                } else {
                    write!(formatter, "manifest is invalid: {detail}")
                }
            }
            Self::Serialize { detail } => {
                write!(formatter, "manifest could not be serialized: {detail}")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// An error produced while decoding, canonicalizing, or validating a lockfile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockfileError {
    /// Lockfile bytes were not valid UTF-8.
    InvalidUtf8 {
        /// Decoder detail.
        detail: String,
    },
    /// TOML or serde decoding failed.
    Parse {
        /// Decoder detail.
        detail: String,
    },
    /// The lockfile violates its model invariants.
    Invalid {
        /// All detected validation problems in deterministic order.
        problems: Vec<ValidationProblem>,
    },
    /// TOML or JSON encoding failed.
    Serialize {
        /// Encoder detail.
        detail: String,
    },
}

impl fmt::Display for LockfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { detail } => {
                write!(formatter, "lockfile is not valid UTF-8: {detail}")
            }
            Self::Parse { detail } => write!(formatter, "lockfile could not be parsed: {detail}"),
            Self::Invalid { problems } => {
                let detail = problems
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ");
                if detail.is_empty() {
                    write!(formatter, "lockfile is invalid")
                } else {
                    write!(formatter, "lockfile is invalid: {detail}")
                }
            }
            Self::Serialize { detail } => {
                write!(formatter, "lockfile could not be serialized: {detail}")
            }
        }
    }
}

impl std::error::Error for LockfileError {}

/// An error produced while discovering or resolving local packages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// A manifest could not be loaded or validated.
    Manifest {
        /// Relative or caller-provided source path.
        path: String,
        /// Manifest failure.
        error: ManifestError,
    },
    /// A local file could not be read.
    Io {
        /// Relative or caller-provided path.
        path: String,
        /// I/O detail.
        detail: String,
    },
    /// A dependency coordinate was not present in the package store.
    MissingDependency {
        /// Coordinate of the package that declared the edge.
        dependent: PackageId,
        /// Coordinate that could not be found.
        dependency: PackageId,
    },
    /// A local resolver encountered a remote dependency without a remote loader.
    RemoteDependencyUnsupported {
        /// Coordinate of the package that declared the edge.
        dependent: PackageId,
        /// Coordinate requested by the edge.
        dependency: PackageId,
        /// Pinned remote source label.
        source: String,
    },
    /// A dependency graph contains a cycle.
    DependencyCycle {
        /// Ordered cycle, with the first coordinate repeated at the end.
        cycle: Vec<PackageId>,
    },
    /// Two distinct package inputs claimed the same coordinate.
    DuplicateCoordinate {
        /// Conflicting coordinate.
        coordinate: PackageId,
        /// Source already associated with the coordinate.
        first_source: String,
        /// New source associated with the coordinate.
        second_source: String,
    },
    /// A dependency path resolved to a manifest with another coordinate.
    CoordinateMismatch {
        /// Coordinate requested by the dependency edge.
        expected: PackageId,
        /// Coordinate declared by the loaded manifest.
        actual: PackageId,
    },
    /// An exact dependency SemVer string did not match the package version string.
    VersionConflict {
        /// Coordinate with the conflicting version.
        coordinate: PackageId,
        /// Exact SemVer text required by the dependent edge.
        expected: String,
        /// Exact SemVer text declared by the loaded package.
        actual: String,
        /// Coordinate that declared the requirement.
        dependent: PackageId,
    },
    /// The package entry could not be discovered.
    EntryNotFound {
        /// Package whose entry is missing.
        coordinate: PackageId,
        /// Entry value from the manifest.
        entry: String,
        /// Relative paths tried by the local discovery helper.
        candidates: Vec<String>,
    },
    /// A capability declaration was not valid under the host grammar.
    Capability(CapabilityError),
    /// A transitive package capability exceeded the root declaration.
    CapabilityLimitExceeded {
        /// Root package coordinate.
        root: PackageId,
        /// Package that declared the excess capability.
        package: PackageId,
        /// Capability that was outside the root bound.
        capability: String,
    },
    /// A generated lockfile failed validation.
    Lockfile(LockfileError),
    /// A package content digest could not be computed.
    Digest(DigestError),
    /// A caller-provided package input is inconsistent.
    InvalidInput {
        /// Coordinate or source path associated with the input.
        context: String,
        /// Reason the input was rejected.
        detail: String,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest { path, error } => write!(formatter, "manifest '{path}': {error}"),
            Self::Io { path, detail } => write!(formatter, "package file '{path}': {detail}"),
            Self::MissingDependency {
                dependent,
                dependency,
            } => write!(
                formatter,
                "package '{dependent}' requires missing dependency '{dependency}'"
            ),
            Self::RemoteDependencyUnsupported {
                dependent,
                dependency,
                source,
            } => write!(
                formatter,
                "package '{dependent}' requires remote dependency '{dependency}' from '{source}', but no remote loader is active"
            ),
            Self::DependencyCycle { cycle } => write!(
                formatter,
                "package dependency cycle: {}",
                cycle
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            Self::DuplicateCoordinate {
                coordinate,
                first_source,
                second_source,
            } => write!(
                formatter,
                "package coordinate '{coordinate}' is provided by both '{first_source}' and '{second_source}'"
            ),
            Self::CoordinateMismatch { expected, actual } => write!(
                formatter,
                "dependency expected coordinate '{expected}', but its manifest declares '{actual}'"
            ),
            Self::VersionConflict {
                coordinate,
                expected,
                actual,
                dependent,
            } => write!(
                formatter,
                "package '{dependent}' requires '{coordinate}' version '{expected}', but '{actual}' was found"
            ),
            Self::EntryNotFound {
                coordinate,
                entry,
                candidates,
            } => write!(
                formatter,
                "package '{coordinate}' entry '{entry}' was not found (tried {})",
                candidates.join(", ")
            ),
            Self::Capability(error) => write!(formatter, "invalid package capability: {error}"),
            Self::CapabilityLimitExceeded {
                root,
                package,
                capability,
            } => write!(
                formatter,
                "package '{package}' capability '{capability}' exceeds root '{root}' upper bound"
            ),
            Self::Lockfile(error) => write!(formatter, "generated lockfile is invalid: {error}"),
            Self::Digest(error) => write!(formatter, "package digest failed: {error}"),
            Self::InvalidInput { context, detail } => {
                write!(formatter, "invalid package input '{context}': {detail}")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

impl From<CapabilityError> for ResolveError {
    fn from(error: CapabilityError) -> Self {
        Self::Capability(error)
    }
}

impl From<DigestError> for ResolveError {
    fn from(error: DigestError) -> Self {
        Self::Digest(error)
    }
}

impl From<LockfileError> for ResolveError {
    fn from(error: LockfileError) -> Self {
        Self::Lockfile(error)
    }
}

impl From<PackageIdError> for ResolveError {
    fn from(error: PackageIdError) -> Self {
        Self::InvalidInput {
            context: "package coordinate".to_string(),
            detail: error.to_string(),
        }
    }
}
