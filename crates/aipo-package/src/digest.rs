//! Deterministic package content digests.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};

use crate::manifest::Manifest;

/// A lowercase hexadecimal SHA-256 content digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentDigest(String);

/// A failure while parsing or computing a content digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestError {
    /// The digest did not contain exactly 64 hexadecimal characters.
    InvalidLength {
        /// Number of characters supplied.
        length: usize,
    },
    /// The digest contained a non-hexadecimal character.
    InvalidHex,
    /// Canonical manifest encoding failed.
    Serialize(String),
}

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { length } => {
                write!(
                    formatter,
                    "SHA-256 digest must contain 64 characters, found {length}"
                )
            }
            Self::InvalidHex => write!(
                formatter,
                "SHA-256 digest contains a non-hexadecimal character"
            ),
            Self::Serialize(detail) => {
                write!(formatter, "canonical manifest encoding failed: {detail}")
            }
        }
    }
}

impl std::error::Error for DigestError {}

impl ContentDigest {
    /// Parses a lowercase hexadecimal SHA-256 digest.
    ///
    /// # Errors
    ///
    /// Returns [`DigestError::InvalidLength`] or [`DigestError::InvalidHex`] for a malformed
    /// digest.
    pub fn parse(value: &str) -> Result<Self, DigestError> {
        if value.len() != 64 {
            return Err(DigestError::InvalidLength {
                length: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|character| character.is_ascii_digit() || (b'a'..=b'f').contains(&character))
        {
            return Err(DigestError::InvalidHex);
        }
        Ok(Self(value.to_string()))
    }

    /// Computes a framed digest from named byte slices.
    #[must_use]
    pub fn from_parts(parts: &[(&str, &[u8])]) -> Self {
        let mut hasher = Sha256::new();
        for (name, bytes) in parts {
            update_part(&mut hasher, name.as_bytes());
            update_part(&mut hasher, bytes);
        }
        Self(hex(&hasher.finalize()))
    }

    /// Computes the canonical digest of a manifest and its discovered entry.
    ///
    /// The manifest is canonicalized before encoding, and the entry path is relative to the
    /// package directory. No absolute path or filesystem metadata enters the digest.
    ///
    /// # Errors
    ///
    /// Returns [`DigestError::Serialize`] when the canonical manifest cannot be encoded.
    pub fn for_package(
        manifest: &Manifest,
        entry_path: &str,
        entry_bytes: &[u8],
    ) -> Result<Self, DigestError> {
        let mut canonical_manifest = manifest.clone();
        canonical_manifest
            .canonicalize()
            .map_err(|error| DigestError::Serialize(error.to_string()))?;
        let manifest_bytes = serde_json::to_vec(&canonical_manifest)
            .map_err(|error| DigestError::Serialize(error.to_string()))?;
        Ok(Self::from_parts(&[
            ("manifest", manifest_bytes.as_slice()),
            ("entry-path", entry_path.as_bytes()),
            ("entry", entry_bytes),
        ]))
    }

    /// Returns the hexadecimal digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the digest algorithm name.
    #[must_use]
    pub const fn algorithm() -> &'static str {
        "sha256"
    }

    /// Consumes the digest and returns its hexadecimal text.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for ContentDigest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<String> for ContentDigest {
    type Error = DigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<ContentDigest> for String {
    fn from(value: ContentDigest) -> Self {
        value.0
    }
}

fn update_part(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dependency, PackageId};

    #[test]
    fn digest_is_stable_for_identical_content() {
        let manifest = Manifest::new(
            PackageId::parse("acme.root").expect("valid"),
            "1.0.0",
            "main",
        )
        .expect("valid manifest");
        let first =
            ContentDigest::for_package(&manifest, "main", b"same bytes").expect("digest computes");
        let second =
            ContentDigest::for_package(&manifest, "main", b"same bytes").expect("digest computes");
        assert_eq!(first, second);
        assert_eq!(first.as_str().len(), 64);
        assert_eq!(ContentDigest::algorithm(), "sha256");
    }

    #[test]
    fn digest_changes_with_manifest_or_entry_content() {
        let manifest = Manifest::new(
            PackageId::parse("acme.root").expect("valid"),
            "1.0.0",
            "main",
        )
        .expect("valid manifest");
        let mut changed_manifest = manifest.clone();
        changed_manifest.dependencies.push(Dependency::new(
            PackageId::parse("acme.dep").expect("valid"),
            "../dep",
        ));
        let baseline = ContentDigest::for_package(&manifest, "main", b"entry").expect("digest");
        let changed =
            ContentDigest::for_package(&changed_manifest, "main", b"entry").expect("digest");
        let changed_entry =
            ContentDigest::for_package(&manifest, "main", b"other").expect("digest");
        assert_ne!(baseline, changed);
        assert_ne!(baseline, changed_entry);
    }

    #[test]
    fn digest_framing_prevents_part_boundary_collisions() {
        let first = ContentDigest::from_parts(&[("ab", b"c"), ("d", b"")]);
        let second = ContentDigest::from_parts(&[("a", b"bc"), ("d", b"")]);
        assert_ne!(first, second);
    }

    #[test]
    fn digest_parser_rejects_non_canonical_text() {
        assert!(ContentDigest::parse("abc").is_err());
        assert!(ContentDigest::parse(&"A".repeat(64)).is_err());
        assert!(ContentDigest::parse(&"a".repeat(64)).is_ok());
    }
}
