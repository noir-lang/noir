use crate::ssa::{
    ir::{
        function::{Function, RuntimeType},
        instruction::{Instruction, Intrinsic},
        types::Type,
        value::Value,
    },
    ssa_gen::Ssa,
};
use acvm::FieldElement;
use fxhash::FxHashSet as HashSet;

impl Ssa {
    /// An SSA pass to find any calls to `Intrinsic::IsUnconstrained` and replacing any uses of the result of the intrinsic
    /// with the resolved boolean value.
    /// Note that this pass must run after the pass that does runtime separation, since in SSA generation an ACIR function can end up targeting brillig.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn resolve_is_unconstrained(mut self) -> Self {
        for func in self.functions.values_mut() {
            replace_is_unconstrained_result(func);
        }
        self
    }
}

fn replace_is_unconstrained_result(func: &mut Function) {
    let mut is_unconstrained_calls = HashSet::default();
    // Collect all calls to is_unconstrained
    for block_id in func.reachable_blocks() {
        for &instruction_id in func.dfg[block_id].instructions() {
            let target_func = match &func.dfg[instruction_id] {
                Instruction::Call { func, .. } => *func,
                _ => continue,
            };

            if let Value::Intrinsic(Intrinsic::IsUnconstrained) = &func.dfg[target_func] {
                is_unconstrained_calls.insert(instruction_id);
            }
        }
    }

    for instruction_id in is_unconstrained_calls {
        let call_returns = func.dfg.instruction_results(instruction_id);
        let original_return_id = call_returns[0];

        // We replace the result with a fresh id. This will be unused, so the DIE pass will remove the leftover intrinsic call.
        func.dfg.replace_result(instruction_id, original_return_id);

        let is_within_unconstrained = func.dfg.make_constant(
            FieldElement::from(matches!(func.runtime(), RuntimeType::Brillig)),
            Type::bool(),
        );
        // Replace all uses of the original return value with the constant
        func.dfg.set_value_from_id(original_return_id, is_within_unconstrained);
    }
}
