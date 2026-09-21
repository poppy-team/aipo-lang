//! Method binding, mutation-during-iteration guards, and the higher-order
//! collection methods that call back into Aipo code.
use super::helpers::{collection_identity, compare_keys, iterable_items};
use super::{MUTATING_METHODS, Vm};
use crate::fault::{VmError, VmFault};
use crate::value::{MethodKind, Value};
use aipo_bytecode::BytecodeModule;
use std::cell::RefCell;
use std::rc::Rc;
impl Vm {
    /// Binds a method of `receiver` by name, if one exists.
    ///
    /// The bound value carries the *plain* method name: the mutation guard and the
    /// higher-order dispatch both key off canonical method names (`add`, `filter`, …),
    /// independent of the receiver type.
    pub(super) fn bind_method(&self, receiver: &Value, method: &str) -> Option<Value> {
        let type_name = receiver.type_name().to_string();
        // Sequence methods dispatch through the scheduler-aware path in
        // `begin_call` (pure stages append, driving stages evaluate).
        if matches!(receiver, Value::Sequence(_)) {
            if let Some((_, arity)) = super::task::SEQUENCE_METHODS
                .iter()
                .find(|(name, _)| *name == method)
            {
                return Some(Value::BoundMethod {
                    name: method.to_string(),
                    arity: *arity,
                    receiver: Box::new(receiver.clone()),
                    kind: MethodKind::HigherOrder,
                });
            }
            return None;
        }
        // Group methods (`spawn`, `wait`) run through the scheduler too.
        if matches!(receiver, Value::Group(_)) {
            let arity = match method {
                "spawn" => 2,
                "wait" => 0,
                _ => return None,
            };
            return Some(Value::BoundMethod {
                name: method.to_string(),
                arity,
                receiver: Box::new(receiver.clone()),
                kind: MethodKind::HigherOrder,
            });
        }
        if let Some((arity, func)) = self.method_natives.get(&(type_name, method.to_string())) {
            return Some(Value::BoundMethod {
                name: method.to_string(),
                arity: *arity,
                receiver: Box::new(receiver.clone()),
                kind: MethodKind::Native(*func),
            });
        }
        if matches!(method, "filter" | "transform" | "sort_by") {
            return Some(Value::BoundMethod {
                name: method.to_string(),
                arity: 1,
                receiver: Box::new(receiver.clone()),
                kind: MethodKind::HigherOrder,
            });
        }
        None
    }

    /// Rejects structural mutation of a collection that is currently being iterated.
    pub(super) fn ensure_mutation_allowed(
        &self,
        receiver: &Value,
        method: &str,
    ) -> Result<(), VmFault> {
        if !MUTATING_METHODS.contains(&method) {
            return Ok(());
        }
        match collection_identity(receiver) {
            Some(id) if self.active_iterations.contains(&id) => {
                Err(VmFault::MutationDuringIteration)
            }
            _ => Ok(()),
        }
    }

    /// Executes the canonical higher-order collection methods (`filter`, `transform`,
    /// `sort_by`) by applying the Aipo callable through [`Vm::invoke`].
    pub(super) fn higher_order(
        &mut self,
        module: &BytecodeModule,
        method: &str,
        receiver: &Value,
        callable: &Value,
    ) -> Result<Value, VmError> {
        let items = iterable_items(receiver)?;
        let guard = collection_identity(receiver);
        if let Some(id) = guard {
            self.active_iterations.push(id);
        }

        let outcome = match method {
            "filter" => {
                let mut kept = Vec::new();
                let mut error = None;
                for item in items {
                    match self.invoke(module, callable.clone(), std::slice::from_ref(&item)) {
                        Ok(Value::Bool(true)) => kept.push(item),
                        Ok(Value::Bool(false)) => {}
                        Ok(other) => {
                            error = Some(VmFault::TypeMismatch {
                                expected: "Bool predicate result".to_string(),
                                actual: other.type_name().to_string(),
                            });
                            break;
                        }
                        Err(err) => {
                            error = Some(VmFault::StackUnderflow);
                            if let VmError::Fault(fault) = err {
                                error = Some(fault);
                            }
                            break;
                        }
                    }
                }
                match error {
                    Some(fault) => Err(VmError::Fault(fault)),
                    None => Ok(Value::List(Rc::new(RefCell::new(kept)))),
                }
            }
            "transform" => {
                let mut mapped = Vec::new();
                let mut error = None;
                for item in items {
                    match self.invoke(module, callable.clone(), &[item]) {
                        Ok(value) => mapped.push(value),
                        Err(err) => {
                            error = Some(err);
                            break;
                        }
                    }
                }
                match error {
                    Some(err) => Err(err),
                    None => Ok(Value::List(Rc::new(RefCell::new(mapped)))),
                }
            }
            "sort_by" => {
                let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
                let mut error = None;
                for item in items {
                    match self.invoke(module, callable.clone(), std::slice::from_ref(&item)) {
                        Ok(key) => keyed.push((key, item)),
                        Err(err) => {
                            error = Some(err);
                            break;
                        }
                    }
                }
                if let Some(err) = error {
                    Err(err)
                } else {
                    let mut failed = false;
                    keyed.sort_by(|(a, _), (b, _)| match compare_keys(a, b) {
                        Some(ordering) => ordering,
                        None => {
                            failed = true;
                            std::cmp::Ordering::Equal
                        }
                    });
                    if failed {
                        Err(VmError::Fault(VmFault::TypeMismatch {
                            expected: "comparable sort keys".to_string(),
                            actual: "mixed non-comparable keys".to_string(),
                        }))
                    } else {
                        let sorted = keyed.into_iter().map(|(_, item)| item).collect();
                        Ok(Value::List(Rc::new(RefCell::new(sorted))))
                    }
                }
            }
            other => Err(VmError::Fault(VmFault::TypeMismatch {
                expected: "filter, transform, or sort_by".to_string(),
                actual: other.to_string(),
            })),
        };

        if guard.is_some() {
            self.active_iterations.pop();
        }
        // `filter`/`transform`/`sort_by` over a `Set` return a `Set`
        // (deduplicated, insertion order); over anything else a `List`.
        match (&receiver, outcome) {
            (Value::Set(_), Ok(Value::List(items))) => {
                let mut unique = Vec::new();
                for item in items.borrow().iter() {
                    if !unique.iter().any(|seen| seen == item) {
                        unique.push(item.clone());
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(unique))))
            }
            (_, outcome) => outcome,
        }
    }
}
