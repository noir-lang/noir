use std::collections::HashMap;

use crate::ssa::{ir::{function::Function, instruction::InstructionId, types::Type}, ssa_gen::Ssa};

impl Ssa {
    /// This pass removes `inc_rc` and `dec_rc` instructions
    /// as long as there are no `array_set` instructions to an array
    /// of the same type in between.
    ///
    /// Note that this pass is very conservative since the array_set
    /// instruction does not need to be to the same array. This is because
    /// the given array may alias another array (e.g. function parameters or
    /// a `load`ed array from a reference).
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_paired_rc(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            remove_paired_rc(function);
        }
        self
    }
}

#[derive(Default)]
struct Context {
    // All inc_rc instructions encountered without a corresponding dec_rc.
    // The type of the array being operated on is recorded.
    // If an array_set to that array type is encountered, that is also recorded.
    inc_rcs: HashMap<Type, InstructionId>,

    dec_rcs: HashMap<Type, InstructionId>,

    // When a dec_rc is encountered, the most recent inc_rc (of a matching array type)
    // is popped off the inc_rc_stack. If the IncDec object was not possibly mutated,
    // then the inc_rc and dec_rc instructions are both pushed here to be removed
    // from the program later.
    inc_decs_to_remove: Vec<(InstructionId, InstructionId)>,
}

fn remove_paired_rc(function: &mut Function) {
    // `dec_rc` is only issued for parameters currently so we can speed things
    // up a bit by skipping any functions without them.
    if !contains_array_parameter(function) {
        return;
    }

    let mut context = Context::default();

    // Iterate through each block in the function to find array_sets first.
    // We'll combine the results of each block later.
    for block in function.reachable_blocks() {

    }
}

fn contains_array_parameter(function: &mut Function) -> bool {
    function.parameters().iter().any(|parameter| {
        function.dfg.type_of_value(*parameter).contains_an_array()
    })
}
