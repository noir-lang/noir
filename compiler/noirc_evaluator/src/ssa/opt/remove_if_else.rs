//! This file contains the SSA `remove_if_else` pass - a required pass for ACIR to remove any remaining
//! `Instruction::IfElse` in the singular program-function, and replace them with
//! arithmetic operations using the `then_condition`.
//!
//! ACIR/Brillig differences within this pass:
//!   - This pass is strictly ACIR-only and never mutates brillig functions.
//!
//!
//! Conditions:
//!   - Precondition: Flatten CFG has been performed which should result in the function having only
//!     one basic block.
//!   - Precondition: `then_value` and `else_value` of `Instruction::IfElse` return arrays or slices. Numeric values should be handled previously by the flattening pass.
//!     Reference or function values are not handled by remove if-else and will cause an error.
//!   - Postcondition: A program without any `IfElse` instructions.
//!
//! Relevance to other passes:
//!   - Flattening inserts `Instruction::IfElse` to merge array or slice values from an if-expression's "then"
//!     and "else" branches. `Instruction::IfElse` with numeric values are directly handled during the flattening
//!     and will cause a panic in the `remove_if_else` pass.
//!   - Defunctionalize removes first-class function values from the program which eliminates the need for remove-if-else to handle `Instruction::IfElse` returning function values.
//!
//! Implementation details & examples:
//! `IfElse` instructions choose between its two operand values,
//! `then_value` and `else_value`, based on the `then_condition`:
//! ```
//!  if then_condition {
//!      then_value
//!  } else {
//!      else_value
//!  }
//! ```
//!
//! These instructions are inserted during the flatten cfg pass, which convert conditional control flow
//! at the basic block level into simple ternary operations returning a value, using these IfElse instructions,
//! and leaving only one basic block. The flatten cfg pass directly handles numeric values and issues
//! `Instruction::IfElse` only for arrays and slices. The remove-if-else pass is used for array and slices
//! in order to track their lengths, depending on existing slice intrinsics which modify slices,
//! or the array set instructions.
//! The `Instruction::IfElse` is removed using a `ValueMerger` which operates recursively for nested arrays/slices.
//!
//! For example, this code:
//! ```noir
//! fn main(x: bool, mut y: [u32; 2]) {
//!     if x {
//!          y[0] = 1;
//!     } else {
//!          y[0] = 2;
//!     }
//!
//!     assert(y[0] == 1);
//!  }
//!  ```
//!
//! will be translated into this code, where the `IfElse` instruction: `v9 = if v0 then v5 else (if v6) v8`
//! is using array v5 from then branch, and array v8 from the else branch:
//! ```
//! acir(inline) predicate_pure fn main f0 {
//!   b0(v0: u1, v1: [u32; 2]):
//!     v2 = allocate -> &mut [u32; 2]
//!     enable_side_effects v0
//!     v5 = array_set v1, index u32 0, value u32 1
//!     v6 = not v0
//!     enable_side_effects v6
//!     v8 = array_set v1, index u32 0, value u32 2
//!     v9 = if v0 then v5 else (if v6) v8
//!     enable_side_effects u1 1
//!     v11 = array_get v9, index u32 0 -> u32
//!     constrain v11 == u32 3
//!     return
//! }
//! ```
//!
//! The IfElse instruction is then replaced by these instruction during the remove if-else pass:
//! ```
//! v13 = cast v0 as u32
//! v14 = cast v6 as u32
//! v15 = unchecked_mul v14, u32 2  
//! v16 = unchecked_add v13, v15
//! v17 = array_get v5, index u32 1 -> u32
//! v18 = array_get v8, index u32 1 -> u32
//! v19 = cast v0 as u32
//! v20 = cast v6 as u32
//! v21 = unchecked_mul v19, v17
//! v22 = unchecked_mul v20, v18   
//! v23 = unchecked_add v21, v22
//! v24 = make_array [v16, v23] : [u32; 2]
//! ```
//!
//! The result of the removed `IfElse` instruction, array `v24`, is a merge of each of the elements of `v5` and `v8`.
//! The elements at index 0 are replaced by their known value, instead of doing an additional array get.
//! Operations with the conditions are unchecked operations, because the conditions are 0 or 1, so it cannot overflow.

use std::collections::hash_map::Entry;

use fxhash::FxHashMap as HashMap;

use crate::errors::RtResult;
use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::instruction::Hint;
use crate::ssa::ir::value::ValueId;
use crate::ssa::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::Type,
        value::Value,
    },
    opt::flatten_cfg::value_merger::ValueMerger,
};

impl Ssa {
    /// Replaces all `Instruction::IfElse` instructions with the result of a
    /// value merger of the then and else values. The specifics of the value merger
    /// depends on the type but is expected to be an equivalent value to the IfElse.
    /// For example, on integers, the merger will be:
    /// `then_condition * then_value + !then_condition * else_value`
    /// which should zero out the branch that was not taken.
    ///
    /// In general this is not possible for all types - notably references - which is
    /// why the Noir frontend does not allow references to be returned from if expressions.
    ///
    /// Also note that `Instruction::IfElse` are first inserted after the flattening pass,
    /// so before then this pass will have no effect.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_if_else(mut self) -> RtResult<Ssa> {
        for function in self.functions.values_mut() {
            function.remove_if_else()?;
        }
        Ok(self)
    }
}

impl Function {
    pub(crate) fn remove_if_else(&mut self) -> RtResult<()> {
        // This should match the check in flatten_cfg
        if matches!(self.runtime(), RuntimeType::Brillig(_)) {
            // skip
        } else {
            Context::default().remove_if_else(self)?;
        }

        #[cfg(debug_assertions)]
        remove_if_else_post_check(self);
        Ok(())
    }
}

#[derive(Default)]
struct Context {
    slice_sizes: HashMap<ValueId, u32>,
}

impl Context {
    /// Process each instruction in the entry block of the (fully flattened) function.
    /// Merge any `IfElse` instruction using a `ValueMerger` and track slice sizes
    /// through intrinsic calls and array set instructions.
    fn remove_if_else(&mut self, function: &mut Function) -> RtResult<()> {
        let block = function.entry_block();

        // Make sure this optimization runs when there's only one block
        assert_eq!(function.dfg[block].successors().count(), 0);

        function.simple_optimization_result(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            match instruction {
                Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                    let then_condition = *then_condition;
                    let else_condition = *else_condition;
                    let then_value = *then_value;
                    let else_value = *else_value;

                    let typ = context.dfg.type_of_value(then_value);
                    // Numeric values should have been handled during flattening
                    assert!(!matches!(typ, Type::Numeric(_)));

                    let call_stack = context.dfg.get_instruction_call_stack_id(instruction_id);
                    let mut value_merger =
                        ValueMerger::new(context.dfg, block, &mut self.slice_sizes, call_stack);

                    let value = value_merger.merge_values(
                        then_condition,
                        else_condition,
                        then_value,
                        else_value,
                    )?;

                    let _typ = context.dfg.type_of_value(value);
                    let results = context.dfg.instruction_results(instruction_id);
                    let result = results[0];

                    context.remove_current_instruction();
                    // The `IfElse` instruction is replaced by the merge done with the `ValueMerger`
                    context.replace_value(result, value);
                }
                Instruction::Call { func, arguments } => {
                    // Track slice sizes through intrinsic calls
                    if let Value::Intrinsic(intrinsic) = context.dfg[*func] {
                        let results = context.dfg.instruction_results(instruction_id);

                        match slice_capacity_change(context.dfg, intrinsic, arguments, results) {
                            SizeChange::None => (),
                            SizeChange::SetTo(value, new_capacity) => {
                                self.slice_sizes.insert(value, new_capacity);
                            }
                            SizeChange::Inc { old, new } => {
                                let old_capacity = self.get_or_find_capacity(context.dfg, old);
                                self.slice_sizes.insert(new, old_capacity + 1);
                            }
                            SizeChange::Dec { old, new } => {
                                let old_capacity = self.get_or_find_capacity(context.dfg, old);
                                // We use a saturating sub here as calling `pop_front` or `pop_back` on a zero-length slice
                                // would otherwise underflow.
                                self.slice_sizes.insert(new, old_capacity.saturating_sub(1));
                            }
                        }
                    }
                }
                // Track slice sizes through array set instructions
                Instruction::ArraySet { array, .. } => {
                    let results = context.dfg.instruction_results(instruction_id);
                    let result = if results.len() == 2 { results[1] } else { results[0] };

                    let old_capacity = self.get_or_find_capacity(context.dfg, *array);
                    self.slice_sizes.insert(result, old_capacity);
                }
                _ => (),
            }
            Ok(())
        })
    }

    //Get the tracked size of array/slices, or retrieve (and track) it for arrays.
    fn get_or_find_capacity(&mut self, dfg: &DataFlowGraph, value: ValueId) -> u32 {
        match self.slice_sizes.entry(value) {
            Entry::Occupied(entry) => return *entry.get(),
            Entry::Vacant(entry) => {
                if let Some((array, typ)) = dfg.get_array_constant(value) {
                    let length = array.len() / typ.element_types().len();
                    return *entry.insert(length as u32);
                }

                if let Type::Array(_, length) = dfg.type_of_value(value) {
                    return *entry.insert(length);
                }
            }
        }

        let dbg_value = &dfg[value];
        unreachable!("No size for slice {value} = {dbg_value:?}")
    }
}

enum SizeChange {
    None,
    SetTo(ValueId, u32),

    // These two variants store the old and new slice ids
    // not their lengths which should be old_len = new_len +/- 1
    Inc { old: ValueId, new: ValueId },
    Dec { old: ValueId, new: ValueId },
}

/// Find the change to a slice's capacity an instruction would have
fn slice_capacity_change(
    dfg: &DataFlowGraph,
    intrinsic: Intrinsic,
    arguments: &[ValueId],
    results: &[ValueId],
) -> SizeChange {
    match intrinsic {
        Intrinsic::SlicePushBack | Intrinsic::SlicePushFront | Intrinsic::SliceInsert => {
            // Expecting:  len, slice = ...
            assert_eq!(results.len(), 2);
            let old = arguments[1];
            let new = results[1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Inc { old, new }
        }

        Intrinsic::SlicePopBack | Intrinsic::SliceRemove => {
            let old = arguments[1];
            let new = results[1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Dec { old, new }
        }

        Intrinsic::SlicePopFront => {
            let old = arguments[1];
            let new = results[results.len() - 1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Dec { old, new }
        }

        Intrinsic::AsSlice => {
            assert_eq!(arguments.len(), 1);
            assert_eq!(results.len(), 2);
            let length = match dfg.type_of_value(arguments[0]) {
                Type::Array(_, length) => length,
                other => unreachable!("slice_capacity_change expected array, found {other:?}"),
            };
            assert!(matches!(dfg.type_of_value(results[1]), Type::Slice(_)));
            SizeChange::SetTo(results[1], length)
        }

        // These cases don't affect slice capacities
        Intrinsic::AssertConstant
        | Intrinsic::StaticAssert
        | Intrinsic::ApplyRangeConstraint
        | Intrinsic::ArrayLen
        | Intrinsic::ArrayAsStrUnchecked
        | Intrinsic::StrAsBytes
        | Intrinsic::BlackBox(_)
        | Intrinsic::Hint(Hint::BlackBox)
        | Intrinsic::AsWitness
        | Intrinsic::IsUnconstrained
        | Intrinsic::DerivePedersenGenerators
        | Intrinsic::ToBits(_)
        | Intrinsic::ToRadix(_)
        | Intrinsic::ArrayRefCount
        | Intrinsic::SliceRefCount
        | Intrinsic::FieldLessThan => SizeChange::None,
    }
}

/// Post-check condition for [Function::remove_if_else].
///
/// Succeeds if:
///   - `func` is a Brillig function, OR
///   - `func` does not contain any if-else instructions.
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn remove_if_else_post_check(func: &Function) {
    // Brillig functions should be unaffected.
    if func.runtime().is_brillig() {
        return;
    }

    // Otherwise there should be no if-else instructions in any reachable block.
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if matches!(func.dfg[*instruction_id], Instruction::IfElse { .. }) {
                panic!("IfElse instruction still remains in ACIR function");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn merge_basic_arrays() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 1;
        //         y[1] = 2;
        //     }
        //
        //     let z = y[0] + y[1];
        //     assert(z == 3);
        // }
        // ```
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v7 = array_set v5, index u32 1, value u32 2
            v8 = not v0
            v9 = if v0 then v7 else (if v8) v1
            enable_side_effects u1 1
            v11 = array_get v9, index u32 0 -> u32
            v12 = array_get v9, index u32 1 -> u32
            v13 = add v11, v12
            v15 = eq v13, u32 3
            constrain v13 == u32 3
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // In case our if block is never activated, we need to fetch each value from the original array.
        // We then should create a new array where each value can be mapped to `(then_condition * then_value) + (!then_condition * else_value)`.
        // The `then_value` and `else_value` for an array will be every element of the array. Thus, we should see array_get operations
        // on the original array as well as the new values we are writing to the array.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v7 = array_set v5, index u32 1, value u32 2
            v8 = not v0
            v9 = array_get v1, index u32 0 -> u32
            v10 = cast v0 as u32
            v11 = cast v8 as u32
            v12 = unchecked_mul v11, v9
            v13 = unchecked_add v10, v12
            v14 = array_get v1, index u32 1 -> u32
            v15 = cast v0 as u32
            v16 = cast v8 as u32
            v17 = unchecked_mul v15, u32 2
            v18 = unchecked_mul v16, v14
            v19 = unchecked_add v17, v18
            v20 = make_array [v13, v19] : [u32; 2]
            enable_side_effects u1 1
            v22 = array_get v20, index u32 0 -> u32
            v23 = array_get v20, index u32 1 -> u32
            v24 = add v22, v23
            v26 = eq v24, u32 3
            constrain v24 == u32 3
            return
        }
        ");
    }

    #[test]
    fn try_merge_only_changed_indices() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 1;
        //     }
        //
        //     let z = y[0] + y[1];
        //     assert(z == 1);
        // }
        // ```
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v6 = not v0
            v7 = if v0 then v5 else (if v6) v1
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> u32
            v10 = array_get v7, index u32 1 -> u32
            v11 = add v9, v10
            v12 = eq v11, u32 1
            constrain v11 == u32 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // We attempt to optimize array mergers to only handle where an array was modified,
        // rather than merging the entire array. As we only modify the `y` array at a single index,
        // we instead only map the if predicate onto the the numeric value we are looking to write,
        // and then write into the array directly.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v6 = not v0
            v7 = array_get v1, index u32 0 -> u32
            v8 = cast v0 as u32
            v9 = cast v6 as u32
            v10 = unchecked_mul v9, v7
            v11 = unchecked_add v8, v10
            v12 = array_get v5, index u32 1 -> u32
            v13 = array_get v1, index u32 1 -> u32
            v14 = cast v0 as u32
            v15 = cast v6 as u32
            v16 = unchecked_mul v14, v12
            v17 = unchecked_mul v15, v13
            v18 = unchecked_add v16, v17
            v19 = make_array [v11, v18] : [u32; 2]
            enable_side_effects u1 1
            v21 = array_get v19, index u32 0 -> u32
            v22 = array_get v19, index u32 1 -> u32
            v23 = add v21, v22
            v24 = eq v23, u32 1
            constrain v23 == u32 1
            return
        }
        ");
    }

    #[test]
    fn merge_slice() {
        let src = "
acir(inline) impure fn main f0 {
  b0(v0: u1, v1: Field, v2: Field):
    v3 = make_array [] : [Field]   
    v4 = allocate -> &mut u32       
    v5 = allocate -> &mut [Field]    
    enable_side_effects v0
    v6 = cast v0 as u32
    v7, v8 = call slice_push_back(v6, v3, v2) -> (u32, [Field])
    v9 = not v0                                   
    v10 = cast v0 as u32   
    v12 = if v0 then v8 else (if v9) v3   
    enable_side_effects u1 1        
    v15, v16 = call slice_push_back(v10, v12, v2) -> (u32, [Field])
    v17 = array_get v16, index u32 0 -> Field    
    constrain v17 == Field 1
    return
}
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Merge slices v3 (empty) and v8 ([v2]) into v12, using a dummy value for the element at index 0 of v3, which does not exist.
        assert_ssa_snapshot!(ssa, @r"
acir(inline) impure fn main f0 {
  b0(v0: u1, v1: Field, v2: Field):
    v3 = make_array [] : [Field]
    v4 = allocate -> &mut u32
    v5 = allocate -> &mut [Field]
    enable_side_effects v0
    v6 = cast v0 as u32
    v8, v9 = call slice_push_back(v6, v3, v2) -> (u32, [Field])
    v10 = not v0
    v11 = cast v0 as u32
    v13 = array_get v9, index u32 0 -> Field
    v14 = cast v0 as Field
    v15 = cast v10 as Field
    v16 = mul v14, v13
    v17 = make_array [v16] : [Field]
    enable_side_effects u1 1
    v19, v20 = call slice_push_back(v11, v17, v2) -> (u32, [Field])
    v21 = array_get v20, index u32 0 -> Field
    constrain v21 == Field 1
    return
}
      ");
    }
}
