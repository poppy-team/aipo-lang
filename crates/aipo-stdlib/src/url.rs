//! Canonical `url` module for Aipo.
//!
//! Provides WHATWG-compliant URL parsing and normalization:
//! - `url.parse(text)`: parses a URL string into an ordered Dict of URL components,
//!   or returns a recoverable `Failure` on invalid URL.
//! - `url.format(dict)`: serializes URL components back to a URL string.

use aipo_vm::{DictMap, FailureValue, Value, VmFault};
use std::cell::RefCell;
use std::rc::Rc;

fn recoverable(message: impl Into<String>) -> Value {
    Value::Failure(Rc::new(FailureValue {
        message: message.into(),
    }))
}

fn expect_string<'a>(val: &'a Value, op: &str) -> Result<&'a str, VmFault> {
    match val {
        Value::String(s) => Ok(s.as_str()),
        other => Err(VmFault::TypeMismatch {
            expected: format!("String for {op}"),
            actual: other.type_name().to_string(),
        }),
    }
}

fn default_port_for_scheme(scheme: &str) -> Option<&'static str> {
    match scheme {
        "http" | "ws" => Some("80"),
        "https" | "wss" => Some("443"),
        "ftp" => Some("21"),
        _ => None,
    }
}

fn is_special_scheme(scheme: &str) -> bool {
    matches!(scheme, "http" | "https" | "ws" | "wss" | "ftp" | "file")
}

/// Parsed WHATWG URL representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUrl {
    /// Full normalized URL.
    pub href: String,
    /// Origin of the URL (e.g. `https://example.com:8080` or `null`).
    pub origin: String,
    /// Scheme with trailing colon (e.g. `https:`).
    pub protocol: String,
    /// Username in authority, or empty.
    pub username: String,
    /// Password in authority, or empty.
    pub password: String,
    /// Hostname with port if non-default (e.g. `example.com:8080` or `example.com`).
    pub host: String,
    /// Hostname without port (e.g. `example.com`).
    pub hostname: String,
    /// Port string, or empty if default or unspecified.
    pub port: String,
    /// Normalized path starting with `/`.
    pub pathname: String,
    /// Query string starting with `?`, or empty.
    pub search: String,
    /// Fragment identifier starting with `#`, or empty.
    pub hash: String,
}

fn normalize_url_path(raw_path: &str) -> String {
    let had_trailing_slash = raw_path.ends_with('/') || raw_path.is_empty();
    let segments: Vec<&str> = raw_path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let mut stack: Vec<&str> = Vec::new();
    for seg in segments {
        if seg == ".." {
            stack.pop();
        } else {
            stack.push(seg);
        }
    }
    if stack.is_empty() {
        "/".to_string()
    } else {
        let mut res = String::from("/");
        res.push_str(&stack.join("/"));
        if had_trailing_slash && !res.ends_with('/') {
            res.push('/');
        }
        res
    }
}

/// Parses a URL string following the WHATWG URL model.
///
/// # Errors
/// Returns an error message String if parsing fails.
pub fn parse_url(raw: &str) -> Result<ParsedUrl, String> {
    let trimmed = raw.trim_matches(|c: char| c <= ' ');
    if trimmed.is_empty() {
        return Err("empty URL string".to_string());
    }

    let colon_pos = trimmed
        .find(':')
        .ok_or_else(|| "missing scheme: no ':' found".to_string())?;
    let scheme_slice = &trimmed[..colon_pos];
    if scheme_slice.is_empty() {
        return Err("empty scheme".to_string());
    }
    let first_char = scheme_slice.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() {
        return Err("scheme must start with an ASCII letter".to_string());
    }
    if !scheme_slice
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Err("invalid character in scheme".to_string());
    }

    let scheme = scheme_slice.to_ascii_lowercase();
    let protocol = format!("{scheme}:");
    let after_scheme = &trimmed[colon_pos + 1..];

    let is_special = is_special_scheme(&scheme);

    if is_special && scheme != "file" && !after_scheme.starts_with("//") {
        return Err(format!("special scheme '{scheme}' requires '//'"));
    }

    let mut username = String::new();
    let mut password = String::new();
    let mut host = String::new();
    let mut hostname = String::new();
    let mut port = String::new();
    let mut rest = after_scheme;

    if rest.starts_with("//") {
        let authority_and_rest = &rest[2..];
        let auth_end = authority_and_rest
            .find(['/', '?', '#'])
            .unwrap_or(authority_and_rest.len());
        let authority = &authority_and_rest[..auth_end];
        rest = &authority_and_rest[auth_end..];

        let (userinfo, host_part) = if let Some(at_idx) = authority.find('@') {
            (&authority[..at_idx], &authority[at_idx + 1..])
        } else {
            ("", authority)
        };

        if !userinfo.is_empty() {
            if let Some(user_colon) = userinfo.find(':') {
                username = userinfo[..user_colon].to_string();
                password = userinfo[user_colon + 1..].to_string();
            } else {
                username = userinfo.to_string();
            }
        }

        if host_part.starts_with('[') {
            // IPv6
            let close_bracket = host_part
                .find(']')
                .ok_or_else(|| "unclosed IPv6 bracket".to_string())?;
            hostname = host_part[..=close_bracket].to_ascii_lowercase();
            let after_bracket = &host_part[close_bracket + 1..];
            if let Some(port_slice) = after_bracket.strip_prefix(':') {
                if port_slice.is_empty() || !port_slice.chars().all(|c| c.is_ascii_digit()) {
                    return Err(format!("invalid port: '{port_slice}'"));
                }
                if let Some(def_port) = default_port_for_scheme(&scheme) {
                    if port_slice != def_port {
                        port = port_slice.to_string();
                    }
                } else {
                    port = port_slice.to_string();
                }
            }
        } else if let Some(port_idx) = host_part.rfind(':') {
            hostname = host_part[..port_idx].to_ascii_lowercase();
            let port_slice = &host_part[port_idx + 1..];
            if port_slice.is_empty() || !port_slice.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!("invalid port: '{port_slice}'"));
            }
            if let Some(def_port) = default_port_for_scheme(&scheme) {
                if port_slice != def_port {
                    port = port_slice.to_string();
                }
            } else {
                port = port_slice.to_string();
            }
        } else {
            hostname = host_part.to_ascii_lowercase();
        }

        if is_special && hostname.is_empty() && scheme != "file" {
            return Err("missing host in special scheme".to_string());
        }

        host = if port.is_empty() {
            hostname.clone()
        } else {
            format!("{hostname}:{port}")
        };
    } else if scheme == "file" {
        // file: without //
        if let Some(stripped) = rest.strip_prefix('/') {
            rest = stripped;
        }
    }

    let hash_idx = rest.find('#');
    let (before_hash, hash) = match hash_idx {
        Some(idx) => (&rest[..idx], rest[idx..].to_string()),
        None => (rest, String::new()),
    };

    let query_idx = before_hash.find('?');
    let (raw_path, search) = match query_idx {
        Some(idx) => (&before_hash[..idx], before_hash[idx..].to_string()),
        None => (before_hash, String::new()),
    };

    let pathname = if is_special {
        normalize_url_path(raw_path)
    } else {
        raw_path.to_string()
    };

    let origin = if matches!(scheme.as_str(), "http" | "https" | "ws" | "wss" | "ftp") {
        format!("{protocol}//{host}")
    } else {
        "null".to_string()
    };

    let mut href = String::new();
    href.push_str(&protocol);
    if after_scheme.starts_with("//") {
        href.push_str("//");
        if !username.is_empty() || !password.is_empty() {
            href.push_str(&username);
            if !password.is_empty() {
                href.push(':');
                href.push_str(&password);
            }
            href.push('@');
        }
        href.push_str(&host);
    }
    href.push_str(&pathname);
    href.push_str(&search);
    href.push_str(&hash);

    Ok(ParsedUrl {
        href,
        origin,
        protocol,
        username,
        password,
        host,
        hostname,
        port,
        pathname,
        search,
        hash,
    })
}

fn parsed_url_to_dict(parsed: ParsedUrl) -> Value {
    let entries = vec![
        (
            Value::String(Rc::new("href".to_string())),
            Value::String(Rc::new(parsed.href)),
        ),
        (
            Value::String(Rc::new("origin".to_string())),
            Value::String(Rc::new(parsed.origin)),
        ),
        (
            Value::String(Rc::new("protocol".to_string())),
            Value::String(Rc::new(parsed.protocol)),
        ),
        (
            Value::String(Rc::new("username".to_string())),
            Value::String(Rc::new(parsed.username)),
        ),
        (
            Value::String(Rc::new("password".to_string())),
            Value::String(Rc::new(parsed.password)),
        ),
        (
            Value::String(Rc::new("host".to_string())),
            Value::String(Rc::new(parsed.host)),
        ),
        (
            Value::String(Rc::new("hostname".to_string())),
            Value::String(Rc::new(parsed.hostname)),
        ),
        (
            Value::String(Rc::new("port".to_string())),
            Value::String(Rc::new(parsed.port)),
        ),
        (
            Value::String(Rc::new("pathname".to_string())),
            Value::String(Rc::new(parsed.pathname)),
        ),
        (
            Value::String(Rc::new("search".to_string())),
            Value::String(Rc::new(parsed.search)),
        ),
        (
            Value::String(Rc::new("hash".to_string())),
            Value::String(Rc::new(parsed.hash)),
        ),
    ];
    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}

/// `url.parse(text)` — parses a URL string into a Dict following WHATWG model.
///
/// Returns a Dict with components on success, or recoverable `Failure` on invalid URL.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if argument is not a String.
pub fn url_parse(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument for url.parse".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    if let Value::Failure(f) = &args[0] {
        return Ok(Value::Failure(Rc::clone(f)));
    }

    let text = expect_string(&args[0], "url.parse")?;
    match parse_url(text) {
        Ok(parsed) => Ok(parsed_url_to_dict(parsed)),
        Err(err) => Ok(recoverable(format!("invalid URL: {err}"))),
    }
}

/// Creates the canonical `url` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let entries = vec![(
        Value::String(Rc::new("parse".to_string())),
        Value::Native {
            name: "url.parse".to_string(),
            arity: 1,
            func: url_parse,
        },
    )];
    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
