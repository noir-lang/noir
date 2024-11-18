use std::collections::HashSet;

use acvm::{blackbox_solver::StubbedBlackBoxSolver, brillig_vm::VMStatus};

use crate::{
    brillig::{brillig_ir::artifact::Label, Brillig},
    ssa::{
        ir::{
            function::{Function, FunctionId},
            instruction::{Instruction, InstructionId},
            value::{Value, ValueId},
        },
        Ssa,
    },
};

impl Ssa {
    pub(crate) fn inline_const_brillig_calls(mut self, brillig: &Brillig) -> Self {
        // Keep track of which brillig functions we couldn't completely inline: we'll remove the ones we could.
        let mut brillig_functions_we_could_not_inline = HashSet::new();

        for func in self.functions.values_mut() {
            func.inline_const_brillig_calls(&brillig, &mut brillig_functions_we_could_not_inline);
        }

        self
    }
}

/// Result of trying to optimize an instruction (any instruction) in this pass.
enum OptimizeResult {
    /// Nothing was done because the instruction wasn't a call to a brillig function,
    /// or some arguments to it were not constants.
    NotABrilligCall,
    /// The instruction was a call to a brillig function, but we couldn't optimize it.
    CannotOptimize(FunctionId),
    /// The instruction was a call to a brillig function and we were able to optimize it,
    /// returning the optimized function and the constant values it returned.
    Optimized(Function, Vec<ValueId>),
}

impl Function {
    pub(crate) fn inline_const_brillig_calls(
        &mut self,
        brillig: &Brillig,
        brillig_functions_we_could_not_inline: &mut HashSet<FunctionId>,
    ) {
        for block_id in self.reachable_blocks() {
            for instruction_id in self.dfg[block_id].take_instructions() {
                let optimize_result = self.optimize_const_brillig_call(
                    instruction_id,
                    brillig,
                    brillig_functions_we_could_not_inline,
                );
                match optimize_result {
                    OptimizeResult::NotABrilligCall => {
                        self.dfg[block_id].instructions_mut().push(instruction_id);
                    }
                    OptimizeResult::CannotOptimize(func_id) => {
                        self.dfg[block_id].instructions_mut().push(instruction_id);
                        brillig_functions_we_could_not_inline.insert(func_id);
                    }
                    OptimizeResult::Optimized(function, return_values) => {
                        // Replace the instruction results with the constant values we got
                        // let current_results = self.dfg.instruction_results(instruction_id).to_vec();
                        // assert_eq!(return_values.len(), current_results.len());

                        // for (current_result_id, return_value_id) in
                        //     current_results.iter().zip(return_values)
                        // {
                        //     let new_return_value_id =
                        //         function.copy_constant_to_function(return_value_id, self);
                        //     self.dfg.set_value_from_id(*current_result_id, new_return_value_id);
                        // }
                    }
                }
            }
        }
    }

    /// Tries to optimize an instruction if it's a call that points to a brillig function,
    /// and all its arguments are constant.
    fn optimize_const_brillig_call(
        &mut self,
        instruction_id: InstructionId,
        brillig: &Brillig,
        brillig_functions_we_could_not_inline: &mut HashSet<FunctionId>,
    ) -> OptimizeResult {
        let instruction = &self.dfg[instruction_id];
        let Instruction::Call { func: func_id, arguments } = instruction else {
            return OptimizeResult::NotABrilligCall;
        };

        let func_value = &self.dfg[*func_id];
        let Value::Function(func_id) = func_value else {
            return OptimizeResult::NotABrilligCall;
        };
        let func_id = *func_id;
        dbg!(func_id);

        let Some(brillig_artifact) = brillig.find_by_label(Label::function(func_id)) else {
            return OptimizeResult::NotABrilligCall;
        };

        if !arguments.iter().all(|argument| self.dfg.is_constant(*argument)) {
            return OptimizeResult::CannotOptimize(func_id);
        }

        // TODO...

        OptimizeResult::CannotOptimize(func_id)
    }
}
