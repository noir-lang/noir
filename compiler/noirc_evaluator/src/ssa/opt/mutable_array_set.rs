//! The purpose of the `mutable_array_set_optimization` SSA pass is to mark `ArraySet` instructions
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
    pub(crate) fn mutable_array_set_optimization(mut self) -> Self {
        for func in self.functions.values_mut() {
            #[cfg(debug_assertions)]
            mutable_array_set_optimization_pre_check(func);

            func.mutable_array_set_optimization();

            #[cfg(debug_assertions)]
            mutable_array_set_optimization_post_check(func);
        }
        self
    }
}

/// Pre-check condition for [Function::mutable_array_set_optimization].
///
/// Only applies to ACIR functions. Panics if:
///   - The function contains more than 1 block, i.e. it hasn't been flattened yet.
///   - There already exists a mutable array set instruction.
///   - There is an `IfElse` instruction which hasn't been removed yet.
///   - There are any Load or Store instructions.
#[cfg(debug_assertions)]
fn mutable_array_set_optimization_pre_check(func: &Function) {
    // This optimization only applies to ACIR functions
    if !func.runtime().is_acir() {
        return;
    }

    // flatten_cfg must have run
    super::checks::assert_cfg_is_flattened(func);
    super::checks::for_each_instruction(func, |instruction, _dfg| {
        // remove_if_else must have run
        super::checks::assert_not_if_else(instruction);
        // mem2reg must have run (no Load/Store remaining)
        super::checks::assert_not_load_or_store(instruction);
        // No mutable array sets should exist yet (they are created by this pass)
        super::checks::assert_not_mutable_array_set(instruction);
    });
}

/// Post-check condition for [Function::mutable_array_set_optimization].
///
/// Panics if a Brillig function contains mutable array set instructions.
/// Brillig uses ref-counting to decide whether to mutate an array, not mutable flags.
#[cfg(debug_assertions)]
fn mutable_array_set_optimization_post_check(func: &Function) {
    // Brillig functions should not have any mutable array sets
    if func.runtime().is_brillig() {
        super::checks::for_each_instruction(func, |instruction, _dfg| {
            super::checks::assert_not_mutable_array_set(instruction);
        });
    }
}

impl Function {
    pub(crate) fn mutable_array_set_optimization(&mut self) {
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
        }

        // We must prevent mutating arrays after they're stored in other arrays.
        let mut element_arrays = HashSet::default();

        for instruction_id in block.instructions() {
            match &self.dfg[*instruction_id] {
                // Reading an array constitutes as use, replacing any previous last use.
                Instruction::ArrayGet { array, .. } => {
                    self.set_last_use(*array, *instruction_id);
                }
                // Writing to an array is a use; mark it for mutation unless it might be shared.
                Instruction::ArraySet { array, value, .. } => {
                    self.set_last_use(*array, *instruction_id);

                    if self.dfg.type_of_value(*value).is_array() {
                        self.set_last_use(*value, *instruction_id);
                    }

                    // If the input array is itself returned, we cannot reuse its memory block.
                    let mut is_array_in_terminator = false;
                    terminator.for_each_value(|term_value| {
                        if term_value == *array {
                            is_array_in_terminator = true;
                        }
                    });

                    // Block mutation when the input array is actually nested as a MakeArray element
                    let is_nested = element_arrays.contains(array);
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

                // Arrays nested in other arrays are a use, and any subsequent `ArraySet`
                // on the element must not mutate in place.
                Instruction::MakeArray { elements, .. } => {
                    for element in elements {
                        if self.dfg.type_of_value(*element).is_array() {
                            element_arrays.insert(*element);
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
        ssa::{
            Ssa,
            interpreter::value::{NumericValue, Value},
            opt::{assert_pass_does_not_affect_execution, assert_ssa_does_not_change},
        },
    };
    use test_case::test_case;

    #[test]
    fn does_not_mutate_array_used_in_make_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/8563.
        // The critical invariant: the first `array_set` on `v1` must NOT become mutable,
        // because `v1` is later nested inside `v5` via `make_array` and used as the value
        // of the second `array_set`. Mutating `v1` in place would corrupt both.
        // The second `array_set` on `v5` CAN be mutable: `v5` is only used here.
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
        let ssa = ssa.mutable_array_set_optimization();
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
        assert_ssa_does_not_change(src, Ssa::mutable_array_set_optimization);
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
        assert_ssa_does_not_change(src, Ssa::mutable_array_set_optimization);
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
        assert_ssa_does_not_change(src, Ssa::mutable_array_set_optimization);
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
        let _ssa = ssa.mutable_array_set_optimization();
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
        let _ssa = ssa.mutable_array_set_optimization();
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
        let _ssa = ssa.mutable_array_set_optimization();
    }

    #[test_case("inline")]
    #[test_case("fold")]
    #[should_panic = "CFG contains more than 1 block"]
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
        let _ssa = ssa.mutable_array_set_optimization();
    }

    // Previously, the first array_set instruction, which modifies v2 in the below
    // code snippet, was marked as mut despite v2 being used in the next array_set instruction.
    // `v5` must NOT be mutable because `v2` is consumed again as the value argument of `v6`.
    // `v6` CAN be mutable because its input `v1` is a parameter used only here.
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mutable_array_set_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: [[Field; 1]; 2], v2: [Field; 1]):
            v5 = array_set v2, index u32 0, value Field 4
            v6 = array_set mut v1, index u32 0, value v2
            return v6, v5
        }
        ");
    }

    /// Two independent array chains returned together: every `array_set` should be
    /// marked mutable. Regression test for https://github.com/noir-lang/noir/issues/12411,
    /// where the over-broad terminator-based `is_nested` guard blocked all mutations
    /// because each chain's final value appeared as "another array in the terminator".
    #[test]
    fn marks_chain_mutable_with_two_independent_returned_arrays() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32, v2: u32):
                v4 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
                v5 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
                v7 = array_set v4, index v0, value Field 1
                v8 = array_set v5, index v0, value Field 1
                v10 = array_set v7, index v1, value Field 2
                v11 = array_set v8, index v1, value Field 2
                v13 = array_set v10, index v2, value Field 3
                v14 = array_set v11, index v2, value Field 3
                return v13, v14
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mutable_array_set_optimization();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v5 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v7 = array_set mut v4, index v0, value Field 1
            v8 = array_set mut v5, index v0, value Field 1
            v10 = array_set mut v7, index v1, value Field 2
            v11 = array_set mut v8, index v1, value Field 2
            v13 = array_set mut v10, index v2, value Field 3
            v14 = array_set mut v11, index v2, value Field 3
            return v13, v14
        }
        ");
    }

    // Mutating an element after it has been nested inside another array would also
    // corrupt the outer array, because the outer `make_array` captured the element's
    // memory. The later `array_set` on `v1` must therefore stay non-mutable even
    // though `v1` is not itself part of the terminator.
    #[test]
    fn does_not_mutate_array_set_on_element_after_nesting() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = make_array [Field 0, Field 0] : [Field; 2]
                v3 = make_array [v1] : [[Field; 2]; 1]
                v5 = array_set v1, index u32 0, value Field 7
                return v3, v5
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let (ssa, _value) =
            assert_pass_does_not_affect_execution(ssa, vec![], Ssa::mutable_array_set_optimization);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 0, Field 0] : [Field; 2]
            v2 = make_array [v1] : [[Field; 2]; 1]
            v5 = array_set v1, index u32 0, value Field 7
            return v2, v5
        }
        ");
    }

    #[test]
    fn array_set_array_value_should_mark_as_last_used() {
        // If we don't mark the value of `array_set` as last used then v7 ends up being
        //
        //     v7 = array_set mut v1, index u32 0, value Field 1
        //
        // which is incorrect.
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 0] : [Field; 1]
            v3 = make_array [Field 7] : [Field; 1]
            v4 = make_array [v3] : [[Field; 1]; 1]
            v7 = array_set v1, index u32 0, value Field 1
            v8 = array_set v4, index u32 0, value v1
            v9 = array_get v8, index u32 0 -> [Field; 1]
            v10 = array_get v9, index u32 0 -> Field
            return v10
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();

        let (ssa, value) =
            assert_pass_does_not_affect_execution(ssa, vec![], Ssa::mutable_array_set_optimization);
        assert_eq!(value.unwrap()[0], Value::Numeric(NumericValue::Field(0_u32.into())));

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 0] : [Field; 1]
            v3 = make_array [Field 7] : [Field; 1]
            v4 = make_array [v3] : [[Field; 1]; 1]
            v7 = array_set v1, index u32 0, value Field 1
            v8 = array_set mut v4, index u32 0, value v1
            v9 = array_get v8, index u32 0 -> [Field; 1]
            v10 = array_get v9, index u32 0 -> Field
            return v10
        }
        ");
    }
}
