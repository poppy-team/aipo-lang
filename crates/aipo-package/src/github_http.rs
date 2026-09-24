//! Opt-in, read-only GitHub HTTP transport and local artifact cache.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};

use crate::digest::ContentDigest;
use crate::github::{GitHubArtifact, GitHubFetchError, GitHubFetcher};
use crate::lock::{GitHubSource, PackageSource};
use crate::manifest::Manifest;

const RAW_GITHUB_HOST: &str = "raw.githubusercontent.com";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CACHE_FILE_BYTES: u64 = 4 * 1024 * 1024;
const CACHE_FORMAT_VERSION: u32 = 1;
static NEXT_CACHE_TEMP: AtomicU64 = AtomicU64::new(0);

/// HTTP limits and policy for public GitHub fetches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubHttpConfig {
    /// Maximum duration of one request.
    pub timeout: Duration,
    /// Maximum bytes accepted for one response.
    pub max_response_bytes: u64,
    /// User-Agent sent without credentials.
    pub user_agent: String,
}

impl Default for GitHubHttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            user_agent: format!("aipo-package/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// A bounded response returned by an HTTP transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubHttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Optional redirect location, retained only for policy diagnostics.
    pub location: Option<String>,
    /// Response body, already bounded by the transport.
    pub body: Vec<u8>,
}

/// Error returned by the low-level HTTP transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubHttpTransportError {
    /// The request could not be completed.
    Request(String),
    /// The request exceeded its timeout.
    Timeout,
    /// The response exceeded the configured byte limit.
    ResponseTooLarge {
        /// Maximum accepted bytes.
        limit: u64,
    },
}

impl std::fmt::Display for GitHubHttpTransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(detail) => write!(formatter, "HTTP request failed: {detail}"),
            Self::Timeout => write!(formatter, "HTTP request timed out"),
            Self::ResponseTooLarge { limit } => {
                write!(formatter, "HTTP response exceeded {limit} bytes")
            }
        }
    }
}

impl std::error::Error for GitHubHttpTransportError {}

/// Narrow transport seam used by the GitHub fetcher.
pub trait GitHubHttpTransport {
    /// Performs one public GET request.
    ///
    /// # Errors
    ///
    /// Returns a typed transport error for connection, timeout or response-size failures.
    fn get(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<GitHubHttpResponse, GitHubHttpTransportError>;
}

/// Rustls-backed ureq transport with redirects disabled and no credentials.
#[derive(Debug, Clone)]
pub struct UreqGitHubTransport {
    agent: ureq::Agent,
}

impl UreqGitHubTransport {
    /// Builds a transport from the public policy configuration.
    #[must_use]
    pub fn new(config: &GitHubHttpConfig) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(config.timeout))
            .https_only(true)
            .proxy(None)
            .http_status_as_error(false)
            .max_redirects(0)
            .max_redirects_will_error(false)
            .max_response_header_size(64 * 1024)
            .user_agent(config.user_agent.clone())
            .build();
        Self {
            agent: config.into(),
        }
    }
}

impl GitHubHttpTransport for UreqGitHubTransport {
    fn get(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<GitHubHttpResponse, GitHubHttpTransportError> {
        let mut response = self
            .agent
            .get(url)
            .header("Accept", "application/octet-stream")
            .call()
            .map_err(map_ureq_error)?;
        if let Some(length) = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
        {
            if length > max_bytes {
                return Err(GitHubHttpTransportError::ResponseTooLarge { limit: max_bytes });
            }
        }
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let body = response
            .body_mut()
            .with_config()
            .limit(max_bytes)
            .read_to_vec()
            .map_err(map_ureq_error)?;
        Ok(GitHubHttpResponse {
            status,
            location,
            body,
        })
    }
}

fn map_ureq_error(error: ureq::Error) -> GitHubHttpTransportError {
    match error {
        ureq::Error::Timeout(_) => GitHubHttpTransportError::Timeout,
        ureq::Error::BodyExceedsLimit(limit) => {
            GitHubHttpTransportError::ResponseTooLarge { limit }
        }
        other => GitHubHttpTransportError::Request(other.to_string()),
    }
}

/// Read-only GitHub fetcher that resolves raw files at an exact commit.
#[derive(Debug, Clone)]
pub struct GitHubHttpFetcher<T = UreqGitHubTransport> {
    transport: T,
    max_response_bytes: u64,
}

impl GitHubHttpFetcher<UreqGitHubTransport> {
    /// Builds the default public GitHub transport.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(&GitHubHttpConfig::default())
    }

    /// Builds the default transport with explicit limits.
    #[must_use]
    pub fn with_config(config: &GitHubHttpConfig) -> Self {
        Self {
            transport: UreqGitHubTransport::new(config),
            max_response_bytes: config.max_response_bytes,
        }
    }
}

impl Default for GitHubHttpFetcher<UreqGitHubTransport> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GitHubHttpFetcher<T>
where
    T: GitHubHttpTransport,
{
    /// Builds a fetcher around a caller-provided transport.
    #[must_use]
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        }
    }

    /// Overrides the response byte limit.
    #[must_use]
    pub fn with_max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    /// Returns the configured response limit.
    #[must_use]
    pub fn max_response_bytes(&self) -> u64 {
        self.max_response_bytes
    }

    fn fetch_path(&self, source: &GitHubSource, path: &str) -> Result<Vec<u8>, GitHubFetchError> {
        let url = raw_url(source, path).map_err(|detail| GitHubFetchError::InvalidResponse {
            source: source.label(),
            detail,
        })?;
        let response = self
            .transport
            .get(&url, self.max_response_bytes)
            .map_err(|error| map_transport_error(source, error))?;
        if response.body.len() as u64 > self.max_response_bytes {
            return Err(GitHubFetchError::ResponseTooLarge {
                source: source.label(),
                limit: self.max_response_bytes,
            });
        }
        match response.status {
            200..=299 => Ok(response.body),
            300..=399 => Err(GitHubFetchError::Redirect {
                source: source.label(),
                status: response.status,
            }),
            404 => Err(GitHubFetchError::NotFound {
                source: source.label(),
            }),
            status => Err(GitHubFetchError::HttpStatus {
                source: source.label(),
                status,
            }),
        }
    }
}

impl<T> GitHubFetcher for GitHubHttpFetcher<T>
where
    T: GitHubHttpTransport,
{
    fn fetch(&self, source: &PackageSource) -> Result<GitHubArtifact, GitHubFetchError> {
        let source = source
            .github_source()
            .ok_or_else(|| GitHubFetchError::UnsupportedSource {
                source: source.location(),
            })?;
        source_validation_error(&source)?;
        let manifest_bytes = self.fetch_path(&source, "aipo.toml")?;
        let manifest = Manifest::from_bytes(&manifest_bytes).map_err(|error| {
            GitHubFetchError::InvalidResponse {
                source: source.label(),
                detail: error.to_string(),
            }
        })?;
        join_remote_path(&source.subpath, &manifest.entry).map_err(|detail| {
            GitHubFetchError::InvalidResponse {
                source: source.label(),
                detail,
            }
        })?;
        let entry_bytes = self.fetch_path(&source, &manifest.entry)?;
        Ok(GitHubArtifact::new(manifest_bytes, entry_bytes))
    }
}

fn map_transport_error(source: &GitHubSource, error: GitHubHttpTransportError) -> GitHubFetchError {
    match error {
        GitHubHttpTransportError::Request(detail) => GitHubFetchError::Transport {
            source: source.label(),
            detail,
        },
        GitHubHttpTransportError::Timeout => GitHubFetchError::Transport {
            source: source.label(),
            detail: "request timed out".to_string(),
        },
        GitHubHttpTransportError::ResponseTooLarge { limit } => {
            GitHubFetchError::ResponseTooLarge {
                source: source.label(),
                limit,
            }
        }
    }
}

fn source_validation_error(source: &GitHubSource) -> Result<(), GitHubFetchError> {
    PackageSource::GitHub {
        repository: source.repository.clone(),
        revision: source.revision.clone(),
        subpath: source.subpath.clone(),
    }
    .validate()
    .map_err(|error| GitHubFetchError::InvalidSource {
        source: source.label(),
        detail: error.to_string(),
    })
}

fn raw_url(source: &GitHubSource, path: &str) -> Result<String, String> {
    let (owner, repository) = source
        .repository
        .split_once('/')
        .ok_or_else(|| "GitHub repository is not owner/repository".to_string())?;
    let path = join_remote_path(&source.subpath, path)?;
    Ok(format!(
        "https://{RAW_GITHUB_HOST}/{owner}/{repository}/{}/{}",
        source.revision, path
    ))
}

fn join_remote_path(base: &str, entry: &str) -> Result<String, String> {
    let combined = if base == "." {
        entry.to_string()
    } else {
        format!("{base}/{entry}")
    };
    if combined.is_empty()
        || combined.starts_with('/')
        || combined.contains('\\')
        || combined.chars().any(char::is_control)
    {
        return Err("remote path is empty or contains an unsafe character".to_string());
    }
    let mut normalized = Vec::new();
    for segment in combined.split('/') {
        match segment {
            "" | "." => continue,
            ".." => return Err("remote path contains parent traversal".to_string()),
            value
                if value.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "-_.".contains(character)
                }) =>
            {
                normalized.push(value);
            }
            _ => return Err("remote path contains an unsupported character".to_string()),
        }
    }
    if normalized.is_empty() {
        Err("remote path is empty".to_string())
    } else {
        Ok(normalized.join("/"))
    }
}

/// Error produced by the local GitHub artifact cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubCacheError {
    /// The cache root or an entry path is not a safe directory.
    InvalidRoot {
        /// Cache path.
        path: String,
        /// Reason.
        detail: String,
    },
    /// Cache I/O failed.
    Io {
        /// Cache path.
        path: String,
        /// I/O detail.
        detail: String,
    },
    /// Cached bytes are malformed or do not match their digest.
    Corrupt {
        /// Source label.
        source: String,
        /// Corruption detail.
        detail: String,
    },
    /// A source is not a GitHub source.
    UnsupportedSource {
        /// Source label.
        source: String,
    },
}

impl std::fmt::Display for GitHubCacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoot { path, detail } => {
                write!(formatter, "invalid GitHub cache root '{path}': {detail}")
            }
            Self::Io { path, detail } => write!(formatter, "GitHub cache I/O '{path}': {detail}"),
            Self::Corrupt { source, detail } => {
                write!(
                    formatter,
                    "GitHub cache entry '{source}' is corrupt: {detail}"
                )
            }
            Self::UnsupportedSource { source } => {
                write!(formatter, "source '{source}' is not a GitHub source")
            }
        }
    }
}

impl std::error::Error for GitHubCacheError {}

/// Local, atomic cache for fetched GitHub package artifacts.
#[derive(Debug, Clone)]
pub struct GitHubCache {
    root: PathBuf,
    max_file_bytes: u64,
}

impl GitHubCache {
    /// Creates a cache handle. The root is created only when an artifact is stored or loaded.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_file_bytes: MAX_CACHE_FILE_BYTES,
        }
    }

    /// Overrides the maximum size of each cached file.
    #[must_use]
    pub fn with_max_file_bytes(mut self, max_file_bytes: u64) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }

    /// Returns the configured cache root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Loads and verifies a cached artifact, if present.
    ///
    /// # Errors
    ///
    /// Returns [`GitHubCacheError`] for unsafe roots, malformed metadata, digest mismatches or I/O
    /// failures. A missing entry returns `Ok(None)`.
    pub fn load(&self, source: &PackageSource) -> Result<Option<GitHubArtifact>, GitHubCacheError> {
        let github = source
            .github_source()
            .ok_or_else(|| GitHubCacheError::UnsupportedSource {
                source: source.location(),
            })?;
        source
            .validate()
            .map_err(|error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            })?;
        let namespace = self.ensure_namespace()?;
        let entry_dir = namespace.join(cache_key(&github));
        if !entry_dir.exists() {
            return Ok(None);
        }
        reject_symlink(&entry_dir, &github.label())?;
        let metadata_bytes = read_bounded(
            &entry_dir.join("metadata.json"),
            self.max_file_bytes,
            &github,
        )?;
        let metadata: CacheMetadata =
            serde_json::from_slice(&metadata_bytes).map_err(|error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            })?;
        if metadata.version != CACHE_FORMAT_VERSION || metadata.source != github.label() {
            return Err(GitHubCacheError::Corrupt {
                source: github.label(),
                detail: "cache metadata does not match the requested source".to_string(),
            });
        }
        let manifest_bytes = read_bounded(
            &entry_dir.join("manifest.toml"),
            self.max_file_bytes,
            &github,
        )?;
        let entry_bytes = read_bounded(&entry_dir.join("entry.bin"), self.max_file_bytes, &github)?;
        let manifest =
            Manifest::from_bytes(&manifest_bytes).map_err(|error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            })?;
        let digest = ContentDigest::for_package(&manifest, &manifest.entry, &entry_bytes).map_err(
            |error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            },
        )?;
        if digest.as_str() != metadata.digest {
            return Err(GitHubCacheError::Corrupt {
                source: github.label(),
                detail: "cached content digest does not match metadata".to_string(),
            });
        }
        Ok(Some(GitHubArtifact::new(manifest_bytes, entry_bytes)))
    }

    /// Atomically stores and verifies an artifact.
    ///
    /// Existing entries are never silently overwritten. A valid existing entry is accepted; a
    /// corrupt existing entry fails closed.
    ///
    /// # Errors
    ///
    /// Returns [`GitHubCacheError`] for invalid content, unsafe roots, I/O failures or an existing
    /// corrupt entry.
    pub fn store(
        &self,
        source: &PackageSource,
        artifact: GitHubArtifact,
    ) -> Result<(), GitHubCacheError> {
        let github = source
            .github_source()
            .ok_or_else(|| GitHubCacheError::UnsupportedSource {
                source: source.location(),
            })?;
        source
            .validate()
            .map_err(|error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            })?;
        let manifest = Manifest::from_bytes(&artifact.manifest).map_err(|error| {
            GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            }
        })?;
        let digest = ContentDigest::for_package(&manifest, &manifest.entry, &artifact.entry)
            .map_err(|error| GitHubCacheError::Corrupt {
                source: github.label(),
                detail: error.to_string(),
            })?;
        let namespace = self.ensure_namespace()?;
        let final_dir = namespace.join(cache_key(&github));
        if final_dir.exists() {
            self.load(source)?;
            return Ok(());
        }
        let temp_dir = create_temp_dir(&namespace)?;
        let result = self.write_temp_entry(&temp_dir, &github, &artifact, digest.as_str());
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(error);
        }
        if let Err(error) = fs::rename(&temp_dir, &final_dir) {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(GitHubCacheError::Io {
                path: final_dir.display().to_string(),
                detail: error.to_string(),
            });
        }
        Ok(())
    }

    fn write_temp_entry(
        &self,
        temp_dir: &Path,
        source: &GitHubSource,
        artifact: &GitHubArtifact,
        digest: &str,
    ) -> Result<(), GitHubCacheError> {
        write_new_file(&temp_dir.join("manifest.toml"), &artifact.manifest, source)?;
        write_new_file(&temp_dir.join("entry.bin"), &artifact.entry, source)?;
        let metadata = CacheMetadata {
            version: CACHE_FORMAT_VERSION,
            source: source.label(),
            digest: digest.to_string(),
        };
        let metadata =
            serde_json::to_vec_pretty(&metadata).map_err(|error| GitHubCacheError::Corrupt {
                source: source.label(),
                detail: error.to_string(),
            })?;
        write_new_file(&temp_dir.join("metadata.json"), &metadata, source)?;
        Ok(())
    }

    fn ensure_namespace(&self) -> Result<PathBuf, GitHubCacheError> {
        ensure_directory(&self.root)?;
        let namespace = self.root.join("github-v1");
        ensure_directory(&namespace)?;
        Ok(namespace)
    }
}

/// A fetcher that serves verified cache entries before delegating to a transport fetcher.
#[derive(Debug, Clone)]
pub struct CachedGitHubFetcher<'a, F> {
    cache: &'a GitHubCache,
    fetcher: F,
}

impl<'a, F> CachedGitHubFetcher<'a, F> {
    /// Creates a cache-backed fetcher.
    #[must_use]
    pub fn new(cache: &'a GitHubCache, fetcher: F) -> Self {
        Self { cache, fetcher }
    }
}

impl<F> GitHubFetcher for CachedGitHubFetcher<'_, F>
where
    F: GitHubFetcher,
{
    fn fetch(&self, source: &PackageSource) -> Result<GitHubArtifact, GitHubFetchError> {
        match self.cache.load(source) {
            Ok(Some(artifact)) => Ok(artifact),
            Ok(None) => {
                let artifact = self.fetcher.fetch(source)?;
                self.cache
                    .store(source, artifact.clone())
                    .map_err(|error| GitHubFetchError::Cache {
                        source: source.location(),
                        detail: error.to_string(),
                    })?;
                Ok(artifact)
            }
            Err(error) => Err(GitHubFetchError::Cache {
                source: source.location(),
                detail: error.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CacheMetadata {
    version: u32,
    source: String,
    digest: String,
}

fn cache_key(source: &GitHubSource) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.label().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn ensure_directory(path: &Path) -> Result<(), GitHubCacheError> {
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(GitHubCacheError::InvalidRoot {
            path: path.display().to_string(),
            detail: "parent traversal is not allowed in cache paths".to_string(),
        });
    }
    reject_symlink_components(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(GitHubCacheError::InvalidRoot {
                    path: path.display().to_string(),
                    detail: "symbolic links are not allowed in the cache root".to_string(),
                });
            }
            if !metadata.is_dir() {
                return Err(GitHubCacheError::InvalidRoot {
                    path: path.display().to_string(),
                    detail: "path is not a directory".to_string(),
                });
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path).map_err(|error| GitHubCacheError::Io {
                path: path.display().to_string(),
                detail: error.to_string(),
            })?;
            ensure_directory(path)
        }
        Err(error) => Err(GitHubCacheError::Io {
            path: path.display().to_string(),
            detail: error.to_string(),
        }),
    }
}

fn reject_symlink_components(path: &Path) -> Result<(), GitHubCacheError> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(ancestor) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            return Err(GitHubCacheError::InvalidRoot {
                path: ancestor.display().to_string(),
                detail: "symbolic links are not allowed in cache paths".to_string(),
            });
        }
    }
    Ok(())
}

fn reject_symlink(path: &Path, source_label: &str) -> Result<(), GitHubCacheError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| GitHubCacheError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(GitHubCacheError::InvalidRoot {
            path: path.display().to_string(),
            detail: format!("symbolic links are not allowed for {source_label}"),
        });
    }
    Ok(())
}

fn create_temp_dir(namespace: &Path) -> Result<PathBuf, GitHubCacheError> {
    for _ in 0..16 {
        let sequence = NEXT_CACHE_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = namespace.join(format!(".tmp-{}-{}", std::process::id(), sequence));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(GitHubCacheError::Io {
                    path: path.display().to_string(),
                    detail: error.to_string(),
                });
            }
        }
    }
    Err(GitHubCacheError::Io {
        path: namespace.display().to_string(),
        detail: "could not allocate a unique temporary cache directory".to_string(),
    })
}

fn write_new_file(
    path: &Path,
    bytes: &[u8],
    source: &GitHubSource,
) -> Result<(), GitHubCacheError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| GitHubCacheError::Io {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
    file.write_all(bytes)
        .map_err(|error| GitHubCacheError::Io {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
    file.sync_all().map_err(|error| GitHubCacheError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    let _ = source;
    Ok(())
}

fn read_bounded(
    path: &Path,
    max_bytes: u64,
    source: &GitHubSource,
) -> Result<Vec<u8>, GitHubCacheError> {
    let metadata = fs::metadata(path).map_err(|error| GitHubCacheError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    if metadata.len() > max_bytes {
        return Err(GitHubCacheError::Corrupt {
            source: source.label(),
            detail: format!("cached file exceeds {max_bytes} bytes"),
        });
    }
    let mut file = File::open(path).map_err(|error| GitHubCacheError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    })?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| GitHubCacheError::Io {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
    if bytes.len() as u64 > max_bytes {
        return Err(GitHubCacheError::Corrupt {
            source: source.label(),
            detail: format!("cached file exceeds {max_bytes} bytes"),
        });
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{GitHubArtifact, GitHubFetchError, GitHubFetcher, InMemoryGitHubStore};
    use std::collections::BTreeMap;

    struct FakeTransport {
        responses: BTreeMap<String, GitHubHttpResponse>,
        requests: std::sync::Mutex<Vec<String>>,
    }

    struct ErrorTransport {
        error: GitHubHttpTransportError,
    }

    impl GitHubHttpTransport for ErrorTransport {
        fn get(
            &self,
            _url: &str,
            _max_bytes: u64,
        ) -> Result<GitHubHttpResponse, GitHubHttpTransportError> {
            Err(self.error.clone())
        }
    }

    impl GitHubHttpTransport for FakeTransport {
        fn get(
            &self,
            url: &str,
            _max_bytes: u64,
        ) -> Result<GitHubHttpResponse, GitHubHttpTransportError> {
            self.requests.lock().expect("lock").push(url.to_string());
            self.responses
                .get(url)
                .cloned()
                .ok_or_else(|| GitHubHttpTransportError::Request("fixture miss".to_string()))
        }
    }

    fn source() -> GitHubSource {
        PackageSource::github_at(
            "acme/packages",
            "0123456789abcdef0123456789abcdef01234567",
            "packages/http",
        )
        .expect("source")
        .github_source()
        .expect("github source")
    }

    fn manifest() -> Vec<u8> {
        b"[package]\nname = \"acme.http\"\nversion = \"1.0.0\"\nentry = \"src/main.aipo\"\n"
            .to_vec()
    }

    fn entry() -> Vec<u8> {
        b"export answer\nfn answer()\nreturn 42\nend\n".to_vec()
    }

    #[test]
    fn fetches_manifest_then_entry_without_credentials() {
        let source = source();
        let base = format!(
            "https://raw.githubusercontent.com/acme/packages/{}/packages/http",
            source.revision
        );
        let mut responses = BTreeMap::new();
        responses.insert(
            format!("{base}/aipo.toml"),
            GitHubHttpResponse {
                status: 200,
                location: None,
                body: manifest(),
            },
        );
        responses.insert(
            format!("{base}/src/main.aipo"),
            GitHubHttpResponse {
                status: 200,
                location: None,
                body: entry(),
            },
        );
        let transport = FakeTransport {
            responses,
            requests: std::sync::Mutex::new(Vec::new()),
        };
        let fetcher = GitHubHttpFetcher::with_transport(transport);
        let artifact = GitHubFetcher::fetch(
            &fetcher,
            &PackageSource::GitHub {
                repository: source.repository.clone(),
                revision: source.revision.clone(),
                subpath: source.subpath.clone(),
            },
        )
        .expect("fetch succeeds");
        assert_eq!(artifact.manifest, manifest());
        assert_eq!(artifact.entry, entry());
    }

    #[test]
    fn rejects_status_redirect_and_transport_failures() {
        let source = source();
        let package_source = PackageSource::GitHub {
            repository: source.repository.clone(),
            revision: source.revision.clone(),
            subpath: source.subpath.clone(),
        };
        let url = raw_url(&source, "aipo.toml").expect("url");
        let mut responses = BTreeMap::new();
        responses.insert(
            url.clone(),
            GitHubHttpResponse {
                status: 302,
                location: Some("https://evil.example".to_string()),
                body: Vec::new(),
            },
        );
        let transport = FakeTransport {
            responses,
            requests: std::sync::Mutex::new(Vec::new()),
        };
        let fetcher = GitHubHttpFetcher::with_transport(transport);
        assert!(matches!(
            GitHubFetcher::fetch(&fetcher, &package_source),
            Err(GitHubFetchError::Redirect { status: 302, .. })
        ));

        let transport = FakeTransport {
            responses: BTreeMap::new(),
            requests: std::sync::Mutex::new(Vec::new()),
        };
        let fetcher = GitHubHttpFetcher::with_transport(transport);
        assert!(matches!(
            GitHubFetcher::fetch(&fetcher, &package_source),
            Err(GitHubFetchError::Transport { .. })
        ));
    }

    #[test]
    fn maps_transport_timeout_and_size_errors() {
        let source = source();
        let package_source = PackageSource::GitHub {
            repository: source.repository.clone(),
            revision: source.revision.clone(),
            subpath: source.subpath.clone(),
        };
        let timeout = GitHubHttpFetcher::with_transport(ErrorTransport {
            error: GitHubHttpTransportError::Timeout,
        });
        assert!(matches!(
            GitHubFetcher::fetch(&timeout, &package_source),
            Err(GitHubFetchError::Transport { .. })
        ));
        let oversized = GitHubHttpFetcher::with_transport(ErrorTransport {
            error: GitHubHttpTransportError::ResponseTooLarge { limit: 7 },
        });
        assert!(matches!(
            GitHubFetcher::fetch(&oversized, &package_source),
            Err(GitHubFetchError::ResponseTooLarge { limit: 7, .. })
        ));
    }

    #[test]
    fn cache_round_trips_and_detects_corruption() {
        let source = PackageSource::github_at(
            "acme/packages",
            "0123456789abcdef0123456789abcdef01234567",
            "packages/http",
        )
        .expect("source");
        let root = std::env::temp_dir().join(format!(
            "aipo-github-cache-test-{}-{}",
            std::process::id(),
            NEXT_CACHE_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let cache = GitHubCache::new(&root);
        let artifact = GitHubArtifact::new(manifest(), entry());
        cache.store(&source, artifact.clone()).expect("store");
        assert_eq!(cache.load(&source).expect("load"), Some(artifact));
        let entry_path = cache
            .root()
            .join("github-v1")
            .join(cache_key(&source.github_source().expect("github")))
            .join("entry.bin");
        fs::write(entry_path, b"corrupt").expect("corrupt fixture");
        assert!(matches!(
            cache.load(&source),
            Err(GitHubCacheError::Corrupt { .. })
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cached_fetcher_populates_and_reuses_verified_entries() {
        let source = PackageSource::github_at(
            "acme/packages",
            "0123456789abcdef0123456789abcdef01234567",
            ".",
        )
        .expect("source");
        let artifact = GitHubArtifact::new(manifest(), entry());
        let mut store = InMemoryGitHubStore::new();
        store
            .insert(source.clone(), artifact.clone())
            .expect("source inserts");
        let root = std::env::temp_dir().join(format!(
            "aipo-cached-fetcher-test-{}-{}",
            std::process::id(),
            NEXT_CACHE_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let cache = GitHubCache::new(&root);
        let fetcher = CachedGitHubFetcher::new(&cache, store);
        let first = GitHubFetcher::fetch(&fetcher, &source).expect("first fetch");
        let second = GitHubFetcher::fetch(&fetcher, &source).expect("cached fetch");
        assert_eq!(first, artifact);
        assert_eq!(second, artifact);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cache_rejects_unsafe_source_and_symlink_root() {
        let source = PackageSource::path("local").expect("path");
        let cache = GitHubCache::new(std::env::temp_dir().join("aipo-cache-not-created"));
        assert!(matches!(
            cache.store(&source, GitHubArtifact::new(manifest(), entry())),
            Err(GitHubCacheError::UnsupportedSource { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn cache_rejects_symlink_components() {
        let base = std::env::temp_dir().join(format!(
            "aipo-github-cache-link-test-{}-{}",
            std::process::id(),
            NEXT_CACHE_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let target = base.join("target");
        let link = base.join("link");
        fs::create_dir_all(&target).expect("target directory");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let cache = GitHubCache::new(link.join("cache"));
        let source = PackageSource::github_at(
            "acme/packages",
            "0123456789abcdef0123456789abcdef01234567",
            ".",
        )
        .expect("source");
        assert!(matches!(
            cache.store(&source, GitHubArtifact::new(manifest(), entry())),
            Err(GitHubCacheError::InvalidRoot { .. })
        ));
        let _ = fs::remove_dir_all(base);
    }
}
