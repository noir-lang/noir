use crate::ssa::{ir::function::Function, ssa_gen::Ssa};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_paired_rc(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            remove_paired_rc(function);
        }
        self
    }
}

fn remove_paired_rc(function: &mut Function) {}
