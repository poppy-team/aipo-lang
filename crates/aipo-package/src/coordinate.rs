//! Canonical package identities.

use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A validated package coordinate in the canonical `namespace.package` form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageId(String);

/// Alias for the canonical package coordinate type.
pub type PackageCoordinate = PackageId;

/// The reason a package coordinate could not be constructed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageIdError {
    /// The coordinate was empty.
    Empty,
    /// The coordinate did not contain exactly one separator.
    InvalidSegmentCount {
        /// Number of dot-separated segments found.
        segments: usize,
    },
    /// A segment did not match `[a-z][a-z0-9_]*`.
    InvalidSegment {
        /// The rejected segment.
        segment: String,
    },
}

impl fmt::Display for PackageIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "package coordinate is empty"),
            Self::InvalidSegmentCount { segments } => write!(
                formatter,
                "package coordinate must have exactly two segments, found {segments}"
            ),
            Self::InvalidSegment { segment } => write!(
                formatter,
                "package coordinate segment '{segment}' must match [a-z][a-z0-9_]*"
            ),
        }
    }
}

impl std::error::Error for PackageIdError {}

impl PackageId {
    /// Parses and validates a canonical package coordinate.
    ///
    /// # Errors
    ///
    /// Returns a typed [`PackageIdError`] for an empty coordinate, a coordinate without
    /// exactly two segments, or a segment outside the lowercase ASCII package grammar.
    pub fn parse(value: &str) -> Result<Self, PackageIdError> {
        if value.is_empty() {
            return Err(PackageIdError::Empty);
        }

        let segments: Vec<&str> = value.split('.').collect();
        if segments.len() != 2 {
            return Err(PackageIdError::InvalidSegmentCount {
                segments: segments.len(),
            });
        }

        validate_segment(segments[0])?;
        validate_segment(segments[1])?;
        Ok(Self(value.to_string()))
    }

    /// Builds a coordinate from its two segments.
    ///
    /// # Errors
    ///
    /// Returns a typed [`PackageIdError`] when either segment is invalid.
    pub fn new(
        namespace: impl AsRef<str>,
        package: impl AsRef<str>,
    ) -> Result<Self, PackageIdError> {
        Self::parse(&format!("{}.{}", namespace.as_ref(), package.as_ref()))
    }

    /// Builds a coordinate from its two named segments.
    ///
    /// # Errors
    ///
    /// Returns a typed [`PackageIdError`] when either segment is invalid.
    pub fn from_segments(
        namespace: impl AsRef<str>,
        package: impl AsRef<str>,
    ) -> Result<Self, PackageIdError> {
        Self::new(namespace, package)
    }

    /// Returns the namespace segment.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.split('.').next().unwrap_or_default()
    }

    /// Returns the package segment.
    #[must_use]
    pub fn package(&self) -> &str {
        self.0.split_once('.').map_or("", |(_, package)| package)
    }

    /// Returns the canonical coordinate text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the coordinate and returns its canonical text.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PackageId {
    type Err = PackageIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for PackageId {
    type Error = PackageIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for PackageId {
    type Error = PackageIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<PackageId> for String {
    fn from(value: PackageId) -> Self {
        value.0
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for PackageId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

fn validate_segment(segment: &str) -> Result<(), PackageIdError> {
    let mut characters = segment.bytes();
    match characters.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => {
            return Err(PackageIdError::InvalidSegment {
                segment: segment.to_string(),
            });
        }
    }

    if characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'_'
    }) {
        Ok(())
    } else {
        Err(PackageIdError::InvalidSegment {
            segment: segment.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_round_trip_and_segments_are_borrowed() {
        let coordinate: PackageId = "acme.http_client2".parse().expect("valid coordinate");
        assert_eq!(coordinate.namespace(), "acme");
        assert_eq!(coordinate.package(), "http_client2");
        assert_eq!(coordinate.as_str(), "acme.http_client2");
        assert_eq!(coordinate.to_string(), "acme.http_client2");
        assert_eq!(
            PackageId::new("acme", "http").expect("valid"),
            coordinate_for("acme.http")
        );
        assert_eq!(
            serde_json::to_string(&coordinate).expect("serialized"),
            "\"acme.http_client2\""
        );
        assert_eq!(
            serde_json::from_str::<PackageId>("\"acme.http_client2\"").expect("deserialized"),
            coordinate
        );
    }

    #[test]
    fn coordinate_rejects_invalid_shapes_without_panicking() {
        assert_eq!(PackageId::parse(""), Err(PackageIdError::Empty));
        assert_eq!(
            PackageId::parse("acme"),
            Err(PackageIdError::InvalidSegmentCount { segments: 1 })
        );
        assert_eq!(
            PackageId::parse("acme.http.client"),
            Err(PackageIdError::InvalidSegmentCount { segments: 3 })
        );
        for value in [
            "Acme.http",
            "acme.Http",
            "acme.2http",
            "acme.-http",
            ".http",
            "acme.",
            "acme..http",
            "acme.http-client",
            "acme.http ",
            "acme.wáll",
        ] {
            assert!(PackageId::parse(value).is_err(), "{value}");
        }
    }

    #[test]
    fn invalid_deserialization_is_typed_and_total() {
        assert!(serde_json::from_str::<PackageId>("\"UPPER.case\"").is_err());
        assert!(serde_json::from_str::<PackageId>("\"a.b.c\"").is_err());
    }

    fn coordinate_for(value: &str) -> PackageId {
        PackageId::parse(value).expect("test coordinate")
    }
}
