//! End-to-end pipeline tests for Wave 3 features (Set, Bytes packing, Duration, Sequence).

use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_syntax::parse;
use aipo_vm::{Value, Vm};

fn run_program(code: &str) -> Vm {
    let src = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);

    vm.run(&bytecode).expect("vm execution failed");
    vm
}

#[test]
fn test_end_to_end_set() {
    let vm = run_program(
        r#"
        var s = Set([10, 20, 10, 30])
        s.add(40)
        var removed = s.remove(20)
        var has10 = s.has(10)
        var has20 = s.has(20)
        var s_len = s.len()
        var l = s.to_list()
        "#,
    );

    assert_eq!(vm.get_global("removed"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("has10"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("has20"), Some(&Value::Bool(false)));
    assert_eq!(vm.get_global("s_len"), Some(&Value::Int(3)));

    if let Some(Value::List(l)) = vm.get_global("l") {
        assert_eq!(
            *l.borrow(),
            vec![Value::Int(10), Value::Int(30), Value::Int(40)]
        );
    } else {
        panic!("expected l to be List");
    }
}

#[test]
fn test_end_to_end_bytes_packing() {
    let vm = run_program(
        r#"
        var buf = Bytes(8)
        buf.write_i16(0, -500)
        buf.write_u16(2, 40000)
        var read_i = buf.read_i16(0)
        var read_u = buf.read_u16(2)
        var text = "Hello Aipo"
        var encoded = text.encode()
        var decoded = encoded.decode()
        "#,
    );

    assert_eq!(vm.get_global("read_i"), Some(&Value::Int(-500)));
    assert_eq!(vm.get_global("read_u"), Some(&Value::Int(40000)));
    if let Some(Value::String(s)) = vm.get_global("decoded") {
        assert_eq!(s.as_str(), "Hello Aipo");
    } else {
        panic!("expected decoded String");
    }
}

#[test]
fn test_end_to_end_duration() {
    let vm = run_program(
        r#"
        var d1 = Duration(2.5)
        var d2 = Duration(1.5)
        var d3 = d1 + d2
        var d_diff = d1 - d2
        var total = d3.total_seconds()
        var is_less = d2 < d1
        "#,
    );

    assert_eq!(vm.get_global("d3"), Some(&Value::Duration(4.0)));
    assert_eq!(vm.get_global("d_diff"), Some(&Value::Duration(1.0)));
    assert_eq!(vm.get_global("total"), Some(&Value::Float(4.0)));
    assert_eq!(vm.get_global("is_less"), Some(&Value::Bool(true)));
}

#[test]
fn test_end_to_end_lazy_sequence() {
    let vm = run_program(
        r#"
        var items = [1, 2, 3]
        var seq = items.lazy()
        "#,
    );

    assert!(matches!(vm.get_global("seq"), Some(Value::Sequence(_))));
}

#[test]
fn test_end_to_end_task_spawn_and_await() {
    let vm = run_program(
        r#"
        fn worker()
            return 42
        end

        var t = task.spawn(worker, [])
        var res = await t
        "#,
    );

    assert_eq!(vm.get_global("res"), Some(&Value::Int(42)));
}

#[test]
fn test_end_to_end_task_all_and_race() {
    let vm = run_program(
        r#"
        fn work1()
            return 10
        end
        fn work2()
            return 20
        end

        var t1 = task.spawn(work1, [])
        var t2 = task.spawn(work2, [])
        var all_res = task.all([t1, t2])
        var race_res = task.race([task.spawn(work1, []), task.spawn(work2, [])])
        "#,
    );

    if let Some(Value::List(l)) = vm.get_global("all_res") {
        assert_eq!(*l.borrow(), vec![Value::Int(10), Value::Int(20)]);
    } else {
        panic!("expected all_res to be List");
    }
    assert!(matches!(
        vm.get_global("race_res"),
        Some(Value::Int(10)) | Some(Value::Int(20))
    ));
}

#[test]
fn test_end_to_end_task_group() {
    let vm = run_program(
        r#"
        fn worker(val)
            return val * 2
        end

        var g = task.group()
        var t1 = g.spawn(worker, [5])
        var t2 = g.spawn(worker, [10])
        var group_res = g.wait()
        "#,
    );

    if let Some(Value::List(l)) = vm.get_global("group_res") {
        assert_eq!(*l.borrow(), vec![Value::Int(10), Value::Int(20)]);
    } else {
        panic!("expected group_res to be List");
    }
}

#[test]
fn test_end_to_end_task_sleep_and_timeout() {
    let vm = run_program(
        r#"
        fn worker()
            task.sleep(1)
            return 99
        end

        var t = task.spawn(worker, [])
        var timeout_res = task.timeout(t, 2)
        "#,
    );

    assert_eq!(vm.get_global("timeout_res"), Some(&Value::Int(99)));
}

#[test]
fn test_end_to_end_task_timeout_expires() {
    let vm = run_program(
        r#"
        fn worker()
            task.sleep(5)
            return 99
        end

        var t = task.spawn(worker, [])
        var timeout_res = task.timeout(t, 1) or_else "timed_out"
        "#,
    );

    assert_eq!(
        vm.get_global("timeout_res"),
        Some(&Value::String(std::rc::Rc::new("timed_out".to_string())))
    );
}

#[test]
fn test_end_to_end_task_cancel() {
    let _vm = run_program(
        r#"
        fn worker()
            task.sleep(10)
            return 1
        end

        var t = task.spawn(worker, [])
        task.cancel(t)
        "#,
    );
}
