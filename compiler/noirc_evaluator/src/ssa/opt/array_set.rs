use std::mem;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, RuntimeType},
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        types::Type::{Array, Slice},
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            #[cfg(debug_assertions)]
            array_set_optimization_pre_check(func);

            func.array_set_optimization();

            #[cfg(debug_assertions)]
            array_set_optimization_post_check(func);
        }
        self
    }
}

/// Pre-check condition for [Function::array_set_optimization].
///
/// Panics if:
///   - there already exists a mutable array set instruction.
#[cfg(debug_assertions)]
fn array_set_optimization_pre_check(func: &Function) {
    // There should be no mutable array sets.
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if matches!(func.dfg[*instruction_id], Instruction::ArraySet { mutable: true, .. }) {
                panic!("mutable ArraySet instruction exists before `array_set_optimization` pass");
            }
        }
    }
}

/// Post-check condition for [Function::array_set_optimization].
///
/// Panics if:
///   - Mutable array_set optimization has been applied to Brillig function
#[cfg(debug_assertions)]
fn array_set_optimization_post_check(func: &Function) {
    // Brillig functions should be not have any mutable array sets.
    if func.runtime().is_brillig() {
        for block_id in func.reachable_blocks() {
            let instruction_ids = func.dfg[block_id].instructions();
            for instruction_id in instruction_ids {
                if matches!(func.dfg[*instruction_id], Instruction::ArraySet { mutable: true, .. })
                {
                    panic!("Mutable array set instruction in Brillig function");
                }
            }
        }
    }
}

impl Function {
    pub(crate) fn array_set_optimization(&mut self) {
        if matches!(self.runtime(), RuntimeType::Brillig(_)) {
            // Brillig is supposed to use refcounting to decide whether to mutate an array;
            // array mutation was only meant for ACIR. We could use it with Brillig as well,
            // but then some of the optimizations that we can do in ACIR around shared
            // references have to be skipped, which makes it more cumbersome.
            return;
        }

        let reachable_blocks = self.reachable_blocks();

        if !self.runtime().is_entry_point() {
            assert_eq!(
                reachable_blocks.len(),
                1,
                "Expected there to be 1 block remaining in Acir function for array_set optimization"
            );
        }

        let mut context = Context::new(&self.dfg);

        for block in reachable_blocks.iter() {
            context.analyze_last_uses(*block);
        }

        let instructions_to_update = mem::take(&mut context.instructions_that_can_be_made_mutable);
        make_mutable(&mut self.dfg, &instructions_to_update);
    }
}

struct Context<'f> {
    dfg: &'f DataFlowGraph,
    array_to_last_use: HashMap<ValueId, InstructionId>,
    instructions_that_can_be_made_mutable: HashSet<InstructionId>,
    // Mapping of an array that comes from a load and whether the address
    // it was loaded from is a reference parameter passed to the block.
    arrays_from_load: HashMap<ValueId, bool>,
}

impl<'f> Context<'f> {
    fn new(dfg: &'f DataFlowGraph) -> Self {
        Context {
            dfg,
            array_to_last_use: HashMap::default(),
            instructions_that_can_be_made_mutable: HashSet::default(),
            arrays_from_load: HashMap::default(),
        }
    }

    /// Builds the set of ArraySet instructions that can be made mutable
    /// because their input value is unused elsewhere afterward.
    fn analyze_last_uses(&mut self, block_id: BasicBlockId) {
        let block = &self.dfg[block_id];

        for instruction_id in block.instructions() {
            match &self.dfg[*instruction_id] {
                Instruction::ArrayGet { array, .. } => {
                    let array = *array;

                    if let Some(existing) = self.array_to_last_use.insert(array, *instruction_id) {
                        self.instructions_that_can_be_made_mutable.remove(&existing);
                    }
                }
                Instruction::ArraySet { array, .. } => {
                    let array = *array;

                    if let Some(existing) = self.array_to_last_use.insert(array, *instruction_id) {
                        self.instructions_that_can_be_made_mutable.remove(&existing);
                    }

                    // If the array we are setting does not come from a load we can safely mark it mutable.
                    // If the array comes from a load we may potentially being mutating an array at a reference
                    // that is loaded from by other values.
                    let terminator = self.dfg[block_id].unwrap_terminator();

                    // If we are in a return block we are not concerned about the array potentially being mutated again.
                    let is_return_block =
                        matches!(terminator, TerminatorInstruction::Return { .. });

                    // We also want to check that the array is not part of the terminator arguments, as this means it is used again.
                    let mut is_array_in_terminator = false;
                    terminator.for_each_value(|value| {
                        // The terminator can contain original IDs, while the SSA has replaced the array value IDs; we need to resolve to compare.
                        if !is_array_in_terminator && value == array {
                            is_array_in_terminator = true;
                        }
                    });

                    let can_mutate = if let Some(is_from_param) = self.arrays_from_load.get(&array)
                    {
                        // If the array was loaded from a reference parameter, we cannot
                        // safely mark that array mutable as it may be shared by another value.
                        !is_from_param && is_return_block
                    } else {
                        !is_array_in_terminator
                    };

                    if can_mutate {
                        self.instructions_that_can_be_made_mutable.insert(*instruction_id);
                    }
                }
                Instruction::Call { arguments, .. } => {
                    for argument in arguments {
                        if matches!(self.dfg.type_of_value(*argument), Array { .. } | Slice { .. })
                        {
                            let argument = *argument;

                            if let Some(existing) =
                                self.array_to_last_use.insert(argument, *instruction_id)
                            {
                                self.instructions_that_can_be_made_mutable.remove(&existing);
                            }
                        }
                    }
                }
                Instruction::Load { address } => {
                    let result = self.dfg.instruction_results(*instruction_id)[0];
                    if matches!(self.dfg.type_of_value(result), Array { .. } | Slice { .. }) {
                        let is_reference_param =
                            self.dfg.block_parameters(block_id).contains(address);
                        self.arrays_from_load.insert(result, is_reference_param);
                    }
                }
                Instruction::MakeArray { elements, .. } => {
                    for element in elements {
                        if let Some(existing) =
                            self.array_to_last_use.insert(*element, *instruction_id)
                        {
                            self.instructions_that_can_be_made_mutable.remove(&existing);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

/// Make each ArraySet instruction in `instructions_to_update` mutable.
fn make_mutable(dfg: &mut DataFlowGraph, instructions_to_update: &HashSet<InstructionId>) {
    for instruction_id in instructions_to_update {
        let instruction = &mut dfg[*instruction_id];
        if let Instruction::ArraySet { mutable, .. } = instruction {
            *mutable = true;
        } else {
            unreachable!("Non-ArraySet instruction in instructions_to_update!\n{instruction:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{Ssa, opt::assert_normalized_ssa_equals},
    };

    #[test]
    fn array_set_in_loop_with_conditional_clone() {
        // We want to make sure that we do not mark a single array set mutable which is loaded
        // from and cloned in a loop. If the array is inadvertently marked mutable, and is cloned in a previous iteration
        // of the loop, its clone will also be altered.
        let src = "
            brillig(inline) fn main f0 {
              b0():
                v2 = make_array [Field 0, Field 0, Field 0, Field 0, Field 0] : [Field; 5]
                v3 = make_array [v2, v2] : [[Field; 5]; 2]
                v4 = allocate -> &mut [Field; 5]
                store v3 at v4
                v5 = allocate -> &mut [Field; 5]
                store v3 at v5
                jmp b1(u32 0)
              b1(v0: u32):
                v8 = lt v0, u32 5
                jmpif v8 then: b3, else: b2
              b2():
                return
              b3():
                v9 = eq v0, u32 5
                jmpif v9 then: b4, else: b5
              b4():
                v10 = load v4 -> [[Field; 5]; 2]
                store v10 at v5
                jmp b5()
              b5():
                v11 = load v4 -> [[Field; 5]; 2]
                v12 = array_get v11, index u32 0 -> [Field; 5]
                v14 = array_set v12, index v0, value Field 20
                v15 = array_set v11, index v0, value v14
                store v15 at v4
                v17 = add v0, u32 1
                jmp b1(v17)
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        // We expect the same result as above
        let ssa = ssa.array_set_optimization();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_mutate_array_used_in_make_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/8563
        // Previously `v2` would be marked as mutable in the first array_set, which results in `v5` being invalid.
        let src = "
            acir(inline) fn main f0 {
              b0():
                v2 = make_array [Field 0] : [Field; 1]
                v3 = array_set v2, index u32 0, value Field 2
                v4 = make_array [v2, v2] : [[Field; 1]; 2]
                v5 = array_set v4, index u32 0, value v2
                return v5
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        // We expect the same result as above
        let ssa = ssa.array_set_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 0] : [Field; 1]
            v4 = array_set v1, index u32 0, value Field 2
            v5 = make_array [v1, v1] : [[Field; 1]; 2]
            v6 = array_set mut v5, index u32 0, value v1
            return v6
        }
        ");
    }
}
