//! Failure recovery: handler dispatch and frame-unwinding propagation (Model B).
use super::Vm;
use crate::fault::VmError;
use crate::value::Value;
impl Vm {
    pub(super) fn handle_failure(&mut self, failure: Value) -> Result<(), VmError> {
        if let Some(handler) = self.handlers.pop() {
            self.frames.truncate(handler.frame_depth);
            self.upvalue_frames.truncate(handler.frame_depth);
            self.stack.truncate(handler.stack_depth);
            let journal_base = self.frames.last().map_or(0, |frame| frame.journal_start);
            self.mutation_journal.truncate(journal_base);
            self.push(failure)?;
            self.ip = handler.handler_ip;
            Ok(())
        } else if let Some(frame) = self.frames.pop() {
            // Propagate through call frames according to Model B
            self.upvalue_frames.pop();
            self.stack.truncate(frame.result_base());
            self.mutation_journal.truncate(frame.journal_start);
            self.push(failure)?;
            self.ip = frame.return_ip;
            Ok(())
        } else {
            // Nothing is left to handle it: the module entry script is the outermost path,
            // so the program ends here and `run` reports the failure.
            self.stack.clear();
            self.halted_with = Some(failure);
            Ok(())
        }
    }
}
