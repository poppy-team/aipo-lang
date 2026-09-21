//! Canonical `time` module for Aipo: the clock reads that canon classifies as host capabilities.
//!
//! Canon is explicit that a nondeterministic source is a *host capability*, not a language
//! primitive, and that a deterministic test profile may replace it. This module implements
//! exactly that shape and nothing more:
//!
//! - `time.now()` — wall clock, behind the `clock.wall` capability.
//! - `time.monotonic()` — monotonic clock, behind the `clock.monotonic` capability.
//!
//! The clock is an installed service, not a permanently available function. With no service
//! installed the reading is not faked and not silently defaulted: it faults with
//! `AIPO_RT_CAPABILITY_DENIED`, which is the same observable behaviour as any other denied
//! capability. Installing no clock is therefore the deny-by-default profile.
//!
//! Both readings return a [`Value::Duration`] measured in seconds, which is the portable
//! representation the language already has; `Date`/`TimeOfDay`/`DateTime` are deliberately not
//! invented here, because their calendar and offset contracts are a separate concern with their
//! own decision.
//!
//! The service is process-global for the same reason [`crate::io`]'s output sink is: a native
//! function is a plain `fn(&[Value])`, so the host configures the environment the natives run
//! in. A host that needs a per-run clock installs one per run.

use std::sync::{LazyLock, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aipo_diagnostics::DiagnosticCode;
use aipo_vm::{Value, VmFault};

/// A source of wall-clock and monotonic readings, in seconds.
///
/// Implemented by the real host and by deterministic test profiles; the language cannot tell
/// the two apart, which is what makes a replay possible.
pub trait ClockSource: Send + 'static {
    /// Seconds since the Unix epoch.
    fn wall_seconds(&self) -> f64;

    /// Seconds from an arbitrary fixed origin, never decreasing.
    fn monotonic_seconds(&self) -> f64;
}

/// The default source: the operating system's clocks.
#[derive(Debug, Default)]
pub struct SystemClock;

impl ClockSource for SystemClock {
    fn wall_seconds(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |elapsed| elapsed.as_secs_f64())
    }

    fn monotonic_seconds(&self) -> f64 {
        // `Instant` has no epoch of its own, so a process-wide origin gives the value a stable
        // meaning across readings; only differences are contractually meaningful.
        ORIGIN.elapsed().as_secs_f64()
    }
}

static ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);

/// The installed clock source, if the `clock` capability was granted.
static CLOCK: LazyLock<Mutex<Option<Box<dyn ClockSource>>>> = LazyLock::new(|| Mutex::new(None));

/// Grants the `clock` capability by installing the host's clock.
///
/// The host calls this once per run; a deterministic profile installs a controlled source here
/// instead of [`SystemClock`].
pub fn install_clock(source: Box<dyn ClockSource>) {
    if let Ok(mut guard) = CLOCK.lock() {
        *guard = Some(source);
    }
}

/// Denies the `clock` capability by removing the installed clock.
///
/// After this, `time.now()` and `time.monotonic()` fault. This is the state a run starts in, so
/// a host that wants the capability has to say so.
pub fn revoke_clock() {
    if let Ok(mut guard) = CLOCK.lock() {
        *guard = None;
    }
}

/// Whether the `clock` capability is granted in this process.
#[must_use]
pub fn clock_granted() -> bool {
    CLOCK.lock().is_ok_and(|guard| guard.is_some())
}

fn read_clock(
    capability: &str,
    operation: &str,
    read: impl FnOnce(&dyn ClockSource) -> f64,
) -> Result<Value, VmFault> {
    let guard = CLOCK.lock().map_err(|_| VmFault::CorruptedBytecode {
        offset: 0,
        reason: "clock service mutex was poisoned".to_string(),
    })?;
    let Some(source) = guard.as_deref() else {
        // Denied: no fake default, no zero, no silently different API surface.
        return Err(VmFault::CapabilityDenied {
            capability: capability.to_string(),
            operation: operation.to_string(),
        });
    };
    Ok(Value::Duration(read(source)))
}

fn require_arity(args: &[Value], expected: usize, op: &str) -> Result<(), VmFault> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(VmFault::TypeMismatch {
            expected: format!("{expected} argument(s) for {op}"),
            actual: format!("{} arguments", args.len()),
        })
    }
}

/// `time.now()` — the wall clock, behind the `clock.wall` capability.
///
/// # Errors
///
/// [`VmFault::CapabilityDenied`] when no clock is installed, and [`VmFault::TypeMismatch`] if
/// called with arguments.
pub fn time_now(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "time.now")?;
    read_clock("clock.wall", "time.now", ClockSource::wall_seconds)
}

/// `time.monotonic()` — the monotonic clock, behind the `clock.monotonic` capability.
///
/// # Errors
///
/// [`VmFault::CapabilityDenied`] when no clock is installed, and [`VmFault::TypeMismatch`] if
/// called with arguments.
pub fn time_monotonic(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "time.monotonic")?;
    read_clock(
        "clock.monotonic",
        "time.monotonic",
        ClockSource::monotonic_seconds,
    )
}

/// The diagnostic code a denied clock reading reports.
///
/// Exposed so tests assert the canon code by name instead of by numeric value.
#[must_use]
pub const fn clock_denied_code() -> DiagnosticCode {
    DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
}

/// Constructs the canonical `time` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    use std::cell::RefCell;
    use std::rc::Rc;

    use aipo_vm::DictMap;

    let entries = vec![
        (
            Value::String(Rc::new("now".to_string())),
            Value::Native {
                name: "time.now".to_string(),
                arity: 0,
                func: time_now,
            },
        ),
        (
            Value::String(Rc::new("monotonic".to_string())),
            Value::Native {
                name: "time.monotonic".to_string(),
                arity: 0,
                func: time_monotonic,
            },
        ),
    ];

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A controllable source, standing in for the deterministic test profile canon describes.
    struct FixedClock {
        wall: f64,
        monotonic: f64,
    }

    impl ClockSource for FixedClock {
        fn wall_seconds(&self) -> f64 {
            self.wall
        }

        fn monotonic_seconds(&self) -> f64 {
            self.monotonic
        }
    }

    /// A source that advances per reading, to prove the second reading is really consulted.
    struct TickingClock(AtomicUsize);

    impl ClockSource for TickingClock {
        fn wall_seconds(&self) -> f64 {
            self.0.fetch_add(1, Ordering::SeqCst) as f64
        }

        fn monotonic_seconds(&self) -> f64 {
            self.wall_seconds()
        }
    }

    /// Serializes the tests: the service is process-global by design.
    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_reading_without_the_capability_faults_with_the_canon_code() {
        let _guard = LOCK.lock().expect("test lock");
        revoke_clock();
        assert!(!clock_granted());
        let fault = time_now(&[]).expect_err("denied");
        assert_eq!(fault.diagnostic_code(), clock_denied_code());
        assert!(fault.to_string().contains("clock.wall"));
        // The same denial covers the monotonic reading, and the fault names it specifically
        // rather than reporting the generic module denial.
        let fault = time_monotonic(&[]).expect_err("denied");
        assert!(fault.to_string().contains("clock.monotonic"));
    }

    #[test]
    fn test_granted_clock_reports_the_source_reading() {
        let _guard = LOCK.lock().expect("test lock");
        install_clock(Box::new(FixedClock {
            wall: 1_700_000_000.5,
            monotonic: 12.25,
        }));
        assert_eq!(
            time_now(&[]).expect("granted"),
            Value::Duration(1_700_000_000.5)
        );
        assert_eq!(
            time_monotonic(&[]).expect("granted"),
            Value::Duration(12.25)
        );
    }

    #[test]
    fn test_a_deterministic_profile_replays_identically() {
        let _guard = LOCK.lock().expect("test lock");
        install_clock(Box::new(FixedClock {
            wall: 42.0,
            monotonic: 0.0,
        }));
        let first = time_now(&[]).expect("granted");
        let second = time_now(&[]).expect("granted");
        assert_eq!(first, second, "a fixed source must replay");
    }

    #[test]
    fn test_readings_consult_the_source_each_call() {
        let _guard = LOCK.lock().expect("test lock");
        install_clock(Box::new(TickingClock(AtomicUsize::new(10))));
        assert_eq!(time_monotonic(&[]).expect("granted"), Value::Duration(10.0));
        assert_eq!(time_monotonic(&[]).expect("granted"), Value::Duration(11.0));
    }

    #[test]
    fn test_revoking_the_capability_after_a_grant_denies_again() {
        let _guard = LOCK.lock().expect("test lock");
        install_clock(Box::new(SystemClock));
        assert!(clock_granted());
        revoke_clock();
        assert!(time_now(&[]).is_err());
    }

    #[test]
    fn test_arity_is_checked_like_every_other_native() {
        let _guard = LOCK.lock().expect("test lock");
        // Arity is checked before the capability, so a misuse is reported as a misuse even in a
        // denied profile.
        revoke_clock();
        let fault = time_now(&[Value::Int(1)]).expect_err("arity");
        assert_eq!(
            fault.diagnostic_code(),
            DiagnosticCode::AIPO_RT_TYPE_MISMATCH
        );
    }

    #[test]
    fn test_system_clock_is_monotonic_and_plausible() {
        let _guard = LOCK.lock().expect("test lock");
        let clock = SystemClock;
        let first = clock.monotonic_seconds();
        let second = clock.monotonic_seconds();
        assert!(second >= first, "monotonic readings never decrease");
        assert!(
            clock.wall_seconds() > 1_600_000_000.0,
            "wall clock reads a real epoch"
        );
    }

    #[test]
    fn test_module_exposes_both_readings() {
        let module = create_module();
        let Value::Dict(entries) = module else {
            panic!("time module is a dictionary");
        };
        let entries = entries.borrow();
        let names: Vec<String> = entries
            .entries()
            .iter()
            .map(|(key, _)| match key {
                Value::String(text) => text.to_string(),
                other => panic!("module key is a string, got {other:?}"),
            })
            .collect();
        assert_eq!(names, vec!["now".to_string(), "monotonic".to_string()]);
    }
}
