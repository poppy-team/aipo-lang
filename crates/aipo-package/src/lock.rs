//! Canonical `aipo.lock` models and validation.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::capabilities::normalize_capabilities;
use crate::coordinate::PackageId;
use crate::digest::ContentDigest;
use crate::error::{LockfileError, ValidationProblem};
use crate::manifest::is_valid_semver;

fn default_github_subpath() -> String {
    ".".to_string()
}

/// Canonical identity of a package fetched from a GitHub repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GitHubSource {
    /// GitHub repository slug in `owner/repository` form.
    pub repository: String,
    /// Exact lowercase 40-character commit SHA.
    pub revision: String,
    /// Relative package directory inside the repository.
    #[serde(default = "default_github_subpath")]
    pub subpath: String,
}

impl GitHubSource {
    /// Returns a stable source label without exposing a network URL.
    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "github:{}@{}/{}",
            self.repository, self.revision, self.subpath
        )
    }
}

/// The source of a package dependency or locked package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PackageSource {
    /// A package resolved from a relative local directory.
    Path {
        /// Relative path from the package declaring the dependency.
        path: String,
    },
    /// A package resolved from an exact GitHub commit.
    GitHub {
        /// GitHub repository slug in `owner/repository` form.
        repository: String,
        /// Exact lowercase 40-character commit SHA.
        revision: String,
        /// Relative package directory inside the repository.
        #[serde(default = "default_github_subpath")]
        subpath: String,
    },
}

/// Alias emphasizing that the source is a lockfile source.
pub type LockSource = PackageSource;

impl PackageSource {
    /// Creates a validated local path source.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] for an absolute or malformed path.
    pub fn path(path: impl Into<String>) -> Result<Self, LockfileError> {
        let source = Self::Path { path: path.into() };
        source.validate().map(|()| source)
    }

    /// Creates a validated GitHub source at the repository root.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] for a malformed repository, revision, or subpath.
    pub fn github(
        repository: impl Into<String>,
        revision: impl Into<String>,
    ) -> Result<Self, LockfileError> {
        Self::github_at(repository, revision, ".")
    }

    /// Creates a validated GitHub source at a repository subpath.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] for a malformed repository, revision, or subpath.
    pub fn github_at(
        repository: impl Into<String>,
        revision: impl Into<String>,
        subpath: impl Into<String>,
    ) -> Result<Self, LockfileError> {
        let source = Self::GitHub {
            repository: repository.into(),
            revision: revision.into(),
            subpath: subpath.into(),
        };
        source.validate().map(|()| source)
    }

    /// Returns the GitHub identity when this source is remote.
    #[must_use]
    pub fn github_source(&self) -> Option<GitHubSource> {
        match self {
            Self::Path { .. } => None,
            Self::GitHub {
                repository,
                revision,
                subpath,
            } => Some(GitHubSource {
                repository: repository.clone(),
                revision: revision.clone(),
                subpath: subpath.clone(),
            }),
        }
    }

    /// Returns a stable source label for diagnostics and duplicate-source errors.
    #[must_use]
    pub fn location(&self) -> String {
        match self {
            Self::Path { path } => path.clone(),
            Self::GitHub {
                repository,
                revision,
                subpath,
            } => format!("github:{repository}@{revision}/{subpath}"),
        }
    }

    /// Validates this source.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] when a local or GitHub source violates its contract.
    pub fn validate(&self) -> Result<(), LockfileError> {
        let mut problems = Vec::new();
        self.validate_at("source", &mut problems);
        finish_validation(problems)
    }

    fn validate_at(&self, prefix: &str, problems: &mut Vec<ValidationProblem>) {
        match self {
            Self::Path { path } => validate_path(path, &format!("{prefix}.path"), problems),
            Self::GitHub {
                repository,
                revision,
                subpath,
            } => {
                validate_github_repository(repository, &format!("{prefix}.repository"), problems);
                validate_github_revision(revision, &format!("{prefix}.revision"), problems);
                validate_github_subpath(subpath, &format!("{prefix}.subpath"), problems);
            }
        }
    }
}

/// One exact SemVer package resolution stored in `aipo.lock`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Canonical package coordinate.
    #[serde(alias = "name", alias = "package", alias = "id")]
    pub coordinate: PackageId,
    /// Exact resolved SemVer version.
    pub version: String,
    /// Resolved source.
    pub source: PackageSource,
    /// SHA-256 digest of canonical manifest and entry content.
    #[serde(alias = "content", alias = "content_digest", alias = "checksum")]
    pub digest: ContentDigest,
    /// Canonical coordinates of direct dependencies.
    #[serde(default)]
    pub dependencies: Vec<PackageId>,
    /// Normalized declared capabilities.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Normalized compilation targets declared by the package.
    #[serde(default)]
    pub targets: Vec<String>,
}

impl LockedPackage {
    /// Returns the canonical package coordinate.
    #[must_use]
    pub fn coordinate(&self) -> &PackageId {
        &self.coordinate
    }

    /// Creates a locked package record.
    #[must_use]
    pub fn new(
        coordinate: PackageId,
        version: impl Into<String>,
        source: PackageSource,
        digest: ContentDigest,
        dependencies: Vec<PackageId>,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            coordinate,
            version: version.into(),
            source,
            digest,
            dependencies,
            capabilities,
            targets: Vec::new(),
        }
    }

    /// Adds normalized compilation targets to this locked package.
    #[must_use]
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = targets;
        self
    }

    /// Validates this record in isolation, excluding graph membership checks.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] when a field violates the lockfile contract.
    pub fn validate(&self) -> Result<(), LockfileError> {
        let mut problems = Vec::new();
        self.validate_at("package", true, &mut problems);
        finish_validation(problems)
    }

    fn validate_at(
        &self,
        prefix: &str,
        require_order: bool,
        problems: &mut Vec<ValidationProblem>,
    ) {
        validate_version(&self.version, &format!("{prefix}.version"), problems);
        self.source
            .validate_at(&format!("{prefix}.source"), problems);

        let mut dependencies = BTreeSet::new();
        for (index, dependency) in self.dependencies.iter().enumerate() {
            let dependency_path = format!("{prefix}.dependencies[{index}]");
            if !dependencies.insert(dependency.clone()) {
                problems.push(ValidationProblem::new(
                    dependency_path.clone(),
                    "dependency is recorded more than once",
                ));
            }
            if dependency == &self.coordinate {
                problems.push(ValidationProblem::new(
                    dependency_path,
                    "package cannot depend on itself",
                ));
            }
        }
        if require_order && !is_sorted(&self.dependencies) {
            problems.push(ValidationProblem::new(
                format!("{prefix}.dependencies"),
                "dependencies are not in canonical order",
            ));
        }

        match normalize_capabilities(&self.capabilities) {
            Ok(normalized) if !require_order || normalized == self.capabilities => {}
            Ok(_) => problems.push(ValidationProblem::new(
                format!("{prefix}.capabilities"),
                "capabilities are not in normalized canonical order",
            )),
            Err(error) => problems.push(ValidationProblem::new(
                format!("{prefix}.capabilities"),
                error.to_string(),
            )),
        }

        let mut targets = self.targets.clone();
        targets.sort();
        targets.dedup();
        if require_order && targets != self.targets {
            problems.push(ValidationProblem::new(
                format!("{prefix}.targets"),
                "targets are not in normalized canonical order",
            ));
        }
        for (index, target) in self.targets.iter().enumerate() {
            validate_target(target, &format!("{prefix}.targets[{index}]"), problems);
        }
    }
}

/// A complete canonical `aipo.lock` document.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Lockfile {
    /// All resolved packages in coordinate order.
    pub packages: Vec<LockedPackage>,
}

/// Alias for callers that prefer the shorter lockfile name.
pub type Lock = Lockfile;

/// Alias for the record stored in a lockfile package list.
pub type LockPackage = LockedPackage;

impl Lockfile {
    /// Creates and canonicalizes a lockfile.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] when the package set is not a valid graph.
    pub fn new(mut packages: Vec<LockedPackage>) -> Result<Self, LockfileError> {
        packages.sort_by(|left, right| left.coordinate.cmp(&right.coordinate));
        let mut lockfile = Self { packages };
        lockfile.canonicalize()?;
        Ok(lockfile)
    }

    /// Parses a lockfile from UTF-8 TOML bytes.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::InvalidUtf8`], [`LockfileError::Parse`], or
    /// [`LockfileError::Invalid`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LockfileError> {
        let text = std::str::from_utf8(bytes).map_err(|error| LockfileError::InvalidUtf8 {
            detail: error.to_string(),
        })?;
        Self::parse_toml(text)
    }

    /// Parses and canonicalizes a lockfile from TOML text.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Parse`] or [`LockfileError::Invalid`].
    pub fn parse_toml(text: &str) -> Result<Self, LockfileError> {
        let mut lockfile: Self = toml::from_str(text).map_err(|error| LockfileError::Parse {
            detail: error.to_string(),
        })?;
        lockfile.canonicalize()?;
        Ok(lockfile)
    }

    /// Parses TOML text using the conventional method name.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Lockfile::parse_toml`].
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Result<Self, LockfileError> {
        Self::parse_toml(text)
    }

    /// Parses TOML text using the explicit lockfile format name.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Lockfile::parse_toml`].
    pub fn from_toml(text: &str) -> Result<Self, LockfileError> {
        Self::parse_toml(text)
    }

    /// Canonicalizes package, dependency, and capability ordering in place.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] for duplicate coordinates, invalid fields, missing
    /// edges, or dependency cycles.
    pub fn canonicalize(&mut self) -> Result<(), LockfileError> {
        self.validate_with_order(false)?;
        self.packages
            .sort_by(|left, right| left.coordinate.cmp(&right.coordinate));
        for package in &mut self.packages {
            package.dependencies.sort();
            package.targets.sort();
            package.targets.dedup();
            package.capabilities =
                normalize_capabilities(&package.capabilities).map_err(|error| {
                    LockfileError::Invalid {
                        problems: vec![ValidationProblem::new(
                            format!("package.{}.capabilities", package.coordinate),
                            error.to_string(),
                        )],
                    }
                })?;
        }
        self.validate_with_order(true)
    }

    /// Validates the complete lockfile, including canonical ordering and acyclicity.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Invalid`] with all detected problems.
    pub fn validate(&self) -> Result<(), LockfileError> {
        self.validate_with_order(true)
    }

    /// Returns a package by coordinate.
    #[must_use]
    pub fn package(&self, coordinate: &PackageId) -> Option<&LockedPackage> {
        self.packages
            .iter()
            .find(|package| &package.coordinate == coordinate)
    }

    /// Iterates over packages in their stored canonical order.
    pub fn iter(&self) -> impl Iterator<Item = &LockedPackage> {
        self.packages.iter()
    }

    /// Returns the number of locked packages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Reports whether the lockfile has no packages.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Serializes a canonical copy as TOML.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Serialize`] when TOML encoding fails.
    pub fn to_toml(&self) -> Result<String, LockfileError> {
        let mut canonical = self.clone();
        canonical.canonicalize()?;
        toml::to_string_pretty(&canonical).map_err(|error| LockfileError::Serialize {
            detail: error.to_string(),
        })
    }

    /// Serializes a canonical copy as JSON.
    ///
    /// # Errors
    ///
    /// Returns [`LockfileError::Serialize`] when JSON encoding fails.
    pub fn to_json(&self) -> Result<String, LockfileError> {
        let mut canonical = self.clone();
        canonical.canonicalize()?;
        serde_json::to_string_pretty(&canonical).map_err(|error| LockfileError::Serialize {
            detail: error.to_string(),
        })
    }

    fn validate_with_order(&self, require_order: bool) -> Result<(), LockfileError> {
        let mut problems = Vec::new();
        let mut coordinates = BTreeSet::new();
        for (index, package) in self.packages.iter().enumerate() {
            if !coordinates.insert(package.coordinate.clone()) {
                problems.push(ValidationProblem::new(
                    format!("packages[{index}].coordinate"),
                    "package coordinate is locked more than once",
                ));
            }
            package.validate_at(&format!("packages[{index}]"), require_order, &mut problems);
        }

        if require_order && !is_sorted_by_coordinates(&self.packages) {
            problems.push(ValidationProblem::new(
                "packages",
                "packages are not in canonical coordinate order",
            ));
        }

        for package in &self.packages {
            for dependency in &package.dependencies {
                if !coordinates.contains(dependency) {
                    problems.push(ValidationProblem::new(
                        format!("packages.{}.dependencies", package.coordinate),
                        format!("dependency '{dependency}' is not locked"),
                    ));
                }
            }
        }

        if let Some(cycle) = find_cycle(self) {
            let cycle_text = cycle
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" -> ");
            problems.push(ValidationProblem::new(
                "packages",
                format!("dependency cycle: {cycle_text}"),
            ));
        }

        finish_validation(problems)
    }
}

impl FromStr for Lockfile {
    type Err = LockfileError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Lockfile::parse_toml(value)
    }
}

fn validate_version(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    if value.is_empty() {
        problems.push(ValidationProblem::new(path, "version is empty"));
    } else if !is_valid_semver(value) {
        problems.push(ValidationProblem::new(
            path,
            "version must be an exact SemVer version: major.minor.patch with optional prerelease and build metadata",
        ));
    }
}

fn validate_target(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    if value.is_empty() {
        problems.push(ValidationProblem::new(path, "target is empty"));
    } else if value.trim() != value
        || value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        problems.push(ValidationProblem::new(
            path,
            "target must not contain surrounding whitespace or control characters",
        ));
    }
}

fn validate_github_repository(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    let mut segments = value.split('/');
    let owner = segments.next();
    let repository = segments.next();
    if owner.is_none() || repository.is_none() || segments.next().is_some() {
        problems.push(ValidationProblem::new(
            path,
            "GitHub repository must use owner/repository form",
        ));
        return;
    }
    for (label, segment) in [
        ("owner", owner.unwrap()),
        ("repository", repository.unwrap()),
    ] {
        let valid = !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_alphanumeric())
            && !segment
                .chars()
                .any(|character| character.is_ascii_uppercase())
            && segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character));
        if !valid {
            problems.push(ValidationProblem::new(
                format!("{path}.{label}"),
                "GitHub repository segment contains an invalid character",
            ));
        }
    }
}

fn validate_github_revision(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        problems.push(ValidationProblem::new(
            path,
            "GitHub revision must be an exact lowercase 40-hex commit SHA",
        ));
    }
}

fn validate_github_subpath(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    validate_path(value, path, problems);
    if value.split('/').any(|segment| segment == "..") {
        problems.push(ValidationProblem::new(
            path,
            "GitHub subpath cannot contain parent traversal",
        ));
    }
    if value.chars().any(|character| {
        character.is_control()
            || character.is_whitespace()
            || !(character.is_ascii_alphanumeric() || "-_./".contains(character))
    }) {
        problems.push(ValidationProblem::new(
            path,
            "GitHub subpath contains an unsupported character",
        ));
    }
}

fn validate_path(value: &str, path: &str, problems: &mut Vec<ValidationProblem>) {
    if value.is_empty() {
        problems.push(ValidationProblem::new(path, "path is empty"));
        return;
    }
    if value.contains('\\') {
        problems.push(ValidationProblem::new(
            path,
            "backslash separators are not portable",
        ));
    }
    if value.starts_with('/') || (value.as_bytes().get(1) == Some(&b':')) {
        problems.push(ValidationProblem::new(
            path,
            "absolute paths are not allowed",
        ));
    }
    if value.split('/').any(|segment| segment.is_empty()) {
        problems.push(ValidationProblem::new(
            path,
            "path contains an empty segment",
        ));
    }
}

fn is_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|window| window[0] <= window[1])
}

fn is_sorted_by_coordinates(packages: &[LockedPackage]) -> bool {
    packages
        .windows(2)
        .all(|window| window[0].coordinate <= window[1].coordinate)
}

fn find_cycle(lockfile: &Lockfile) -> Option<Vec<PackageId>> {
    let packages: BTreeMap<PackageId, &LockedPackage> = lockfile
        .packages
        .iter()
        .map(|package| (package.coordinate.clone(), package))
        .collect();
    let mut states = BTreeMap::<PackageId, u8>::new();
    let mut stack = Vec::new();

    for coordinate in packages.keys() {
        if !states.contains_key(coordinate)
            && find_cycle_from(coordinate, &packages, &mut states, &mut stack)
        {
            return Some(stack);
        }
    }
    None
}

fn find_cycle_from(
    coordinate: &PackageId,
    packages: &BTreeMap<PackageId, &LockedPackage>,
    states: &mut BTreeMap<PackageId, u8>,
    stack: &mut Vec<PackageId>,
) -> bool {
    match states.get(coordinate).copied() {
        Some(1) => {
            if let Some(start) = stack.iter().position(|item| item == coordinate) {
                let mut cycle = stack[start..].to_vec();
                cycle.push(coordinate.clone());
                *stack = cycle;
            }
            return true;
        }
        Some(2) => return false,
        _ => {}
    }

    states.insert(coordinate.clone(), 1);
    stack.push(coordinate.clone());
    if let Some(package) = packages.get(coordinate) {
        for dependency in &package.dependencies {
            if packages.contains_key(dependency)
                && find_cycle_from(dependency, packages, states, stack)
            {
                return true;
            }
        }
    }
    stack.pop();
    states.insert(coordinate.clone(), 2);
    false
}

fn finish_validation(problems: Vec<ValidationProblem>) -> Result<(), LockfileError> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(LockfileError::Invalid { problems })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_canonicalizes_package_edges_and_capabilities() {
        let dependency = LockedPackage::new(
            coordinate("acme.dep"),
            "1.0.0",
            PackageSource::Path {
                path: "../dep".to_string(),
            },
            digest(),
            Vec::new(),
            Vec::new(),
        );
        let root = LockedPackage::new(
            coordinate("acme.root"),
            "1.0.0",
            PackageSource::Path {
                path: ".".to_string(),
            },
            digest(),
            vec![coordinate("acme.dep")],
            vec!["clock.wall".to_string(), "clock".to_string()],
        );
        let lockfile = Lockfile::new(vec![root, dependency]).expect("lockfile is valid");
        assert_eq!(
            lockfile
                .iter()
                .map(|package| package.coordinate.as_str())
                .collect::<Vec<_>>(),
            vec!["acme.dep", "acme.root"]
        );
        assert_eq!(lockfile.packages[1].capabilities, vec!["clock"]);
        assert!(lockfile.validate().is_ok());
    }

    #[test]
    fn github_source_round_trips_and_keeps_provenance() {
        let source = PackageSource::github_at(
            "acme/packages",
            "0123456789abcdef0123456789abcdef01234567",
            "packages/http",
        )
        .expect("valid GitHub source");
        let package = LockedPackage::new(
            coordinate("acme.http"),
            "1.0.0",
            source.clone(),
            digest(),
            Vec::new(),
            Vec::new(),
        );
        let lockfile = Lockfile::new(vec![package]).expect("lockfile is valid");
        let toml = lockfile.to_toml().expect("lockfile serializes");
        assert!(toml.contains("type = \"github\""));
        assert!(toml.contains("0123456789abcdef0123456789abcdef01234567"));
        assert_eq!(
            Lockfile::from_str(&toml).expect("lockfile parses"),
            lockfile
        );
        let json = lockfile.to_json().expect("lockfile serializes as JSON");
        let json_lockfile: Lockfile = serde_json::from_str(&json).expect("JSON lockfile parses");
        assert_eq!(json_lockfile, lockfile);
        assert_eq!(
            source.location(),
            "github:acme/packages@0123456789abcdef0123456789abcdef01234567/packages/http"
        );
    }

    #[test]
    fn github_source_rejects_mutable_or_malformed_revisions() {
        for revision in [
            "main",
            "v1.0.0",
            "0123456789ABCDEF0123456789abcdef01234567",
            "0123456789abcdef0123456789abcdef0123456",
        ] {
            assert!(
                PackageSource::github("acme/packages", revision).is_err(),
                "{revision}"
            );
        }
        assert!(
            PackageSource::github_at(
                "https://github.com/acme/packages",
                "0123456789abcdef0123456789abcdef01234567",
                "."
            )
            .is_err()
        );
        assert!(
            PackageSource::github_at(
                "Acme/packages",
                "0123456789abcdef0123456789abcdef01234567",
                "."
            )
            .is_err()
        );
        assert!(
            PackageSource::github_at(
                "acme/packages",
                "0123456789abcdef0123456789abcdef01234567",
                "../escape"
            )
            .is_err()
        );
    }

    #[test]
    fn lockfile_round_trip_is_deterministic() {
        let lockfile = Lockfile::new(vec![LockedPackage::new(
            coordinate("acme.root"),
            "1.0.0",
            PackageSource::Path {
                path: ".".to_string(),
            },
            digest(),
            Vec::new(),
            vec!["filesystem.read".to_string()],
        )])
        .expect("lockfile is valid");
        let first = lockfile.to_toml().expect("lockfile serializes");
        let second = lockfile.to_toml().expect("lockfile serializes");
        assert_eq!(first, second);
        let decoded = Lockfile::from_str(&first).expect("lockfile parses");
        assert_eq!(decoded, lockfile);
        assert!(!first.contains("/home/"));
    }

    #[test]
    fn lockfile_rejects_non_semver_versions() {
        for version in ["abc", "1.0", "1.0.0.0", " 1.0.0", "^1.0.0", "~1.0.0"] {
            let package = LockedPackage::new(
                coordinate("acme.root"),
                version,
                PackageSource::Path {
                    path: ".".to_string(),
                },
                digest(),
                Vec::new(),
                Vec::new(),
            );
            assert!(
                package.validate().is_err(),
                "version should be invalid: {version}"
            );
        }
    }

    #[test]
    fn lockfile_rejects_missing_edges_absolute_sources_and_cycles() {
        let missing = LockedPackage::new(
            coordinate("acme.root"),
            "1.0.0",
            PackageSource::Path {
                path: ".".to_string(),
            },
            digest(),
            vec![coordinate("acme.missing")],
            Vec::new(),
        );
        assert!(Lockfile::new(vec![missing]).is_err());

        let absolute = LockedPackage::new(
            coordinate("acme.root"),
            "1.0.0",
            PackageSource::Path {
                path: "/tmp/root".to_string(),
            },
            digest(),
            Vec::new(),
            Vec::new(),
        );
        assert!(Lockfile::new(vec![absolute]).is_err());

        let first = LockedPackage::new(
            coordinate("acme.first"),
            "1.0.0",
            PackageSource::Path {
                path: ".".to_string(),
            },
            digest(),
            vec![coordinate("acme.second")],
            Vec::new(),
        );
        let second = LockedPackage::new(
            coordinate("acme.second"),
            "1.0.0",
            PackageSource::Path {
                path: ".".to_string(),
            },
            digest(),
            vec![coordinate("acme.first")],
            Vec::new(),
        );
        assert!(Lockfile::new(vec![first, second]).is_err());
    }

    fn coordinate(value: &str) -> PackageId {
        PackageId::parse(value).expect("test coordinate")
    }

    fn digest() -> ContentDigest {
        ContentDigest::from_parts(&[("test", b"content")])
    }
}
