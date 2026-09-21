//! Invariant journal: provisional field mutations, stable-boundary commit
//! with rollback, and guarded-type detection.
use super::{MutationEntry, Vm};
use crate::fault::VmError;
use crate::value::{FailureValue, StructInstance, Value};
use aipo_bytecode::BytecodeModule;
use std::cell::RefCell;
use std::rc::Rc;
impl Vm {
    /// Returns whether `type_name` declares an `invariant()` hook.
    pub(super) fn type_is_guarded(&self, type_name: &str) -> bool {
        self.struct_invariant_entries.contains_key(type_name)
            || self.struct_invariants.contains_key(type_name)
    }

    /// Journals one provisional assignment of a guarded field.
    ///
    /// Only the first write to a field within the frame is recorded, so a rollback restores the
    /// value the field held when the mutable operation began.
    pub(super) fn journal_mutation(
        &mut self,
        instance: Rc<RefCell<StructInstance>>,
        field: String,
        previous: Value,
    ) {
        let base = self.frames.last().map_or(0, |frame| frame.journal_start);
        let already = self.mutation_journal[base..]
            .iter()
            .any(|entry| Rc::ptr_eq(&entry.instance, &instance) && entry.field == field);
        if !already {
            self.mutation_journal.push(MutationEntry {
                instance,
                field,
                previous,
            });
        }
    }

    /// Verifies the frame's provisional mutations and publishes or rolls them back.
    ///
    /// Canon validates `invariant()` at a stable mutable boundary: every participating instance
    /// is checked, and if any check fails the direct fields of all of them return to their entry
    /// values and the operation produces a recoverable `Failure`.
    pub(super) fn commit_mutations(
        &mut self,
        module: &BytecodeModule,
        base: usize,
    ) -> Result<(), VmError> {
        if self.mutation_journal.len() <= base {
            return Ok(());
        }

        let mut instances: Vec<Rc<RefCell<StructInstance>>> = Vec::new();
        for entry in &self.mutation_journal[base..] {
            if !instances
                .iter()
                .any(|seen| Rc::ptr_eq(seen, &entry.instance))
            {
                instances.push(Rc::clone(&entry.instance));
            }
        }

        let mut violation: Option<String> = None;
        for instance in &instances {
            let type_name = instance.borrow().type_name.clone();
            let outcome = match self.struct_invariant_entries.get(&type_name).copied() {
                Some(entry_ip) => {
                    let callee = Value::Function {
                        entry_ip,
                        arity: 1,
                        is_async: false,
                    };
                    let receiver = Value::Struct(Rc::clone(instance));
                    match self.invoke(module, callee, &[receiver])? {
                        Value::Bool(true) => Ok(()),
                        _ => Err(format!(
                            "invariant() of {type_name} did not hold after mutation"
                        )),
                    }
                }
                None => match self.struct_invariants.get(&type_name) {
                    Some(validator) => validator(&instance.borrow()),
                    None => Ok(()),
                },
            };
            if let Err(message) = outcome {
                violation = Some(message);
                break;
            }
        }

        match violation {
            None => {
                self.mutation_journal.truncate(base);
                Ok(())
            }
            Some(message) => {
                // Canon returns the direct fields of every participating instance to the state
                // the operation started from before the failure propagates.
                for entry in self.mutation_journal[base..].iter().rev() {
                    entry
                        .instance
                        .borrow_mut()
                        .restore_field(&entry.field, entry.previous.clone());
                }
                self.mutation_journal.truncate(base);
                self.handle_failure(Value::Failure(Rc::new(FailureValue { message })))?;
                Ok(())
            }
        }
    }
}
