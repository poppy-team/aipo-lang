//! Failure recovery: handler dispatch and frame-unwinding propagation (Model B).
use super::Vm;
use super::task::{MAIN_TASK, TaskOutcome};
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
            // Unwinding past the last frame of a *driven* task ends the task with the
            // failure, exactly like `Return` ends it with a value. Falling through to
            // `frame.return_ip` (0 for a task's base frame) would restart the module
            // entry script inside the task, which is a live-lock, not recovery.
            if self.frames.is_empty() && self.current.is_some_and(|current| current != MAIN_TASK) {
                self.stack.clear();
                self.complete_current(TaskOutcome::Failed(failure));
                return Ok(());
            }
            self.push(failure)?;
            self.ip = frame.return_ip;
            Ok(())
        } else {
            // Nothing is left to handle it: the module entry script is the outermost path,
            // so the program ends here and `run` reports the failure.
            self.stack.clear();
            self.mutation_journal.clear();
            self.halted_with = Some(failure);
            Ok(())
        }
    }
}
