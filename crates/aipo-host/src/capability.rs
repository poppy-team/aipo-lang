//! Deny-by-default capability model for host services.
//!
//! Canon (Fechamento Arquitetural §11) fixes the initial hierarchy and requires that a
//! sandboxed host grants nothing implicitly. A capability is a dotted path, and the path
//! *is* the tree: granting `clock` grants `clock.wall` and `clock.monotonic`, granting
//! `filesystem` grants its three leaves, and a host may add its own namespace (`poppy.ecs`)
//! without this crate learning what it means.
//!
//! Two rules from canon are encoded here and nowhere else:
//!
//! - **Deny by default.** A [`CapabilitySet`] starts empty and must be told what it grants.
//! - **Narrowing only.** A declared capability set is an *upper bound*; host or project
//!   policy may grant less, never more. [`CapabilitySet::narrow`] therefore only ever
//!   removes grants.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::fault::HostFault;

/// The canon capability hierarchy (Fechamento Arquitetural §11).
///
/// Exposed so a target checker can explain what a host surface needs, and so the
/// `poppy.*` family has a concrete spelling (`poppy` grants every `poppy.<name>`).
pub const CANON_CAPABILITIES: &[&str] = &[
    "filesystem.read",
    "filesystem.write",
    "filesystem.roots",
    "network.http",
    "network.tcp",
    "network.udp",
    "process.spawn",
    "env.read",
    "clock.wall",
    "clock.monotonic",
    "crypto.random",
    "crypto.keystore",
    "poppy",
];

/// A capability path could not be constructed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// The path was empty.
    Empty,
    /// A segment did not match `[a-z][a-z0-9_-]*`.
    InvalidSegment(String),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "capability path is empty"),
            Self::InvalidSegment(segment) => write!(
                formatter,
                "capability segment '{segment}' must match [a-z][a-z0-9_-]*"
            ),
        }
    }
}

impl std::error::Error for CapabilityError {}

/// A dotted capability path such as `filesystem.read` or `clock.wall`.
///
/// Ordering and hashing are over the path text, so a set of capabilities is deterministic
/// regardless of the order the host declared them in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability(String);

impl Capability {
    /// `filesystem.read` — reading host files.
    pub const FILESYSTEM_READ: &'static str = "filesystem.read";
    /// `filesystem.write` — writing host files.
    pub const FILESYSTEM_WRITE: &'static str = "filesystem.write";
    /// `filesystem.roots` — learning which roots the host exposes.
    pub const FILESYSTEM_ROOTS: &'static str = "filesystem.roots";
    /// `network.http` — HTTP requests.
    pub const NETWORK_HTTP: &'static str = "network.http";
    /// `network.tcp` — raw TCP.
    pub const NETWORK_TCP: &'static str = "network.tcp";
    /// `network.udp` — raw UDP.
    pub const NETWORK_UDP: &'static str = "network.udp";
    /// `process.spawn` — starting processes.
    pub const PROCESS_SPAWN: &'static str = "process.spawn";
    /// `env.read` — reading environment variables.
    pub const ENV_READ: &'static str = "env.read";
    /// `clock.wall` — wall-clock time.
    pub const CLOCK_WALL: &'static str = "clock.wall";
    /// `clock.monotonic` — monotonic time.
    pub const CLOCK_MONOTONIC: &'static str = "clock.monotonic";
    /// `clock` — the umbrella grant covering both clocks.
    pub const CLOCK: &'static str = "clock";
    /// `crypto.random` — host entropy.
    pub const CRYPTO_RANDOM: &'static str = "crypto.random";
    /// `crypto.keystore` — host keystore access.
    pub const CRYPTO_KEYSTORE: &'static str = "crypto.keystore";
    /// `poppy` — the game-host namespace.
    pub const POPPY: &'static str = "poppy";

    /// Parses and validates a dotted capability path.
    ///
    /// # Errors
    ///
    /// [`CapabilityError::Empty`] for an empty path, [`CapabilityError::InvalidSegment`]
    /// for a segment outside `[a-z][a-z0-9_-]*`.
    pub fn parse(path: &str) -> Result<Self, CapabilityError> {
        if path.is_empty() {
            return Err(CapabilityError::Empty);
        }
        for segment in path.split('.') {
            if !is_valid_segment(segment) {
                return Err(CapabilityError::InvalidSegment(segment.to_string()));
            }
        }
        Ok(Self(path.to_string()))
    }

    /// The path text, for diagnostics and AHS round-trips.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// The immediate enclosing namespace, if any: `clock.wall` → `clock`.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('.')
            .map(|(parent, _)| Self(parent.to_string()))
    }

    /// Whether this capability covers `other`.
    ///
    /// A path covers itself and everything below it: `clock` covers `clock.wall`, and
    /// `filesystem` covers every `filesystem.*` leaf. A sibling never covers a sibling.
    #[must_use]
    pub fn grants(&self, other: &Self) -> bool {
        self == other
            || (other.0.len() > self.0.len()
                && other.0.starts_with(&self.0)
                && other.0.as_bytes()[self.0.len()] == b'.')
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl FromStr for Capability {
    type Err = CapabilityError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// Whether a segment matches `[a-z][a-z0-9_-]*`.
fn is_valid_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
}

/// A deny-by-default set of granted capabilities.
///
/// The set is kept minimal: it never stores a grant that another stored grant already
/// covers, so `allows` is a scan over incomparable paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    grants: BTreeSet<Capability>,
}

impl CapabilitySet {
    /// An empty set: every capability is denied. Canon's default for a sandboxed host.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Whether any stored grant covers `capability`.
    #[must_use]
    pub fn allows(&self, capability: &Capability) -> bool {
        self.grants.iter().any(|grant| grant.grants(capability))
    }

    /// Adds a grant, returning whether the set changed.
    ///
    /// Granting a path already covered by a stored grant is a no-op. Granting a *broader*
    /// path drops the narrower grants it subsumes, keeping the set minimal.
    pub fn grant(&mut self, capability: Capability) -> bool {
        if self.allows(&capability) {
            return false;
        }
        self.grants.retain(|existing| !capability.grants(existing));
        self.grants.insert(capability)
    }

    /// Removes a grant, returning whether the set changed.
    ///
    /// Only an exact path is revoked: revoking `clock` does not silently drop a separate
    /// `clock.wall` grant, because that would widen nothing but hide a policy mistake.
    pub fn revoke(&mut self, capability: &Capability) -> bool {
        self.grants.remove(capability)
    }

    /// The upper-bound intersection of two sets: grants kept only where the policy allows
    /// them.
    ///
    /// Canon: a declared set is an upper bound and policy may grant less, never more. A
    /// broad grant the policy only partially allows is dropped rather than silently split,
    /// so the result can only ever deny more than either input.
    #[must_use]
    pub fn narrow(&self, policy: &Self) -> Self {
        Self {
            grants: self
                .grants
                .iter()
                .filter(|grant| policy.allows(grant))
                .cloned()
                .collect(),
        }
    }

    /// Iterates the stored grants in path order.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.grants.iter()
    }

    /// Whether nothing is granted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// Number of stored grants.
    #[must_use]
    pub fn len(&self) -> usize {
        self.grants.len()
    }

    /// Fails when `capability` is not granted, naming the denied operation.
    ///
    /// This is the single door a host service goes through, so a missing capability is
    /// always the same fault with the same stable code.
    ///
    /// # Errors
    ///
    /// [`HostFault::CapabilityDenied`] carrying the capability and the operation.
    pub fn require(&self, capability: &Capability, operation: &str) -> Result<(), HostFault> {
        if self.allows(capability) {
            return Ok(());
        }
        Err(HostFault::CapabilityDenied {
            capability: capability.clone(),
            operation: operation.to_string(),
        })
    }
}

impl FromIterator<Capability> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = Capability>>(iter: T) -> Self {
        let mut set = Self::none();
        for capability in iter {
            set.grant(capability);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(path: &str) -> Capability {
        Capability::parse(path).expect("test capability is valid")
    }

    #[test]
    fn test_parse_rejects_malformed_paths() {
        assert_eq!(Capability::parse(""), Err(CapabilityError::Empty));
        assert_eq!(
            Capability::parse("Clock.wall"),
            Err(CapabilityError::InvalidSegment("Clock".to_string()))
        );
        assert_eq!(
            Capability::parse("clock..wall"),
            Err(CapabilityError::InvalidSegment(String::new()))
        );
        assert!(Capability::parse("poppy.ecs-scope").is_ok());
    }

    #[test]
    fn test_namespace_grants_descendants_not_siblings() {
        let clock = capability("clock");
        assert!(clock.grants(&capability("clock.wall")));
        assert!(clock.grants(&capability("clock.monotonic")));
        assert!(clock.grants(&clock));
        assert!(!clock.grants(&capability("crypto.random")));
        assert!(!clock.grants(&capability("clockwork")));
        assert!(!capability("clock.wall").grants(&clock));
    }

    #[test]
    fn test_deny_by_default() {
        let set = CapabilitySet::none();
        assert!(!set.allows(&capability("clock.wall")));
        assert!(set.is_empty());
        let denied = set.require(&capability("clock.wall"), "time.now");
        assert_eq!(
            denied.expect_err("denied").code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
        );
    }

    #[test]
    fn test_grant_is_minimal_and_covers_descendants() {
        let mut set = CapabilitySet::none();
        assert!(set.grant(capability("clock.wall")));
        assert!(!set.grant(capability("clock.wall")));
        assert!(set.allows(&capability("clock.wall")));
        assert!(!set.allows(&capability("clock.monotonic")));

        // A broader grant subsumes the narrower one instead of storing both.
        assert!(set.grant(capability("clock")));
        assert_eq!(set.len(), 1);
        assert!(set.allows(&capability("clock.monotonic")));
    }

    #[test]
    fn test_narrow_never_widens() {
        let declared: CapabilitySet = [
            capability("clock"),
            capability("filesystem.read"),
            capability("network.http"),
        ]
        .into_iter()
        .collect();
        let policy: CapabilitySet = [capability("clock.wall"), capability("filesystem.read")]
            .into_iter()
            .collect();

        let effective = declared.narrow(&policy);
        // `filesystem.read` is allowed by both, so it survives.
        assert!(effective.allows(&capability("filesystem.read")));
        // `network.http` is not in the policy, so it is gone.
        assert!(!effective.allows(&capability("network.http")));
        // A broad `clock` grant the policy only partially allows is dropped, never split.
        assert!(!effective.allows(&capability("clock")));

        let widening = policy.narrow(&declared);
        assert!(!widening.allows(&capability("network.http")));
    }

    #[test]
    fn test_revoke_removes_only_the_exact_grant() {
        let mut set: CapabilitySet = [capability("clock"), capability("filesystem.read")]
            .into_iter()
            .collect();
        assert!(set.revoke(&capability("clock")));
        assert!(!set.allows(&capability("clock.wall")));
        assert!(set.allows(&capability("filesystem.read")));
        assert!(!set.revoke(&capability("clock")));
    }

    #[test]
    fn test_require_passes_when_an_ancestor_is_granted() {
        let set: CapabilitySet = [capability("clock")].into_iter().collect();
        assert!(
            set.require(&capability("clock.monotonic"), "time.monotonic")
                .is_ok()
        );
    }

    #[test]
    fn test_canon_hierarchy_is_well_formed() {
        for path in CANON_CAPABILITIES {
            let capability = capability(path);
            assert_eq!(capability.name(), *path);
        }
    }
}
