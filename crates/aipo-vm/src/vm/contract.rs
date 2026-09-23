//! Signature contracts and structural interface conformance.
use super::Vm;
use crate::convert::TypeTag;
use crate::value::Value;
impl Vm {
    /// Returns the number of arguments `name` takes on `value`, when the value exposes it.
    ///
    /// Mirrors the dot-call resolution (`GetField` binds a struct's associated function, then the
    /// native and higher-order tables answer the rest), so an interface contract asks exactly the
    /// question a call would answer.
    pub(super) fn operation_arity(&self, value: &Value, name: &str) -> Option<usize> {
        if let Value::Struct(instance) = value {
            let type_name = instance.borrow().type_name.clone();
            if let Some((_, total_arity, _)) =
                self.struct_methods.get(&(type_name, name.to_string()))
            {
                return Some(total_arity.saturating_sub(1));
            }
        }
        match self.bind_method(value, name) {
            Some(Value::BoundMethod(bm)) => Some(bm.arity),
            _ => None,
        }
    }

    /// Returns whether `value` satisfies a written signature contract.
    ///
    /// `T?` also accepts `none`. A core-type name is checked against its [`TypeTag`]; a name the
    /// running module declares as a struct is checked against the instance's type. A name the
    /// MVP cannot resolve at runtime (an interface) is accepted, because inventing a failure for
    /// a contract the runtime cannot evaluate would be worse than the missing check.
    pub(super) fn contract_holds(&self, type_name: &str, nullable: bool, value: &Value) -> bool {
        if matches!(value, Value::None) {
            return nullable;
        }
        if type_name == "Function" {
            return matches!(
                value,
                Value::Function { .. }
                    | Value::Closure(_)
                    | Value::Native { .. }
                    | Value::BoundMethod(_)
            );
        }
        if let Some(tag) = TypeTag::from_name(type_name) {
            return tag.matches(value);
        }
        if self.struct_defs.contains_key(type_name) {
            return match value {
                Value::Struct(instance) => instance.borrow().type_name == type_name,
                _ => false,
            };
        }
        true
    }
}
