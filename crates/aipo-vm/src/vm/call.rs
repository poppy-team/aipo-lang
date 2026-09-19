//! Call protocol: callee dispatch, frames, arity, natives, conversions,
//! bound methods. Failure short-circuiting follows Model B.
use super::Vm;
use crate::convert::convert_via_type;
use crate::fault::{VmError, VmFault};
use crate::frame::CallFrame;
use crate::value::{MethodKind, Value};
use aipo_bytecode::BytecodeModule;
use std::rc::Rc;
impl Vm {
    /// Resolves and starts a call whose callee and arguments are the top
    /// `arg_count + 1` operand stack entries.
    ///
    /// Frame-based callees (functions, closures, struct methods) push a call frame;
    /// natives, type conversions and native bound methods complete inline and leave
    /// their result on the stack.
    pub(super) fn begin_call(
        &mut self,
        module: &BytecodeModule,
        arg_count: usize,
    ) -> Result<(), VmError> {
        if self.stack.len() < arg_count + 1 {
            return Err(VmFault::StackUnderflow.into());
        }

        let callee_idx = self.stack.len() - 1 - arg_count;
        let callee = self.stack[callee_idx].clone();

        // Failure propagation (Model B):
        if callee.is_failure() {
            self.stack.truncate(callee_idx);
            self.push(callee)?;
            return Ok(());
        }
        for i in 0..arg_count {
            if self.stack[callee_idx + 1 + i].is_failure() {
                let failure = self.stack[callee_idx + 1 + i].clone();
                self.stack.truncate(callee_idx);
                self.push(failure)?;
                return Ok(());
            }
        }

        match callee {
            Value::Function { entry_ip, arity } => {
                self.check_arity(arg_count, arity)?;
                let journal_start = self.mutation_journal.len();
                self.frames.push(CallFrame::new(
                    self.ip,
                    callee_idx + 1,
                    arg_count,
                    journal_start,
                ));
                self.upvalue_frames.push(None);
                self.ip = entry_ip;
            }
            Value::Closure {
                entry_ip,
                arity,
                upvalues,
            } => {
                self.check_arity(arg_count, arity)?;
                let journal_start = self.mutation_journal.len();
                self.frames.push(CallFrame::new(
                    self.ip,
                    callee_idx + 1,
                    arg_count,
                    journal_start,
                ));
                self.upvalue_frames.push(Some(Rc::new(upvalues)));
                self.ip = entry_ip;
            }
            Value::Native { arity, func, .. } => {
                self.check_arity(arg_count, arity)?;
                let args = self.stack[callee_idx + 1..callee_idx + 1 + arg_count].to_vec();
                let result = func(&args)?;
                self.stack.truncate(callee_idx);
                self.push(result)?;
            }
            Value::Type(tag) => {
                let args = self.stack[callee_idx + 1..callee_idx + 1 + arg_count].to_vec();
                let result = convert_via_type(tag, &args)?;
                self.stack.truncate(callee_idx);
                self.push(result)?;
            }
            Value::BoundMethod {
                name,
                arity,
                receiver,
                kind,
            } => {
                self.check_arity(arg_count, arity)?;
                self.ensure_mutation_allowed(&receiver, &name)?;
                match kind {
                    MethodKind::Native(func) => {
                        let args = self.stack[callee_idx + 1..callee_idx + 1 + arg_count].to_vec();
                        let result = func(&receiver, &args)?;
                        self.stack.truncate(callee_idx);
                        self.push(result)?;
                    }
                    MethodKind::Function {
                        entry_ip,
                        total_arity,
                    } => {
                        if arg_count + 1 != total_arity {
                            return Err(VmFault::TypeMismatch {
                                expected: format!("{} arguments for {name}", total_arity - 1),
                                actual: format!("{arg_count} arguments"),
                            }
                            .into());
                        }
                        // The receiver takes the callee slot and becomes argument 0.
                        self.stack[callee_idx] = (*receiver).clone();
                        let journal_start = self.mutation_journal.len();
                        self.frames.push(CallFrame::method(
                            self.ip,
                            callee_idx,
                            arg_count + 1,
                            journal_start,
                        ));
                        self.upvalue_frames.push(None);
                        self.ip = entry_ip;
                    }
                    MethodKind::HigherOrder => {
                        let callable = self.stack[callee_idx + 1..callee_idx + 1 + arg_count]
                            .first()
                            .cloned()
                            .unwrap_or(Value::None);
                        self.stack.truncate(callee_idx);
                        let result = self.higher_order(module, &name, &receiver, &callable)?;
                        self.push(result)?;
                    }
                }
            }
            other => {
                return Err(VmFault::NotCallable {
                    type_name: other.type_name().to_string(),
                }
                .into());
            }
        }

        Ok(())
    }

    pub(super) fn check_arity(&self, provided: usize, expected: usize) -> Result<(), VmFault> {
        if provided == expected {
            Ok(())
        } else {
            Err(VmFault::TypeMismatch {
                expected: format!("{expected} arguments"),
                actual: format!("{provided} arguments"),
            })
        }
    }

    /// Calls an Aipo callable from host code (used by higher-order collection methods).
    ///
    /// Runs the interpreter until the invoked frame returns, so user functions and
    /// closures can be applied while the outer computation is still in progress.
    pub(super) fn invoke(
        &mut self,
        module: &BytecodeModule,
        callee: Value,
        args: &[Value],
    ) -> Result<Value, VmError> {
        let stack_base = self.stack.len();
        let frame_base = self.frames.len();
        let journal_base = self.mutation_journal.len();
        self.push(callee)?;
        for arg in args {
            self.push(arg.clone())?;
        }
        self.begin_call(module, args.len())?;
        while self.frames.len() > frame_base {
            if self.step(module)? {
                break;
            }
        }
        let result = self.pop()?;
        self.stack.truncate(stack_base);
        self.upvalue_frames.truncate(frame_base);
        self.mutation_journal.truncate(journal_base);
        Ok(result)
    }
}
