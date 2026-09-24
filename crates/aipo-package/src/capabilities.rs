//! Capability validation and deterministic normalization backed by `aipo-host`.

use aipo_host::{Capability, CapabilityError, CapabilitySet};

/// Parses a package capability declaration into the host's deny-by-default set.
pub fn parse_capabilities(names: &[String]) -> Result<CapabilitySet, CapabilityError> {
    let mut capabilities = CapabilitySet::none();
    for name in names {
        capabilities.grant(Capability::parse(name)?);
    }
    Ok(capabilities)
}

/// Parses and minimally normalizes package capability names.
pub fn normalize_capabilities(names: &[String]) -> Result<Vec<String>, CapabilityError> {
    Ok(parse_capabilities(names)?
        .iter()
        .map(ToString::to_string)
        .collect())
}

/// Computes the deterministic union of capability sets.
pub fn union_capabilities<'a>(sets: impl IntoIterator<Item = &'a CapabilitySet>) -> CapabilitySet {
    let mut union = CapabilitySet::none();
    for set in sets {
        for capability in set.iter() {
            union.grant(capability.clone());
        }
    }
    union
}

/// Returns the first capability in `capabilities` not covered by `root`.
///
/// The returned capability is cloned so callers can retain their original declaration set
/// while reporting a stable upper-bound violation.
#[must_use]
pub fn first_outside_bound(
    root: &CapabilitySet,
    capabilities: &CapabilitySet,
) -> Option<Capability> {
    capabilities
        .iter()
        .find(|capability| !root.allows(capability))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_normalization_uses_the_host_hierarchy() {
        let names = vec![
            "clock.wall".to_string(),
            "clock".to_string(),
            "filesystem.read".to_string(),
        ];
        assert_eq!(
            normalize_capabilities(&names).expect("valid capabilities"),
            vec!["clock", "filesystem.read"]
        );
    }

    #[test]
    fn capability_union_and_upper_bound_are_deterministic() {
        let root = parse_capabilities(&["clock".to_string()]).expect("root capabilities");
        let dependency =
            parse_capabilities(&["clock.wall".to_string()]).expect("dependency capabilities");
        let union = union_capabilities([&root, &dependency]);
        assert!(union.allows(&Capability::parse("clock.monotonic").expect("valid")));
        assert!(first_outside_bound(&root, &union).is_none());

        let outside = parse_capabilities(&["network.http".to_string()]).expect("valid capability");
        assert_eq!(
            first_outside_bound(&root, &outside)
                .expect("outside capability")
                .name(),
            "network.http"
        );
    }

    #[test]
    fn invalid_capability_names_are_rejected() {
        assert!(parse_capabilities(&["Clock".to_string()]).is_err());
        assert!(parse_capabilities(&["clock..wall".to_string()]).is_err());
    }
}
