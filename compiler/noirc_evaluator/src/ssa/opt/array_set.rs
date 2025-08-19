//! The purpose of the `array_set_optimization` SSA pass is to mark `ArraySet` instructions
//! as mutable _iff_ the array is not potentially shared with the callers or callees of the
//! function and won't be used again in the function itself either. In other words, if this
//! is the last time we use this version of the array, we can mutate it in place, and avoid
//! having to make a copy of it.
//!
//! This optimization only applies to ACIR. In Brillig we use ref-counting to decide when
//! there are no other references to an array.
//!
//! The pass is expected to run at most once, and requires these passes to occur before itself:
//! * unrolling
//! * flattening
//! * removal of if-else instructions

use std::mem;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

impl Ssa {
    /// Finds the last instruction that writes to an array and modifies it
    /// to do an in-place mutation instead of making a copy if there are
    /// no potential shared references to it.
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
///   - An ACIR function contains more than 1 block, i.e. it hasn't been flattened yet.
///   - There already exists a mutable array set instruction.
///   - There is an `IfElse` instruction which hasn't been removed yet.
#[cfg(debug_assertions)]
fn array_set_optimization_pre_check(func: &Function) {
    // We only want to run this pass for ACIR.
    if func.runtime().is_brillig() {
        return;
    }

    let reachable_blocks = func.reachable_blocks();

    if !func.runtime().is_entry_point() {
        assert_eq!(
            reachable_blocks.len(),
            1,
            "Expected there to be 1 block remaining in ACIR function for array_set optimization"
        );
    }

    for block_id in reachable_blocks {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            match func.dfg[*instruction_id] {
                // There should be no mutable array sets.
                Instruction::ArraySet { mutable: true, .. } => {
                    panic!(
                        "mutable ArraySet instruction exists before `array_set_optimization` pass"
                    );
                }
                // The pass might mutate an array result of an `IfElse` and thus modify the input even if it's used later,
                // so we assert that such instructions have already been removed by the `remove_if_else` pass.
                Instruction::IfElse { .. } => {
                    panic!("IfElse instruction exists before `array_set_optimization` pass");
                }
                _ => {}
            }
        }
    }
}

/// Post-check condition for [Function::array_set_optimization].
///
/// Panics if:
///   - Mutable array_set optimization has been applied to Brillig function.
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
        if self.runtime().is_brillig() {
            // Brillig is supposed to use ref-counting to decide whether to mutate an array;
            // array mutation was only meant for ACIR. We could use it with Brillig as well,
            // but then some of the optimizations that we can do in ACIR around shared
            // references have to be skipped, which makes it more cumbersome.
            return;
        }

        let mut context = Context::new(&self.dfg);

        for block in self.reachable_blocks() {
            context.analyze_last_uses(block);
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

    /// Remember this instruction as the last time the array has been read or written to.
    ///
    /// Any previous instruction marked to be made mutable needs to be cancelled,
    /// as it turned out not to be the last use.
    fn set_last_use(&mut self, array: ValueId, instruction_id: InstructionId) {
        if let Some(existing) = self.array_to_last_use.insert(array, instruction_id) {
            self.instructions_that_can_be_made_mutable.remove(&existing);
        }
    }

    /// Builds the set of `ArraySet` instructions that can be made mutable
    /// because their input value is unused elsewhere afterward.
    ///
    /// Only expected to execute on ACIR functions.
    fn analyze_last_uses(&mut self, block_id: BasicBlockId) {
        assert!(self.dfg.runtime().is_acir());

        let block = &self.dfg[block_id];

        let terminator = self.dfg[block_id].unwrap_terminator();

        // If we are in a return block we are not concerned about the array potentially being mutated again.
        // In ACIR this should be the only kind of block we encounter, unless it's marked unreachable,
        // in which case we don't need to optimize the array writes since we will end up with a failure anyway.
        match terminator {
            TerminatorInstruction::Return { .. } => {}
            TerminatorInstruction::Unreachable { .. } => {
                return;
            }
            other => {
                panic!("unexpected terminator in ACIR: {other:?}")
            }
        };

        for instruction_id in block.instructions() {
            match &self.dfg[*instruction_id] {
                // Reading an array constitutes as use, replacing any previous last use.
                Instruction::ArrayGet { array, .. } => {
                    self.set_last_use(*array, *instruction_id);
                }
                // Writing to an array is a use; mark it for mutation unless it might be shared.
                Instruction::ArraySet { array, .. } => {
                    self.set_last_use(*array, *instruction_id);

                    // If the array we are setting does not come from a load we can safely mark it mutable.
                    // If the array comes from a load we may potentially being mutating an array at a reference
                    // that is loaded from by other values.

                    // We also want to check that the array is not part of the terminator arguments, as this means it is used again.
                    let mut is_array_in_terminator = false;
                    terminator.for_each_value(|value| {
                        is_array_in_terminator |= value == *array;
                    });

                    let can_mutate = if let Some(is_from_param) = self.arrays_from_load.get(array) {
                        !is_from_param
                    } else {
                        !is_array_in_terminator
                    };

                    if can_mutate {
                        self.instructions_that_can_be_made_mutable.insert(*instruction_id);
                    }
                }
                // Array arguments passed in calls constitute a use.
                Instruction::Call { arguments, .. } => {
                    for argument in arguments {
                        if self.dfg.type_of_value(*argument).is_array() {
                            self.set_last_use(*argument, *instruction_id);
                        }
                    }
                }
                // Arrays loaded from input references might be shared with the caller.
                Instruction::Load { address } => {
                    let result = self.dfg.instruction_results(*instruction_id)[0];
                    if self.dfg.type_of_value(result).is_array() {
                        let is_reference_param =
                            self.dfg.block_parameters(block_id).contains(address);
                        self.arrays_from_load.insert(result, is_reference_param);
                    }
                }
                // Arrays nested in other arrays are a use.
                Instruction::MakeArray { elements, .. } => {
                    for element in elements {
                        if self.dfg.type_of_value(*element).is_array() {
                            self.set_last_use(*element, *instruction_id);
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
        ssa::{Ssa, opt::assert_ssa_does_not_change},
    };

    #[test]
    fn array_set_in_loop_with_conditional_clone() {
        // We want to make sure that we do not mark a single array set mutable which is loaded
        // from and cloned in a loop. If the array is inadvertently marked mutable, and is cloned in a previous iteration
        // of the loop, its clone will also be altered.

        // Note that this is a Brillig function, so it's skipped altogether.
        // We can't just change it to ACIR, because it uses multiple blocks, it's not unrolled and flattened.
        // It's left here to prevent any regressions, should we ever run the array_set optimization on Brillig again.
        let src = "
            brillig(inline) fn main f0 {
              b0():
                v2 = make_array [Field 0, Field 0, Field 0, Field 0, Field 0] : [Field; 5]
                v3 = make_array [v2, v2] : [[Field; 5]; 2]
                v4 = allocate -> &mut [[Field; 5]; 2]
                store v3 at v4
                v5 = allocate -> &mut [[Field; 5]; 2]
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
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
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

        // The first array_set should not be mutable, but the second one can be.
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

    #[test]
    fn does_not_mutate_array_used_as_call_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = array_set v1, index u32 0, value Field 1
                v3 = call f1(v1) -> Field
                return v3
            }

            brillig(inline) fn func_1 f1 {
              b0(v0: [Field; 1]):
                v1 = array_get v0, index u32 0 -> Field
                return v1
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    #[test]
    fn does_not_mutate_array_returned() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = array_set v1, index u32 0, value Field 1
                return v1
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    #[test]
    fn does_not_mutate_array_loaded_from_reference_parameter() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = allocate -> &mut [Field; 1]
                store v1 at v2
                call f1(v2)
                return
            }

            acir(inline) fn func_1 f1 {
              b0(v0: &mut [Field; 1]):
                v1 = load v0 -> [Field; 1]
                v2 = array_set v1, index u32 0, value Field 1
                return
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    #[test]
    fn mutate_arrays_loaded_from_non_parameter_reference() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = allocate -> &mut [Field; 1]
                store v1 at v2
                v3 = load v2 -> [Field; 1]
                v4 = array_set v3, index u32 0, value Field 1
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.array_set_optimization();
        assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = allocate -> &mut [Field; 1]
                store v1 at v2
                v3 = load v2 -> [Field; 1]
                v6 = array_set mut v3, index u32 0, value Field 1
                return
            }
        ");
    }

    #[test]
    fn does_not_mutate_arrays_in_unreachable_blocks() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v2 = allocate -> &mut [Field; 1]
                store v1 at v2
                v3 = load v2 -> [Field; 1]
                v4 = array_set v3, index u32 0, value Field 1
                constrain u1 0 == u1 1
                unreachable
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    #[test]
    #[should_panic = "instruction exists before"]
    fn does_not_mutate_array_with_if_else() {
        let src = "
            acir(inline) predicate_pure fn main f0 {
              b0(v0: u1, v1: [u32; 2], v2: [u32; 2]):
                v3 = not v0
                v4 = if v0 then v1 else (if v3) v2
                v5 = array_set v4, index u32 0, value u32 1
                return v1
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }
}
