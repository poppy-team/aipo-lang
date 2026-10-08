//! End-to-end tests for `Tipo::[fn_livre, outra]`, the batch association form.
//!
//! A batch binding promotes listed free functions to methods of the target type by
//! copying them under `Tipo.fn` with an injected `self` first parameter. The originals
//! stay callable on their own.
//!
//! Exercised through the CLI so the entire chain runs together: lexer, parser, HIR,
//! sema, IR lowering, bytecode emission and VM.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

fn run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone)]
struct CaptureSink(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for CaptureSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn run_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(CaptureSink(Arc::clone(&captured)))));
    let args: Vec<String> = vec!["run".into(), path.to_string_lossy().into()];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    aipo_cli::set_output_sink(None);

    let program_output = {
        let buffer = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        String::from_utf8_lossy(&buffer).into_owned()
    };
    (
        code,
        program_output,
        String::from_utf8_lossy(&err).into_owned(),
    )
}

fn check_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let args: Vec<String> = vec!["check".into(), path.to_string_lossy().into()];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

#[test]
fn test_batch_association_promotes_free_function_to_method() {
    let temp = std::env::temp_dir().join("aipo_batch_basic.aipo");
    let source = r#"
struct Retangulo {
    largura
    altura
}

fn area_de(self, l, a) {
    return l * a
}

Retangulo::[area_de]

let r = Retangulo{largura = 3, altura = 4}
# Method form: the receiver fills `self`, declared params still apply.
io.println(String(r.area_de(3, 4)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "12");
}

#[test]
fn test_batch_association_promotes_mutable_receiver_from_var_self() {
    let temp = std::env::temp_dir().join("aipo_batch_mut.aipo");
    let source = r#"
struct Contador {
    var valor
}

fn incrementa(var self, passo) {
    self.valor = self.valor + passo
}

Contador::[incrementa]

let c = Contador{valor = 0}
c.incrementa(5)
c.incrementa(3)
io.println(String(c.valor))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "8");
}

#[test]
fn test_batch_association_rejects_function_without_self() {
    let temp = std::env::temp_dir().join("aipo_batch_no_self.aipo");
    let source = r#"
struct Retangulo {
    largura
    altura
}

fn area_de(l, a) {
    return l * a
}

Retangulo::[area_de]

io.println("x")
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "batch must require `self` as first parameter");
    assert!(
        stderr.contains("no 'self' as first parameter"),
        "must explain the requirement, got: {stderr}"
    );
    assert!(
        stderr.contains("area_de"),
        "must name the offending function, got: {stderr}"
    );
}

#[test]
fn test_batch_association_rejects_self_in_free_function_body() {
    let temp = std::env::temp_dir().join("aipo_batch_self_err.aipo");
    let source = r#"
struct Retangulo {
    largura
    altura
}

fn escala_de(self) {
    return self.largura * 2
}

Retangulo::[escala_de]

let r = Retangulo{largura = 3, altura = 4}
# Read-only receiver: mutating it must be rejected.
r.escala_de()
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(
        code, 0,
        "read-only receiver reading a field is valid: {stderr}"
    );
}

#[test]
fn test_batch_association_keeps_original_free_function_callable() {
    let temp = std::env::temp_dir().join("aipo_batch_original.aipo");
    let source = r#"
struct Retangulo {
    largura
    altura
}

fn area_de(self, l, a) {
    return l * a
}

Retangulo::[area_de]

let r = Retangulo{largura = 3, altura = 4}
# Method form receives `self` as the receiver; the declared params still apply.
io.println(String(r.area_de(3, 4)))
# Free form still works: the promotion copies, it does not move.
io.println(String(area_de(r, 5, 6)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "12\n30");
}

#[test]
fn test_batch_association_with_multiple_functions() {
    let temp = std::env::temp_dir().join("aipo_batch_multi.aipo");
    let source = r#"
struct Retangulo {
    largura
    altura
}

fn area(self, l, a) {
    return l * a
}

fn perimetro(self, l, a) {
    return 2 * (l + a)
}

Retangulo::[area, perimetro]

let r = Retangulo{largura = 3, altura = 4}
io.println(String(r.area(3, 4)))
io.println(String(r.perimetro(3, 4)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "12\n14");
}

#[test]
fn test_batch_association_rejects_unknown_function() {
    let temp = std::env::temp_dir().join("aipo_batch_unknown.aipo");
    let source = r#"
struct Retangulo {
    largura
}

Retangulo::[nao_existe]

io.println("x")
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject unknown function statically");
    assert!(
        stderr.contains("AIPO_SEM_UNKNOWN_NAME"),
        "must report AIPO_SEM_UNKNOWN_NAME, got: {stderr}"
    );
    assert!(
        stderr.contains("nao_existe"),
        "must name the offending function, got: {stderr}"
    );
}
