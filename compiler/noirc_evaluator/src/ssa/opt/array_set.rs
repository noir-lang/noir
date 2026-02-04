//! The purpose of the `array_set_optimization` SSA pass is to mark `ArraySet` instructions
//! as mutable _iff_ the array is not potentially shared with the callers or callees of the
//! function and won't be used again in the function itself either. In other words, if this
//! is the last time we use this version of the array, we can mutate it in place, and avoid
//! having to make a copy of it.
//!
//! This optimization only applies to ACIR. In Brillig we use ref-counting to decide when
//! there are no other references to an array.
//! This pass assumes that Load and Store instructions have been previously removed.
//!
//! The pass is expected to run at most once, and requires these passes to occur before itself:
//! * unrolling
//! * flattening
//! * removal of if-else instructions
//! * mem2reg and DIE, in order to remove Load/Store instructions

use core::panic;
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
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

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

    assert_eq!(
        reachable_blocks.len(),
        1,
        "Expected there to be 1 block remaining in ACIR function for array_set optimization"
    );

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
                Instruction::Load { .. } => {
                    panic!("Load instruction exists before `array_set_optimization` pass");
                }
                Instruction::Store { .. } => {
                    panic!("Store instruction exists before `array_set_optimization` pass");
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
    // Brillig functions should not have any mutable array sets.
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
        let entry_block = self.entry_block();
        context.analyze_last_uses(entry_block);

        let instructions_to_update = mem::take(&mut context.instructions_that_can_be_made_mutable);
        make_mutable(&mut self.dfg, &instructions_to_update);
    }
}

struct Context<'f> {
    dfg: &'f DataFlowGraph,
    array_to_last_use: HashMap<ValueId, InstructionId>,
    instructions_that_can_be_made_mutable: HashSet<InstructionId>,
}

impl<'f> Context<'f> {
    fn new(dfg: &'f DataFlowGraph) -> Self {
        Context {
            dfg,
            array_to_last_use: HashMap::default(),
            instructions_that_can_be_made_mutable: HashSet::default(),
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

                    // We also want to check that the array is not part of the terminator arguments, as this means it is used again.
                    let mut is_array_in_terminator = false;
                    let mut is_nested = false;
                    terminator.for_each_value(|value| {
                        let is_value_array_in_terminator = value == *array;
                        if !is_value_array_in_terminator && self.dfg.type_of_value(value).is_array()
                        {
                            is_nested = true;
                            self.set_last_use(value, *instruction_id);
                        }
                        is_array_in_terminator |= is_value_array_in_terminator;
                    });

                    let can_mutate = !is_array_in_terminator && !is_nested;

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

                // Arrays nested in other arrays are a use.
                Instruction::MakeArray { elements, .. } => {
                    for element in elements {
                        if self.dfg.type_of_value(*element).is_array() {
                            self.set_last_use(*element, *instruction_id);
                        }
                    }
                }

                // The pass might mutate an array result of an `IfElse` and thus modify the input even if it's used later,
                // so we assert that such instructions have already been removed by the `remove_if_else` pass.
                Instruction::IfElse { .. } => {
                    unreachable!("IfElse instruction exists before `array_set_optimization` pass");
                }

                // Arrays loaded from memory might reference an existing array
                // For instance if the array comes from a load we may potentially be mutating an array
                // at a reference that is loaded from by other values.
                Instruction::Load { .. } => {
                    unreachable!("Load instruction exists before `array_set_optimization` pass");
                }
                // We also disallow Store instructions for the same reason.
                Instruction::Store { .. } => {
                    unreachable!("Store instruction exists before `array_set_optimization` pass");
                }

                // These instructions do not interact with arrays, so we do not need to track them.
                Instruction::Binary(..)
                | Instruction::Cast(..)
                | Instruction::Not(..)
                | Instruction::Truncate { .. }
                | Instruction::Constrain(..)
                | Instruction::ConstrainNotEqual(..)
                | Instruction::RangeCheck { .. }
                | Instruction::Allocate
                | Instruction::EnableSideEffectsIf { .. }
                | Instruction::IncrementRc { .. }
                | Instruction::DecrementRc { .. }
                | Instruction::Noop => (),
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
    use test_case::test_case;

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

        let ssa = ssa.array_set_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 0] : [Field; 1]
            v4 = array_set v1, index u32 0, value Field 2
            v5 = make_array [v1, v1] : [[Field; 1]; 2]
            v6 = array_set v5, index u32 0, value v1
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
    fn does_not_mutate_arrays_in_unreachable_blocks() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0] : [Field; 1]
                v4 = array_set v1, index u32 0, value Field 1
                constrain u1 0 == u1 1
                unreachable
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }

    // Demonstrate that we assume that `IfElse` instructions have been
    // removed by previous passes. Otherwise we would need to handle transitive
    // relations between arrays.
    #[test]
    #[should_panic]
    fn assumes_no_if_else() {
        // v4 can be v1 or v2. v1 is returned, so v4 should not be mutated.
        let src = "
            acir(inline) predicate_pure fn main f0 {
              b0(v0: u1, v1: [u32; 2], v2: [u32; 2]):
                v3 = not v0
                v4 = if v0 then v1 else (if v3) v2
                v5 = array_set v4, index u32 0, value u32 1
                return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ssa = ssa.array_set_optimization();
    }

    #[test]
    #[should_panic]
    fn assumes_no_load() {
        let src = "
            acir(inline) predicate_pure fn main f0 {
              b0(v0: u1, v1: [u32; 2], v2: [u32; 2]):
                v3 = load v2 -> [u32; 2]
                v4 = array_get v3, index u32 0 -> u32
                v5 = array_set v1, index u32 0, value v4
                return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ssa = ssa.array_set_optimization();
    }

    #[test]
    #[should_panic]
    fn assumes_no_store() {
        let src = "
            acir(inline) predicate_pure fn main f0 {
              b0(v0: u1, v1: [u32; 2], v2: [u32; 2]):
                v3 = allocate -> &mut [u32; 2]
                store v2 at v3
                v5 = array_set v3, index u32 0, value u32 1
                return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ssa = ssa.array_set_optimization();
    }

    #[test_case("inline")]
    #[test_case("fold")]
    #[should_panic = "Expected there to be 1 block remaining in ACIR function for array_set optimization"]
    fn disallows_multiple_blocks(inline_type: &str) {
        let src = format!(
            "
        acir({inline_type}) fn main f0 {{
          b0():
            v1 = make_array [Field 0] : [Field; 1]
            v2 = array_set v1, index u32 0, value Field 1
            jmp b1()
          b1():
            v3 = array_get v2, index u32 0 -> Field
            return v3
        }}"
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let _ssa = ssa.array_set_optimization();
    }

    // Previously, the first array_set instruction, which modifies v2 in the below
    // code snippet, was marked as mut despite v2 being used in the next array_set instruction.
    #[test]
    fn regression_10245() {
        let src = "
            acir(inline) predicate_pure fn main f0 {
              b0(v0: Field, v1: [[Field; 1]; 2], v2: [Field; 1]):
                v5 = array_set v2, index u32 0, value Field 4
                v6 = array_set v1, index u32 0, value v2
                return v6, v5
            }
            ";
        assert_ssa_does_not_change(src, Ssa::array_set_optimization);
    }
}
