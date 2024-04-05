use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, InstructionId, Intrinsic},
        types::Type,
        value::Value,
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn as_slice_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            let known_slice_lengths = known_slice_lengths(func);
            replace_known_slice_lengths(func, known_slice_lengths);
        }
        self
    }
}

fn known_slice_lengths(func: &Function) -> HashMap<InstructionId, usize> {
    let mut known_slice_lengths = HashMap::default();
    for block_id in func.reachable_blocks() {
        let block = &func.dfg[block_id];
        for instruction_id in block.instructions() {
            let (target_func, arguments) = match &func.dfg[*instruction_id] {
                Instruction::Call { func, arguments } => (func, arguments),
                _ => continue,
            };

            match &func.dfg[*target_func] {
                Value::Intrinsic(Intrinsic::AsSlice) => {
                    let array_typ = func.dfg.type_of_value(arguments[0]);
                    if let Type::Array(_, length) = array_typ {
                        known_slice_lengths.insert(*instruction_id, length);
                    } else {
                        unreachable!("AsSlice called with non-array {}", array_typ);
                    }
                }
                _ => continue,
            };
        }
    }
    known_slice_lengths
}

fn replace_known_slice_lengths(
    func: &mut Function,
    known_slice_lengths: HashMap<InstructionId, usize>,
) {
    known_slice_lengths.into_iter().for_each(|(instruction_id, known_length)| {
        let call_returns = func.dfg.instruction_results(instruction_id);
        let original_slice_length = call_returns[0];

        // We won't use the new id for the original unknown length.
        func.dfg.replace_result(instruction_id, original_slice_length);
        let known_length = func.dfg.make_constant(known_length.into(), Type::length_type());
        func.dfg.set_value_from_id(original_slice_length, known_length);
    });
}
