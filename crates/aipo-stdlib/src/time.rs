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

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use aipo_diagnostics::DiagnosticCode;
use aipo_vm::{FailureValue, StructInstance, Value, Vm, VmFault};

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

/// A deterministic clock source with a controllable virtual time.
#[derive(Debug, Clone, Default)]
pub struct DeterministicClock {
    /// The current virtual timestamp in seconds.
    pub seconds: f64,
}

impl ClockSource for DeterministicClock {
    fn wall_seconds(&self) -> f64 {
        self.seconds
    }

    fn monotonic_seconds(&self) -> f64 {
        self.seconds
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

/// Checks if a year is a leap year in the Gregorian calendar.
#[must_use]
pub fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in a given month of a given year.
#[must_use]
pub fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Computes the number of days since the Unix epoch (1970-01-01) for a civil date.
#[must_use]
pub fn days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = if month <= 2 { month + 12 } else { month };
    let doy = (153 * (m - 3) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn make_date_struct(year: i64, month: i64, day: i64) -> Result<Value, VmFault> {
    use aipo_vm::{FailureValue, StructInstance};
    use std::cell::RefCell;
    use std::rc::Rc;

    if !(1..=9999).contains(&year) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "year {year} out of valid range 1..=9999"
        )))));
    }
    if !(1..=12).contains(&month) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "month {month} out of valid range 1..=12"
        )))));
    }
    let max_d = days_in_month(year, month);
    if day < 1 || day > max_d {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "day {day} out of valid range 1..={max_d} for month {month} in year {year}"
        )))));
    }

    let inst = StructInstance {
        type_name: "Date".to_string(),
        fields: vec![
            ("year".to_string(), Value::Int(year)),
            ("month".to_string(), Value::Int(month)),
            ("day".to_string(), Value::Int(day)),
        ],
        fixed_fields: HashSet::new(),
        under_construction: false,
    };
    Ok(Value::Struct(Rc::new(RefCell::new(inst))))
}

fn make_time_struct(
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
) -> Result<Value, VmFault> {
    if !(0..=23).contains(&hour) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "hour {hour} out of range 0..=23"
        )))));
    }
    if !(0..=59).contains(&minute) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "minute {minute} out of range 0..=59"
        )))));
    }
    if !(0..=59).contains(&second) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "second {second} out of range 0..=59"
        )))));
    }
    if !(0..=999).contains(&millisecond) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "millisecond {millisecond} out of range 0..=999"
        )))));
    }

    let inst = StructInstance {
        type_name: "TimeOfDay".to_string(),
        fields: vec![
            ("hour".to_string(), Value::Int(hour)),
            ("minute".to_string(), Value::Int(minute)),
            ("second".to_string(), Value::Int(second)),
            ("millisecond".to_string(), Value::Int(millisecond)),
        ],
        fixed_fields: HashSet::new(),
        under_construction: false,
    };
    Ok(Value::Struct(Rc::new(RefCell::new(inst))))
}

#[allow(clippy::too_many_arguments)]
fn make_datetime_struct(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    millisecond: i64,
    offset_minutes: i64,
) -> Result<Value, VmFault> {
    let date_val = make_date_struct(year, month, day)?;
    if let Value::Failure(_) = date_val {
        return Ok(date_val);
    }
    let time_val = make_time_struct(hour, minute, second, millisecond)?;
    if let Value::Failure(_) = time_val {
        return Ok(time_val);
    }

    if !(-840..=840).contains(&offset_minutes) {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "offset_minutes {offset_minutes} out of range -840..=840"
        )))));
    }

    let inst = StructInstance {
        type_name: "DateTime".to_string(),
        fields: vec![
            ("year".to_string(), Value::Int(year)),
            ("month".to_string(), Value::Int(month)),
            ("day".to_string(), Value::Int(day)),
            ("hour".to_string(), Value::Int(hour)),
            ("minute".to_string(), Value::Int(minute)),
            ("second".to_string(), Value::Int(second)),
            ("millisecond".to_string(), Value::Int(millisecond)),
            ("offset_minutes".to_string(), Value::Int(offset_minutes)),
        ],
        fixed_fields: HashSet::new(),
        under_construction: false,
    };
    Ok(Value::Struct(Rc::new(RefCell::new(inst))))
}

fn read_int(val: &Value, name: &str) -> Result<i64, VmFault> {
    match val {
        Value::Int(n) => Ok(*n),
        Value::Byte(b) => Ok(i64::from(*b)),
        other => Err(VmFault::TypeMismatch {
            expected: format!("Int for {name}"),
            actual: other.type_name().to_string(),
        }),
    }
}

/// `time.date(year, month, day)`
pub fn time_date(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 3, "time.date")?;
    let y = read_int(&args[0], "year")?;
    let m = read_int(&args[1], "month")?;
    let d = read_int(&args[2], "day")?;
    make_date_struct(y, m, d)
}

/// `time.time_of_day(hour, minute, second = 0, millisecond = 0)`
pub fn time_time_of_day(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 2 || args.len() > 4 {
        return Err(VmFault::TypeMismatch {
            expected: "2 to 4 arguments for time.time_of_day".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let h = read_int(&args[0], "hour")?;
    let m = read_int(&args[1], "minute")?;
    let s = if args.len() >= 3 {
        read_int(&args[2], "second")?
    } else {
        0
    };
    let ms = if args.len() >= 4 {
        read_int(&args[3], "millisecond")?
    } else {
        0
    };
    make_time_struct(h, m, s, ms)
}

/// `time.date_time(...)`
pub fn time_date_time(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() == 2 || args.len() == 3 {
        // (date, time_of_day, [offset_minutes])
        let (y, m, d) = match &args[0] {
            Value::Struct(inst) if inst.borrow().type_name == "Date" => {
                let b = inst.borrow();
                let y = match b.get_field("year") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                let m = match b.get_field("month") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                let d = match b.get_field("day") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                (y, m, d)
            }
            other => {
                return Err(VmFault::TypeMismatch {
                    expected: "Date struct as first argument".to_string(),
                    actual: other.type_name().to_string(),
                });
            }
        };
        let (h, min, s, ms) = match &args[1] {
            Value::Struct(inst) if inst.borrow().type_name == "TimeOfDay" => {
                let b = inst.borrow();
                let h = match b.get_field("hour") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                let min = match b.get_field("minute") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                let s = match b.get_field("second") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                let ms = match b.get_field("millisecond") {
                    Some(Value::Int(n)) => *n,
                    _ => 0,
                };
                (h, min, s, ms)
            }
            other => {
                return Err(VmFault::TypeMismatch {
                    expected: "TimeOfDay struct as second argument".to_string(),
                    actual: other.type_name().to_string(),
                });
            }
        };
        let offset = if args.len() == 3 {
            read_int(&args[2], "offset_minutes")?
        } else {
            0
        };
        make_datetime_struct(y, m, d, h, min, s, ms, offset)
    } else if args.len() >= 4 && args.len() <= 8 {
        let y = read_int(&args[0], "year")?;
        let m = read_int(&args[1], "month")?;
        let d = read_int(&args[2], "day")?;
        let h = read_int(&args[3], "hour")?;
        let min = if args.len() >= 5 {
            read_int(&args[4], "minute")?
        } else {
            0
        };
        let s = if args.len() >= 6 {
            read_int(&args[5], "second")?
        } else {
            0
        };
        let ms = if args.len() >= 7 {
            read_int(&args[6], "millisecond")?
        } else {
            0
        };
        let offset = if args.len() >= 8 {
            read_int(&args[7], "offset_minutes")?
        } else {
            0
        };
        make_datetime_struct(y, m, d, h, min, s, ms, offset)
    } else {
        Err(VmFault::TypeMismatch {
            expected: "2, 3 or 4..=8 arguments for time.date_time".to_string(),
            actual: format!("{} arguments", args.len()),
        })
    }
}

/// `time.duration(seconds)`
pub fn time_duration(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "time.duration")?;
    match &args[0] {
        Value::Float(f) => Ok(Value::Duration(*f)),
        Value::Int(n) => Ok(Value::Duration(*n as f64)),
        Value::Duration(d) => Ok(Value::Duration(*d)),
        other => Err(VmFault::TypeMismatch {
            expected: "Float or Int for time.duration".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// `time.parse_date(text)`
pub fn time_parse_date(args: &[Value]) -> Result<Value, VmFault> {
    use aipo_vm::FailureValue;
    use std::rc::Rc;

    require_arity(args, 1, "time.parse_date")?;
    let Value::String(s) = &args[0] else {
        return Err(VmFault::TypeMismatch {
            expected: "String for time.parse_date".to_string(),
            actual: args[0].type_name().to_string(),
        });
    };

    let text = s.trim();
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid date format \"{text}\", expected YYYY-MM-DD"
        )))));
    }

    let Ok(y) = parts[0].parse::<i64>() else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid year in date: \"{}\"",
            parts[0]
        )))));
    };
    let Ok(m) = parts[1].parse::<i64>() else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid month in date: \"{}\"",
            parts[1]
        )))));
    };
    let Ok(d) = parts[2].parse::<i64>() else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid day in date: \"{}\"",
            parts[2]
        )))));
    };

    make_date_struct(y, m, d)
}

/// `time.parse_time(text)`
pub fn time_parse_time(args: &[Value]) -> Result<Value, VmFault> {
    use aipo_vm::FailureValue;
    use std::rc::Rc;

    require_arity(args, 1, "time.parse_time")?;
    let Value::String(s) = &args[0] else {
        return Err(VmFault::TypeMismatch {
            expected: "String for time.parse_time".to_string(),
            actual: args[0].type_name().to_string(),
        });
    };

    let text = s.trim();
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid time format \"{text}\", expected HH:MM[:SS[.sss]]"
        )))));
    }

    let Ok(h) = parts[0].parse::<i64>() else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid hour in time: \"{}\"",
            parts[0]
        )))));
    };
    let Ok(min) = parts[1].parse::<i64>() else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid minute in time: \"{}\"",
            parts[1]
        )))));
    };

    let (s, ms) = if parts.len() == 3 {
        let sec_str = parts[2];
        if let Some((sec_p, ms_p)) = sec_str.split_once('.') {
            let Ok(sec) = sec_p.parse::<i64>() else {
                return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
                    "invalid second: \"{sec_p}\""
                )))));
            };
            let mut ms_text = ms_p.to_string();
            if ms_text.len() > 3 {
                ms_text.truncate(3);
            }
            while ms_text.len() < 3 {
                ms_text.push('0');
            }
            let ms = ms_text.parse::<i64>().unwrap_or(0);
            (sec, ms)
        } else {
            let Ok(sec) = sec_str.parse::<i64>() else {
                return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
                    "invalid second: \"{sec_str}\""
                )))));
            };
            (sec, 0)
        }
    } else {
        (0, 0)
    };

    make_time_struct(h, min, s, ms)
}

/// `time.parse_iso(text)` / `time.parse_datetime(text)`
pub fn time_parse_iso(args: &[Value]) -> Result<Value, VmFault> {
    use aipo_vm::FailureValue;
    use std::rc::Rc;

    require_arity(args, 1, "time.parse_iso")?;
    let Value::String(s) = &args[0] else {
        return Err(VmFault::TypeMismatch {
            expected: "String for time.parse_iso".to_string(),
            actual: args[0].type_name().to_string(),
        });
    };

    let text = s.trim();
    let (date_str, time_and_offset) = if let Some((d, t)) = text.split_once('T') {
        (d, t)
    } else if let Some((d, t)) = text.split_once('t') {
        (d, t)
    } else if let Some((d, t)) = text.split_once(' ') {
        (d, t)
    } else {
        return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "invalid ISO 8601 string \"{text}\": missing 'T' separator"
        )))));
    };

    // Parse date
    let date_val = time_parse_date(&[Value::String(Rc::new(date_str.to_string()))])?;
    let (y, m, d) = match date_val {
        Value::Struct(inst) => {
            let b = inst.borrow();
            let y = match b.get_field("year") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            let m = match b.get_field("month") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            let d = match b.get_field("day") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            (y, m, d)
        }
        Value::Failure(_) => return Ok(date_val),
        _ => unreachable!(),
    };

    // Parse offset
    let (time_str, offset_minutes) =
        if time_and_offset.ends_with('Z') || time_and_offset.ends_with('z') {
            (&time_and_offset[..time_and_offset.len() - 1], 0)
        } else if let Some(idx) = time_and_offset.rfind('+') {
            let (t, off) = time_and_offset.split_at(idx);
            let off_str = &off[1..];
            let off_min = parse_offset(off_str, 1);
            (t, off_min)
        } else if let Some(idx) = time_and_offset.rfind('-') {
            let (t, off) = time_and_offset.split_at(idx);
            let off_str = &off[1..];
            let off_min = parse_offset(off_str, -1);
            (t, off_min)
        } else {
            (time_and_offset, 0)
        };

    let time_val = time_parse_time(&[Value::String(Rc::new(time_str.to_string()))])?;
    let (h, min, s, ms) = match time_val {
        Value::Struct(inst) => {
            let b = inst.borrow();
            let h = match b.get_field("hour") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            let min = match b.get_field("minute") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            let s = match b.get_field("second") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            let ms = match b.get_field("millisecond") {
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            (h, min, s, ms)
        }
        Value::Failure(_) => return Ok(time_val),
        _ => unreachable!(),
    };

    make_datetime_struct(y, m, d, h, min, s, ms, offset_minutes)
}

fn parse_offset(s: &str, sign: i64) -> i64 {
    if let Some((h_str, m_str)) = s.split_once(':') {
        let h = h_str.parse::<i64>().unwrap_or(0);
        let m = m_str.parse::<i64>().unwrap_or(0);
        sign * (h * 60 + m)
    } else if s.len() == 4 {
        let h = s[..2].parse::<i64>().unwrap_or(0);
        let m = s[2..].parse::<i64>().unwrap_or(0);
        sign * (h * 60 + m)
    } else if s.len() == 2 {
        let h = s.parse::<i64>().unwrap_or(0);
        sign * (h * 60)
    } else {
        0
    }
}

// --- Methods on Date, TimeOfDay, DateTime ---

/// Formats a `Date` struct instance as ISO 8601 `YYYY-MM-DD`.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `Date`.
pub fn method_date_to_iso(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "Date.to_iso")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "Date struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let y = match b.get_field("year") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("month") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let d = match b.get_field("day") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    Ok(Value::String(Rc::new(format!("{y:04}-{m:02}-{d:02}"))))
}

/// Formats a `TimeOfDay` struct instance as ISO 8601 `HH:MM:SS[.sss]`.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `TimeOfDay`.
pub fn method_time_to_iso(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "TimeOfDay.to_iso")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "TimeOfDay struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let h = match b.get_field("hour") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("minute") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let s = match b.get_field("second") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let ms = match b.get_field("millisecond") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    if ms > 0 {
        Ok(Value::String(Rc::new(format!(
            "{h:02}:{m:02}:{s:02}.{ms:03}"
        ))))
    } else {
        Ok(Value::String(Rc::new(format!("{h:02}:{m:02}:{s:02}"))))
    }
}

/// Formats a `DateTime` struct instance as ISO 8601 `YYYY-MM-DDTHH:MM:SS[.sss]Z` or with offset.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `DateTime`.
pub fn method_datetime_to_iso(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "DateTime.to_iso")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "DateTime struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let y = match b.get_field("year") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("month") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let d = match b.get_field("day") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let h = match b.get_field("hour") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let min = match b.get_field("minute") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let s = match b.get_field("second") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let ms = match b.get_field("millisecond") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let off = match b.get_field("offset_minutes") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };

    let time_str = if ms > 0 {
        format!("{h:02}:{min:02}:{s:02}.{ms:03}")
    } else {
        format!("{h:02}:{min:02}:{s:02}")
    };

    let offset_str = if off == 0 {
        "Z".to_string()
    } else {
        let sign = if off >= 0 { '+' } else { '-' };
        let abs_off = off.abs();
        let off_h = abs_off / 60;
        let off_m = abs_off % 60;
        format!("{sign}{off_h:02}:{off_m:02}")
    };

    Ok(Value::String(Rc::new(format!(
        "{y:04}-{m:02}-{d:02}T{time_str}{offset_str}"
    ))))
}

/// Extracts the `Date` component from a `DateTime` struct instance.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `DateTime`.
pub fn method_datetime_date(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "DateTime.date")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "DateTime struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let y = match b.get_field("year") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("month") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let d = match b.get_field("day") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    make_date_struct(y, m, d)
}

/// Extracts the `TimeOfDay` component from a `DateTime` struct instance.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `DateTime`.
pub fn method_datetime_time(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "DateTime.time")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "DateTime struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let h = match b.get_field("hour") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("minute") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let s = match b.get_field("second") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let ms = match b.get_field("millisecond") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    make_time_struct(h, m, s, ms)
}

/// Computes the total seconds since Unix epoch (1970-01-01T00:00:00Z) for a `DateTime`.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if called with arguments or receiver is not `DateTime`.
pub fn method_datetime_epoch_seconds(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 0, "DateTime.epoch_seconds")?;
    let Value::Struct(inst) = receiver else {
        return Err(VmFault::TypeMismatch {
            expected: "DateTime struct receiver".to_string(),
            actual: receiver.type_name().to_string(),
        });
    };
    let b = inst.borrow();
    let y = match b.get_field("year") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let m = match b.get_field("month") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let d = match b.get_field("day") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let h = match b.get_field("hour") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let min = match b.get_field("minute") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let s = match b.get_field("second") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let ms = match b.get_field("millisecond") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };
    let off = match b.get_field("offset_minutes") {
        Some(Value::Int(n)) => *n,
        _ => 0,
    };

    let days = days_since_epoch(y, m, d);
    let total_sec = days * 86_400 + h * 3600 + min * 60 + s - (off * 60);
    let total = total_sec as f64 + (ms as f64) / 1000.0;
    Ok(Value::Float(total))
}

/// Registers methods for Date, TimeOfDay, and DateTime on the VM.
pub fn register_methods(vm: &mut Vm) {
    vm.register_method_native("Date", "to_iso", 0, method_date_to_iso);
    vm.register_method_native("Date", "to_string", 0, method_date_to_iso);

    vm.register_method_native("TimeOfDay", "to_iso", 0, method_time_to_iso);
    vm.register_method_native("TimeOfDay", "to_string", 0, method_time_to_iso);

    vm.register_method_native("DateTime", "to_iso", 0, method_datetime_to_iso);
    vm.register_method_native("DateTime", "to_string", 0, method_datetime_to_iso);
    vm.register_method_native("DateTime", "date", 0, method_datetime_date);
    vm.register_method_native("DateTime", "time", 0, method_datetime_time);
    vm.register_method_native(
        "DateTime",
        "epoch_seconds",
        0,
        method_datetime_epoch_seconds,
    );
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
            Value::native("time.now", 0, time_now),
        ),
        (
            Value::String(Rc::new("monotonic".to_string())),
            Value::native("time.monotonic", 0, time_monotonic),
        ),
        (
            Value::String(Rc::new("date".to_string())),
            Value::native("time.date", 3, time_date),
        ),
        (
            Value::String(Rc::new("time_of_day".to_string())),
            Value::native("time.time_of_day", usize::MAX, time_time_of_day),
        ),
        (
            Value::String(Rc::new("date_time".to_string())),
            Value::native("time.date_time", usize::MAX, time_date_time),
        ),
        (
            Value::String(Rc::new("parse_date".to_string())),
            Value::native("time.parse_date", 1, time_parse_date),
        ),
        (
            Value::String(Rc::new("parse_time".to_string())),
            Value::native("time.parse_time", 1, time_parse_time),
        ),
        (
            Value::String(Rc::new("parse_iso".to_string())),
            Value::native("time.parse_iso", 1, time_parse_iso),
        ),
        (
            Value::String(Rc::new("parse_datetime".to_string())),
            Value::native("time.parse_datetime", 1, time_parse_iso),
        ),
        (
            Value::String(Rc::new("duration".to_string())),
            Value::native("time.duration", 1, time_duration),
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
        assert_eq!(
            names,
            vec![
                "now".to_string(),
                "monotonic".to_string(),
                "date".to_string(),
                "time_of_day".to_string(),
                "date_time".to_string(),
                "parse_date".to_string(),
                "parse_time".to_string(),
                "parse_iso".to_string(),
                "parse_datetime".to_string(),
                "duration".to_string(),
            ]
        );
    }
}
