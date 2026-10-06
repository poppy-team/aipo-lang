//! Canonical `sh` system automation module for shell scripting and process control (Marco 4 / ADP-014).

#![forbid(unsafe_code)]

use aipo_vm::{DictMap, FailureValue, Value, VmFault};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

fn expect_string<'a>(arg: &'a Value, op: &str) -> Result<&'a str, VmFault> {
    match arg {
        Value::String(s) => Ok(s.as_str()),
        other => Err(VmFault::TypeMismatch {
            expected: format!("String for {op}"),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Executes an external system command: `sh.run(cmd: String, args: List[String] = []) -> Dict`.
///
/// Returns a Dictionary with:
/// - `"code"`: exit status code as integer (0 for success, non-zero for error, -1 if terminated by signal)
/// - `"stdout"`: standard output text captured
/// - `"stderr"`: standard error text captured
/// - `"ok"`: boolean (`code == 0`)
pub fn sh_run(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() || args.len() > 2 {
        return Err(VmFault::TypeMismatch {
            expected: "1 or 2 arguments for sh.run (cmd, args)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let cmd_name = expect_string(&args[0], "sh.run command")?;
    let mut command = Command::new(cmd_name);

    if args.len() == 2 {
        match &args[1] {
            Value::List(list) => {
                for item in list.borrow().iter() {
                    let s = expect_string(item, "sh.run argument item")?;
                    command.arg(s);
                }
            }
            Value::None => {}
            other => {
                return Err(VmFault::TypeMismatch {
                    expected: "List of strings for sh.run arguments".to_string(),
                    actual: other.type_name().to_string(),
                });
            }
        }
    }

    let output = match command.output() {
        Ok(out) => out,
        Err(err) => {
            return Ok(Value::Failure(Rc::new(FailureValue::new(format!(
                "failed to execute command '{cmd_name}': {err}"
            )))));
        }
    };

    let exit_code = output.status.code().unwrap_or(-1) as i64;
    let stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
    let ok = output.status.success();

    let entries = vec![
        (
            Value::String(Rc::new("code".to_string())),
            Value::Int(exit_code),
        ),
        (
            Value::String(Rc::new("stdout".to_string())),
            Value::String(Rc::new(stdout_str)),
        ),
        (
            Value::String(Rc::new("stderr".to_string())),
            Value::String(Rc::new(stderr_str)),
        ),
        (Value::String(Rc::new("ok".to_string())), Value::Bool(ok)),
    ];

    Ok(Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(
        entries,
    )))))
}

/// Changes the current working directory: `sh.cd(path: String) -> Bool`.
pub fn sh_cd(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument for sh.cd".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let path = expect_string(&args[0], "sh.cd")?;
    match std::env::set_current_dir(path) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(err) => Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "failed to change directory to '{path}': {err}"
        ))))),
    }
}

/// Returns the current working directory: `sh.pwd() -> String`.
pub fn sh_pwd(args: &[Value]) -> Result<Value, VmFault> {
    if !args.is_empty() {
        return Err(VmFault::TypeMismatch {
            expected: "0 arguments for sh.pwd".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match std::env::current_dir() {
        Ok(dir) => Ok(Value::String(Rc::new(dir.to_string_lossy().into_owned()))),
        Err(err) => Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "failed to get current directory: {err}"
        ))))),
    }
}

/// Retrieves an environment variable: `sh.env(key: String) -> String?`.
pub fn sh_env(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument for sh.env".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let key = expect_string(&args[0], "sh.env")?;
    match std::env::var(key) {
        Ok(val) => Ok(Value::String(Rc::new(val))),
        Err(std::env::VarError::NotPresent) => Ok(Value::None),
        Err(err) => Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "failed to read environment variable '{key}': {err}"
        ))))),
    }
}

/// Looks up the path of an executable in PATH: `sh.which(cmd: String) -> String?`.
pub fn sh_which(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument for sh.which".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let cmd = expect_string(&args[0], "sh.which")?;
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(cmd);
        if full.is_file() {
            return Ok(Value::String(Rc::new(full.to_string_lossy().into_owned())));
        }
    }
    Ok(Value::None)
}

/// Constructs the canonical `sh` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let entries = vec![
        (
            Value::String(Rc::new("run".to_string())),
            Value::native("sh.run", usize::MAX, sh_run),
        ),
        (
            Value::String(Rc::new("cd".to_string())),
            Value::native("sh.cd", 1, sh_cd),
        ),
        (
            Value::String(Rc::new("pwd".to_string())),
            Value::native("sh.pwd", 0, sh_pwd),
        ),
        (
            Value::String(Rc::new("env".to_string())),
            Value::native("sh.env", 1, sh_env),
        ),
        (
            Value::String(Rc::new("which".to_string())),
            Value::native("sh.which", 1, sh_which),
        ),
    ];
    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
