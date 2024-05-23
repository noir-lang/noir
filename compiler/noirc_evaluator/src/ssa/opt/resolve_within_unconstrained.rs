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
    /// A simple SSA pass to find any calls to `Intrinsic::WithinUnconstrained` and replacing any references to the result of the intrinsic
    /// with the correct boolean value.
    /// Note that this pass must run after the pass that does runtime separation, since in SSA generation an ACIR function can end up targeting brillig.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn resolve_within_unconstrained(mut self) -> Self {
        for func in self.functions.values_mut() {
            replace_within_unconstrained(func);
        }
        self
    }
}

fn replace_within_unconstrained(func: &mut Function) {
    let mut within_unconstrained_calls = HashSet::default();
    for block_id in func.reachable_blocks() {
        for &instruction_id in func.dfg[block_id].instructions() {
            let target_func = match &func.dfg[instruction_id] {
                Instruction::Call { func, .. } => *func,
                _ => continue,
            };

            match &func.dfg[target_func] {
                Value::Intrinsic(Intrinsic::WithinUnconstrained) => {
                    within_unconstrained_calls.insert(instruction_id);
                }
                _ => continue,
            };
        }
    }

    for instruction_id in within_unconstrained_calls {
        let call_returns = func.dfg.instruction_results(instruction_id);
        let original_return_id = call_returns[0];

        func.dfg.replace_result(instruction_id, original_return_id);
        let known_value = func.dfg.make_constant(
            FieldElement::from(matches!(func.runtime(), RuntimeType::Brillig)),
            Type::bool(),
        );
        func.dfg.set_value_from_id(original_return_id, known_value);
    }
}
