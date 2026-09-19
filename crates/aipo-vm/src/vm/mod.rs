//! Stack-based Virtual Machine execution engine for Aipo.

use crate::fault::{VmError, VmFault};
use crate::frame::{CallFrame, HandlerFrame};
use crate::value::{StructInstance, Value};
use aipo_bytecode::BytecodeModule;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod call;
mod contract;
mod dispatch;
mod failure;
mod helpers;
mod journal;
mod method;

/// Receiver-first native method implementation.
pub type MethodNative = fn(&Value, &[Value]) -> Result<Value, VmFault>;

/// Methods that structurally mutate their receiver, so the VM can enforce the
/// canonical `MutationDuringIteration` fault.
const MUTATING_METHODS: [&str; 6] = [
    "add",
    "insert",
    "remove",
    "remove_at",
    "remove_last",
    "clear",
];

/// Type alias for struct invariant validation functions.
pub type InvariantValidator = Box<dyn Fn(&StructInstance) -> Result<(), String>>;

/// One provisional, invariant-protected field assignment.
///
/// Canon makes a guarded update follow `candidate -> provisional apply -> verify -> commit`,
/// so the value the field held when the frame started mutating it is kept until a stable
/// mutable boundary verifies the instance.
struct MutationEntry {
    /// Instance whose direct field was assigned.
    instance: Rc<RefCell<StructInstance>>,
    /// Name of the assigned field.
    field: String,
    /// Value the field held when the frame first mutated it.
    previous: Value,
}

/// Captured upvalue cells of one call frame.
///
/// `None` marks a plain call; a closure call carries the cells it closed over, shared by
/// reference so that writes through an upvalue are visible to every holder.
pub type UpvalueFrame = Option<Rc<Vec<Rc<RefCell<Value>>>>>;

/// Virtual Machine executing Aipo bytecode.
pub struct Vm {
    /// Operand stack.
    pub stack: Vec<Value>,
    /// Call frames stack.
    pub frames: Vec<CallFrame>,
    /// Active recovery handler frames stack.
    pub handlers: Vec<HandlerFrame>,
    /// Global variable environment.
    pub globals: HashMap<String, Value>,
    /// Instruction pointer within current code buffer.
    pub ip: usize,
    /// Registered struct definitions: type name -> [(field_name, is_fixed)].
    pub struct_defs: HashMap<String, Vec<(String, bool)>>,
    /// Registered struct invariant validators: type name -> validator.
    pub struct_invariants: HashMap<String, InvariantValidator>,
    /// Compiled `<Type>.invariant` predicate entry points, resolved from the running module.
    ///
    /// The module records the predicate as a function named `Type.invariant`, so the runtime
    /// reaches it without a second expression evaluator.
    struct_invariant_entries: HashMap<String, usize>,
    /// Provisional field mutations of the active frames, oldest first.
    mutation_journal: Vec<MutationEntry>,
    /// Closures' captured cells, one entry per call frame (`None` for plain calls).
    pub upvalue_frames: Vec<UpvalueFrame>,
    /// Receiver-first native methods registered by the standard library.
    pub method_natives: HashMap<(String, String), (usize, MethodNative)>,
    /// User methods registered per struct type: (type, method) -> (entry ip, total arity).
    pub struct_methods: HashMap<(String, String), (usize, usize)>,
    /// Identities of collections currently under an active `each` iteration (stack order).
    active_iterations: Vec<usize>,
    /// Failure that ended the program because nothing was left to handle it.
    ///
    /// Canon makes recoverable `Failure` values propagate automatically; once propagation
    /// escapes the module entry script there is no code left to run, so execution stops and
    /// [`Vm::run`] reports it as an uncaught failure.
    halted_with: Option<Value>,
    /// Maximum allowed operand stack depth.
    pub max_stack_depth: usize,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    /// Constructs a new virtual machine instance with default limits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(128),
            frames: Vec::with_capacity(16),
            handlers: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
            struct_defs: HashMap::new(),
            struct_invariants: HashMap::new(),
            struct_invariant_entries: HashMap::new(),
            mutation_journal: Vec::new(),
            upvalue_frames: Vec::new(),
            method_natives: HashMap::new(),
            struct_methods: HashMap::new(),
            active_iterations: Vec::new(),
            halted_with: None,
            max_stack_depth: 1024,
        }
    }

    /// Registers a receiver-first native method for a type (for example `String.len`).
    pub fn register_method_native(
        &mut self,
        type_name: &str,
        method: &str,
        arity: usize,
        func: MethodNative,
    ) {
        self.method_natives
            .insert((type_name.to_string(), method.to_string()), (arity, func));
    }

    /// Registers a user-defined method of a struct type.
    pub fn register_struct_method(
        &mut self,
        type_name: &str,
        method: &str,
        entry_ip: usize,
        total_arity: usize,
    ) {
        self.struct_methods.insert(
            (type_name.to_string(), method.to_string()),
            (entry_ip, total_arity),
        );
    }

    /// Registers a struct type with its field names and `fixed` flags.
    pub fn register_struct(&mut self, type_name: impl Into<String>, fields: Vec<(&str, bool)>) {
        let field_defs = fields
            .into_iter()
            .map(|(name, fixed)| (name.to_string(), fixed))
            .collect();
        self.struct_defs.insert(type_name.into(), field_defs);
    }

    /// Registers an invariant validator for a struct type.
    pub fn register_struct_invariant<F>(&mut self, type_name: &str, validator: F)
    where
        F: Fn(&StructInstance) -> Result<(), String> + 'static,
    {
        self.struct_invariants
            .insert(type_name.to_string(), Box::new(validator));
    }

    /// Defines or updates a global variable.
    pub fn define_global(&mut self, name: impl Into<String>, value: Value) {
        self.globals.insert(name.into(), value);
    }

    /// Retrieves a global variable's value.
    #[must_use]
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    /// Pushes a value onto the operand stack.
    ///
    /// # Errors
    /// Returns `VmFault::StackUnderflow` / overflow if stack exceeds limit.
    pub fn push(&mut self, val: Value) -> Result<(), VmFault> {
        if self.stack.len() >= self.max_stack_depth {
            return Err(VmFault::Overflow {
                details: format!(
                    "operand stack exceeded depth limit {} ({} active call frames; deep or unbounded recursion is the usual cause)",
                    self.max_stack_depth,
                    self.frames.len()
                ),
            });
        }
        self.stack.push(val);
        Ok(())
    }

    /// Pops a value from the operand stack.
    ///
    /// # Errors
    /// Returns `VmFault::StackUnderflow` if the stack is empty.
    pub fn pop(&mut self) -> Result<Value, VmFault> {
        self.stack.pop().ok_or(VmFault::StackUnderflow)
    }

    /// Peeks at the top value of the operand stack.
    ///
    /// # Errors
    /// Returns `VmFault::StackUnderflow` if the stack is empty.
    pub fn peek(&self) -> Result<&Value, VmFault> {
        self.stack.last().ok_or(VmFault::StackUnderflow)
    }

    /// Runs a bytecode module from instruction 0 until completion.
    ///
    /// # Errors
    /// Returns `VmError` if a runtime fault occurs or an uncaught failure reaches top level.
    pub fn run(&mut self, module: &BytecodeModule) -> Result<Value, VmError> {
        self.ip = 0;
        self.stack.clear();
        self.frames.clear();
        self.handlers.clear();
        self.upvalue_frames.clear();
        self.active_iterations.clear();
        self.halted_with = None;
        self.mutation_journal.clear();

        // Struct invariants are compiled as `Type.invariant` predicates, so the module itself
        // records how to reach each type's check: the runtime resolves the entry points once
        // per run and never needs a second expression evaluator.
        self.struct_invariant_entries.clear();
        for function in &module.functions {
            if let Some((type_name, "invariant")) = function.name.split_once('.') {
                self.struct_invariant_entries
                    .insert(type_name.to_string(), function.entry_ip);
            }
        }

        while self.ip < module.code.len() {
            let halted = self.step(module)?;
            if halted {
                break;
            }
        }

        let result = match self.halted_with.take() {
            Some(failure) => failure,
            None => self.stack.pop().unwrap_or(Value::None),
        };
        if let Value::Failure(err) = &result {
            return Err(VmError::UncaughtFailure(err.message.clone()));
        }

        Ok(result)
    }
}
