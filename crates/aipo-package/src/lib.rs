//! Package identity, manifest, lockfile, and deterministic resolution for Aipo.
//!
//! This crate is the package protocol foundation. It owns the canonical coordinate,
//! manifest and lockfile models, SHA-256 content digests, local path discovery, pinned
//! GitHub source resolution, and the dependency graph used by later compiler and CLI
//! integration. The default feature set performs no network access and exposes the verified local
//! cache API plus read-only cache verification. The optional `http` feature adds an explicit public
//! GitHub fetcher; package scripts are never executed.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod capabilities;
pub mod coordinate;
pub mod digest;
pub mod error;
pub mod github;
pub mod github_http;
pub mod lock;
pub mod manifest;
pub mod resolver;

pub use capabilities::{
    first_outside_bound, normalize_capabilities, parse_capabilities, union_capabilities,
};
pub use coordinate::{PackageCoordinate, PackageId, PackageIdError};
pub use digest::{ContentDigest, DigestError};
pub use error::{LockfileError, ManifestError, ResolveError, ValidationProblem};
pub use github::{
    GitHubArtifact, GitHubFetchError, GitHubFetcher, GitHubGraphError, GitHubPackageGraph,
    InMemoryGitHubStore, package_input_from_artifact, resolve_github_package_graph,
};
pub use github_http::{
    CacheOnlyGitHubFetcher, CacheVerification, CachedGitHubFetcher, CachedGraphError,
    CachedPackageGraph, GitHubCache, GitHubCacheError, resolve_cached_github_graph,
    resolve_mixed_package_graph,
};
#[cfg(feature = "http")]
pub use github_http::{
    GitHubHttpConfig, GitHubHttpFetcher, GitHubHttpResponse, GitHubHttpTransport,
    GitHubHttpTransportError, UreqGitHubTransport,
};
pub use lock::{
    GitHubSource, Lock, LockPackage, LockSource, LockedPackage, Lockfile, PackageSource,
};
pub use manifest::{Dependency, Manifest};
pub use resolver::{
    DiscoveredLocalPackages, LocalPackageResolver, PackageInput, PackagePathMap,
    ResolvedLocalPackages, ResolvedPackage, ResolvedPackageGraph, resolve_manifests,
    resolve_package_inputs, resolve_path, resolve_path_with_paths,
};

/// Canonical manifest filename.
pub const MANIFEST_FILE_NAME: &str = "aipo.toml";

/// Canonical lockfile filename.
pub const LOCK_FILE_NAME: &str = "aipo.lock";
