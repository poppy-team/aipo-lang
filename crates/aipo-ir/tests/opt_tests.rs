//! Differential tests for the Core IR optimization passes.
//!
//! Constant folding, peephole and dead-code elimination must be invisible to
//! the program: every corpus entry runs through the VM twice, once optimized and
//! once not, and both runs must agree on the printed output and the return value.

use aipo_ir::{optimize, CoreConstant, CoreFunction, CoreInst};
use aipo_testkit::pipeline::{lower_to_ir, run_capture};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Serializes VM runs against the shared capture sink.
///
/// `run_capture` writes to one process-wide sink, so two tests running in
/// parallel would interleave their output and compare mismatched transcripts.
fn run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Builds a `CoreFunction` from a raw instruction list, for unit-level checks.
fn function(name: &str, instructions: Vec<CoreInst>) -> CoreFunction {
    CoreFunction {
        name: name.to_string(),
        is_async: false,
        params: Vec::new(),
        locals: Vec::new(),
        upvalues: Vec::new(),
        instructions,
        span: aipo_source::SourceSpan::empty(0),
    }
}

/// Runs one program through the VM twice and asserts both runs agree.
fn assert_same_output(text: &str) {
    let _serialized = run_lock();

    let (_source, plain) = lower_to_ir("opt.aipo", text).expect("module lowers");

    let optimized = optimize(&plain);
    let module = emit(&plain);
    let optimized_module = emit(&optimized);

    let plain_run = run_capture(&module).expect("plain module runs");
    let optimized_run = run_capture(&optimized_module).expect("optimized module runs");

    assert_eq!(
        plain_run, optimized_run,
        "optimization changed behaviour for:\n{text}"
    );
}

/// Emits a Core module to bytecode, applying nothing.
fn emit(core: &aipo_ir::CoreModule) -> aipo_bytecode::BytecodeModule {
    aipo_bytecode::compile(core).expect("core module emits to bytecode")
}

// --- constant folding ---

/// `10 + 20` is a compile-time value, so it collapses to one constant.
#[test]
fn test_folds_integer_addition() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(10), span),
            CoreInst::Constant(CoreConstant::Int(20), span),
            CoreInst::Binary(aipo_ast::BinaryOp::Add, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    let instructions = &optimized.functions[0].instructions;
    assert_eq!(instructions.len(), 2, "{instructions:?}");
    assert!(matches!(
        &instructions[0],
        CoreInst::Constant(CoreConstant::Int(30), _)
    ));
}

/// String concatenation of two literals is also a compile-time value.
#[test]
fn test_folds_string_concatenation() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::String("ab".to_string()), span),
            CoreInst::Constant(CoreConstant::String("cd".to_string()), span),
            CoreInst::Binary(aipo_ast::BinaryOp::Add, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert!(matches!(
        &optimized.functions[0].instructions[0],
        CoreInst::Constant(CoreConstant::String(text), _) if text == "abcd"
    ));
}

/// Concatenation folds to NFC, matching the runtime's `Add`.
///
/// Canon makes NFC an invariant of `String`. If the folder joined the operands with
/// `format!` while the VM composed them, an optimized build would store `"e\u{0301}"`
/// where an unoptimized one stores `"é"` — a value change that also shifts `len`.
#[test]
fn test_folds_string_concatenation_to_nfc() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::String("e".to_string()), span),
            CoreInst::Constant(CoreConstant::String("\u{0301}".to_string()), span),
            CoreInst::Binary(aipo_ast::BinaryOp::Add, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert!(
        matches!(
            &optimized.functions[0].instructions[0],
            CoreInst::Constant(CoreConstant::String(text), _) if text == "\u{e9}"
        ),
        "the fold must compose 'e' + U+0301 into U+00E9, got {:?}",
        optimized.functions[0].instructions[0]
    );
}

/// A negation of a literal folds to the negated literal.
#[test]
fn test_folds_unary_negation() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(5), span),
            CoreInst::Unary(aipo_ast::UnaryOp::Neg, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert!(matches!(
        &optimized.functions[0].instructions[0],
        CoreInst::Constant(CoreConstant::Int(-5), _)
    ));
}

/// `not true` folds to `false`.
#[test]
fn test_folds_boolean_negation() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Bool(true), span),
            CoreInst::Unary(aipo_ast::UnaryOp::Not, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert!(matches!(
        &optimized.functions[0].instructions[0],
        CoreInst::Constant(CoreConstant::Bool(false), _)
    ));
}

/// Integer overflow is not folded: the runtime must still fault on it.
#[test]
fn test_does_not_fold_overflowing_addition() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(i64::MAX), span),
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Binary(aipo_ast::BinaryOp::Add, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    let instructions = &optimized.functions[0].instructions;
    // The two operands and the operator survive; nothing was folded away.
    assert!(
        matches!(instructions.get(2), Some(CoreInst::Binary(aipo_ast::BinaryOp::Add, _))),
        "overflow must not become a folded constant: {instructions:?}"
    );
    assert!(
        !matches!(instructions.first(), Some(CoreInst::Constant(CoreConstant::Int(n), _)) if *n != i64::MAX),
        "the folded value, if any, is not a new constant: {instructions:?}"
    );
}

/// Division is excluded from folding because its trap semantics belong to the VM.
#[test]
fn test_does_not_fold_division() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(10), span),
            CoreInst::Constant(CoreConstant::Int(2), span),
            CoreInst::Binary(aipo_ast::BinaryOp::IntDiv, span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert_eq!(optimized.functions[0].instructions.len(), 4);
}

// --- peephole ---

/// A `Jump` into the next instruction is a fall-through already.
#[test]
fn test_removes_jump_to_next_instruction() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Jump(1, span),
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert_eq!(optimized.functions[0].instructions.len(), 2);
}

/// A constant that is popped immediately never escapes the stack.
#[test]
fn test_removes_popped_constant() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(7), span),
            CoreInst::Pop(span),
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    assert_eq!(optimized.functions[0].instructions.len(), 2);
}

/// A branch on a known `Bool` resolves before execution: `false` always jumps.
#[test]
fn test_resolves_known_false_branch() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Bool(false), span),
            CoreInst::JumpIfFalse(3, span),
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Jump(4, span),
            CoreInst::Constant(CoreConstant::Int(2), span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    let instructions = &optimized.functions[0].instructions;
    // The known-false branch always jumps past `Constant(1)` and `Jump(4)` to
    // the `Constant(2)` arm, and dead-code elimination then removes the skipped
    // body. The surviving stream therefore pushes `2` and returns.
    assert_eq!(instructions.len(), 2, "{instructions:?}");
    assert!(
        matches!(instructions.first(), Some(CoreInst::Constant(CoreConstant::Int(2), _))),
        "the false branch resolved to the second arm: {instructions:?}"
    );
}

// --- dead code elimination ---

/// Instructions after an unconditional `Jump` can never run.
#[test]
fn test_removes_instructions_after_jump() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Jump(5, span),
            // Dead: only reachable by a jump, and none points here.
            CoreInst::Constant(CoreConstant::Int(99), span),
            CoreInst::Constant(CoreConstant::Int(98), span),
            CoreInst::Constant(CoreConstant::Int(97), span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    let instructions = &optimized.functions[0].instructions;
    assert_eq!(instructions.len(), 2, "{instructions:?}");
    assert!(
        matches!(instructions[1], CoreInst::Return { .. }),
        "the jump target still resolves to the return"
    );
}

/// A jump into removed code lands on the next surviving instruction, so a
/// removed body never swallows the target.
#[test]
fn test_jump_targets_survive_dead_code_removal() {
    let span = aipo_source::SourceSpan::empty(0);
    let function = function(
        "f",
        vec![
            CoreInst::Jump(6, span),
            // Dead body that a conditional branch jumps over.
            CoreInst::Constant(CoreConstant::Int(1), span),
            CoreInst::Constant(CoreConstant::Int(2), span),
            // Live branch: jumps forward to the return.
            CoreInst::Constant(CoreConstant::Bool(true), span),
            CoreInst::JumpIfFalse(6, span),
            CoreInst::Constant(CoreConstant::Int(42), span),
            CoreInst::Return {
                has_value: true,
                span,
            },
        ],
    );

    let optimized = optimize(&aipo_ir::CoreModule {
        functions: vec![function],
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    });

    let instructions = &optimized.functions[0].instructions;
    // Every remaining jump must land on a real instruction boundary.
    for inst in instructions {
        let target = match inst {
            CoreInst::Jump(target, _) | CoreInst::JumpIfFalse(target, _) => *target,
            _ => continue,
        };
        assert!(
            (0..=instructions.len() as isize).contains(&target),
            "jump target {target} is outside the optimized stream {instructions:?}"
        );
    }
}

// --- differential: the passes must be invisible ---

/// Arithmetic folds everywhere, and behaviour is identical either way.
#[test]
fn test_folding_preserves_program_behaviour() {
    assert_same_output(
        r#"
fn add(a, b)
  return a + b
end
io.println(String(add(2, 3) + add(10, 20)))
io.println(String(2 * 3 + 4 * 5))
"#,
    );
}

/// Loops with branches and jumps exercise the jump re-noting.
#[test]
fn test_control_flow_preserves_program_behaviour() {
    assert_same_output(
        r#"
fn classify(v)
  if v < 0
    return "negative"
  elif v == 0
    return "zero"
  else
    return "positive"
  end
end
each n in [-2, -1, 0, 1, 2]
  io.println(classify(n))
end
"#,
    );
}

/// `while` with `break` and `continue` is the densest jump shape in the IR.
#[test]
fn test_loops_preserve_program_behaviour() {
    assert_same_output(
        r#"
var total = 0
var i = 0
while i < 20
  i = i + 1
  if i % 3 == 0
    continue
  end
  if i > 15
    break
  end
  total = total + i
end
io.println(String(total))
"#,
    );
}

/// `return` inside a loop still returns from the function after dead code is cut.
#[test]
fn test_return_inside_loop_preserves_behaviour() {
    assert_same_output(
        r#"
fn find_first_even(items)
  each item in items
    if item % 2 == 0
      return item
    end
  end
  return -1
end
io.println(String(find_first_even([1, 3, 4, 5, 6])))
io.println(String(find_first_even([1, 3, 5])))
"#,
    );
}

/// `match` arms compile to jumps; the first matching arm must still win.
#[test]
fn test_match_preserves_program_behaviour() {
    assert_same_output(
        r#"
fn bucket(v)
  match v
    when 0, 1, 2
      io.println("low")
    when 3, 4
      io.println("mid")
    else
      io.println("high")
  end
end
each n in [0, 1, 2, 3, 4, 9]
  bucket(n)
end
"#,
    );
}

/// Failure handling installs and pops handlers; those are never removed.
#[test]
fn test_attempt_preserves_program_behaviour() {
    assert_same_output(
        r#"
fn risky(flag)
  if flag
    fail ("boom")
  end
  return 1
end

fn guarded(flag)
  attempt
    let value = risky(flag)
    io.println(String(value))
  failed err
    io.println("caught")
  end
end

guarded(false)
guarded(true)
"#,
    );
}

/// Struct construction and field access must survive the passes untouched.
#[test]
fn test_structs_preserve_program_behaviour() {
    assert_same_output(
        r#"
struct Point
  x
  y
end

impl Point
  fn sum(self)
    return self.x + self.y
  end
end

let p = Point{x: 3, y: 4}
io.println(String(p.sum()))
"#,
    );
}

/// Closures and higher-order calls are the shapes most likely to expose a bad
/// stack-balance assumption in a peephole rule.
#[test]
fn test_closures_preserve_program_behaviour() {
    assert_same_output(
        r#"
fn apply_twice(f, v)
  return f(f(v))
end
let inc = n => n + 1
io.println(String(apply_twice(inc, 5)))
"#,
    );
}

/// Literal-heavy arithmetic is exactly what folding rewrites.
#[test]
fn test_literal_heavy_program_is_unchanged() {
    assert_same_output(
        r#"
io.println(String(1 + 2 + 3 + 4 + 5))
io.println(String(100 * 100))
io.println(f"a" + "b" + "c")
"#,
    );
}

/// An empty module must survive the passes untouched.
#[test]
fn test_empty_module_is_stable() {
    let span = aipo_source::SourceSpan::empty(0);
    let module = aipo_ir::CoreModule {
        functions: Vec::new(),
        top_level: function_empty(),
        structs: Vec::new(),
        span,
    };
    assert_eq!(optimize(&module), module);
}

/// The passes must be idempotent: a second run changes nothing.
#[test]
fn test_optimization_is_idempotent() {
    let (_source, module) = lower_to_ir(
        "idempotent.aipo",
        "fn add(a, b)\n  return a + b\nend\nio.println(String(add(1, 2)))",
    )
    .expect("module lowers");

    let once = optimize(&module);
    let twice = optimize(&once);
    assert_eq!(once, twice);
}

/// Builds an empty function body used to pad modules in the unit tests.
fn function_empty() -> CoreFunction {
    CoreFunction {
        name: "__top_level__".to_string(),
        is_async: false,
        params: Vec::new(),
        locals: Vec::new(),
        upvalues: Vec::new(),
        instructions: Vec::new(),
        span: aipo_source::SourceSpan::empty(0),
    }
}
