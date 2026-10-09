//! This file contains the SSA `remove_if_else` pass - a required pass for ACIR to remove any
//! remaining `Instruction::IfElse` in the singular program-function, and replace them with
//! arithmetic operations using the `then_condition`.
//!
//! ACIR/Brillig differences within this pass:
//!   - This pass is strictly ACIR-only and never mutates brillig functions.
//!
//!
//! Conditions:
//!   - Precondition: Flatten CFG has been performed which should result in the function having only
//!     one basic block.
//!   - Precondition: `then_value` and `else_value` of `Instruction::IfElse` return arrays or vectors.
//!     Numeric values should be handled previously by the flattening pass.
//!     Reference or function values are not handled by remove if-else and will cause an error.
//!   - Postcondition: A program without any `IfElse` instructions.
//!
//! Relevance to other passes:
//!   - Flattening inserts `Instruction::IfElse` to merge array or vector values from an
//!     if-expression's "then" and "else" branches. `Instruction::IfElse` with numeric values are
//!     directly handled during flattening, [via instruction simplification][crate::ssa::ir::dfg::simplify::simplify],
//!     and will cause a panic in the `remove_if_else` pass.
//!   - Defunctionalize removes first-class function values from the program which eliminates the need
//!     for remove-if-else to handle `Instruction::IfElse` returning function values.
//!
//! Implementation details & examples:
//! `IfElse` instructions choose between its two operand values,
//! `then_value` and `else_value`, based on the `then_condition`:
//! ```ssa
//!  if then_condition {
//!      then_value
//!  } else {
//!      else_value
//!  }
//! ```
//!
//! These instructions are inserted during the flatten cfg pass, which convert conditional control flow
//! at the basic block level into simple ternary operations returning a value, using these `IfElse` instructions,
//! and leaving only one basic block. The flatten cfg pass directly handles numeric values and issues
//! `Instruction::IfElse` only for arrays and vectors. The remove-if-else pass is used for array and vectors
//! in order to track their lengths, depending on existing vector intrinsics which modify vectors,
//! or the array set instructions.
//! The `Instruction::IfElse` is removed using a `ValueMerger` which operates recursively for nested arrays/vectors.
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
//!     assert(y[0] == 3);
//!  }
//!  ```
//!
//! will be translated into this code, where the `IfElse` instruction: `v9 = if v0 then v5 else (if v6) v8`
//! is using array v5 from then branch, and array v8 from the else branch:
//! ```ssa
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
//! The `IfElse` instruction is then replaced by these instruction during the remove if-else pass:
//! ```ssa
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
//!
//! For vectors the logic is similar except that vector lengths need to be tracked in order to know
//! the length of the merged vector resulting in a `make_array` instruction. This length will be the
//! maximum length of the two input vectors. Note that the actual length of the merged vector should
//! have been merged during flattening.

use std::collections::hash_map::Entry;

use acvm::acir::brillig::lengths::SemanticLength;
use acvm::{AcirField, FieldElement};
use rustc_hash::FxHashMap as HashMap;

use crate::errors::RtResult;

use crate::ssa::ir::dfg::simplify::value_merger::ValueMerger;
use crate::ssa::ir::types::NumericType;
use crate::ssa::opt::ArrayGetOptimizationSideEffects;
use crate::ssa::{
    Ssa,
    ir::{
        dfg::{
            DataFlowGraph,
            vector_capacity::{constant_vector_lengths, vector_capacity_flows},
        },
        function::Function,
        instruction::Instruction,
        types::Type,
        value::{Value, ValueId},
    },
};

impl Ssa {
    /// Replaces all `Instruction::IfElse` instructions with the result of a
    /// value merger of the then and else values. The specifics of the value merger
    /// depends on the type but is expected to be an equivalent value to the `IfElse`.
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
        if self.runtime().is_brillig() {
            return Ok(());
        }

        #[cfg(debug_assertions)]
        remove_if_else_pre_check(self);

        Context::default().remove_if_else(self)?;

        #[cfg(debug_assertions)]
        remove_if_else_post_check(self);
        Ok(())
    }
}

#[derive(Default)]
struct Context {
    /// Keeps track of each size a vector is known to be.
    ///
    /// This is passed to the `ValueMerger` because when merging two vectors
    /// we need to know their sizes to create the merged vector.
    ///
    /// Note: as this pass operates on a single block, which is an entry block,
    /// and because vectors are disallowed in entry blocks, all vector lengths
    /// should be known at this point.
    vector_sizes: HashMap<ValueId, SemanticLength>,
}

impl Context {
    /// Process each instruction in the entry block of the (fully flattened) function.
    /// Merge any `IfElse` instruction using a `ValueMerger` and track vector sizes
    /// through intrinsic calls and array set instructions.
    fn remove_if_else(&mut self, function: &mut Function) -> RtResult<()> {
        let block = function.entry_block();

        // Early return if there is no IfElse instruction.
        if !function.dfg[block]
            .instructions()
            .iter()
            .any(|inst| matches!(function.dfg[*inst], Instruction::IfElse { .. }))
        {
            return Ok(());
        }

        // Keeps track of side effect vars associated to each `array_set` instruction.
        let mut array_set_predicates = std::collections::HashMap::new();

        function.simple_optimization_result(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            match instruction {
                Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                    let then_condition = *then_condition;
                    let else_condition = *else_condition;
                    let then_value = *then_value;
                    let else_value = *else_value;

                    // Register values for the merger to use.
                    self.ensure_capacity(context.dfg, then_value);
                    self.ensure_capacity(context.dfg, else_value);

                    // Because the ValueMerger might produce some `array_get` instructions, we
                    // need those to always execute as otherwise they'll produce incorrect
                    // merged arrays. For this, we set the side effects var to `true` for the merge.
                    let old_side_effects = context.enable_side_effects;
                    let old_side_effects_is_not_one = context
                        .dfg
                        .get_numeric_constant(old_side_effects)
                        .is_none_or(|value| !value.is_one());

                    if old_side_effects_is_not_one {
                        let one =
                            context.dfg.make_constant(FieldElement::one(), NumericType::bool());
                        let _ = context.insert_instruction(
                            Instruction::EnableSideEffectsIf { condition: one },
                            None,
                        );
                    }

                    let call_stack = context.dfg.get_instruction_call_stack_id(instruction_id);
                    let array_get_optimization_data = Some(ArrayGetOptimizationSideEffects {
                        side_effects_var: context.enable_side_effects,
                        array_set_predicates: &array_set_predicates,
                    });
                    let mut value_merger = ValueMerger::new(
                        context.dfg,
                        block,
                        &self.vector_sizes,
                        call_stack,
                        array_get_optimization_data,
                    );

                    let value = value_merger.merge_values(
                        then_condition,
                        else_condition,
                        then_value,
                        else_value,
                    )?;

                    if old_side_effects_is_not_one {
                        let _ = context.insert_instruction(
                            Instruction::EnableSideEffectsIf { condition: old_side_effects },
                            None,
                        );
                    }

                    let [result] = context.dfg.instruction_result(instruction_id);

                    context.remove_current_instruction();
                    // The `IfElse` instruction is replaced by the merge done with the `ValueMerger`
                    context.replace_value(result, value);
                }
                Instruction::Call { func, arguments } => {
                    // Track vector sizes through intrinsic calls
                    if let Value::Intrinsic(intrinsic) = context.dfg[*func] {
                        let results = context.dfg.instruction_results(instruction_id);

                        // If we have already determined a constant for the vector length, we can
                        // override the backing capacity of the vector contents. Using the capacity
                        // over the vector length would require laying down more instructions to
                        // handle the extra padding, while preventing downstream passes or runtimes
                        // from implementing optimizations using the vector length.
                        // A call that only runs under a predicate says nothing about the vector's
                        // length when the predicate is false, so its length is only used when the
                        // call always runs.
                        let always_runs = context
                            .dfg
                            .get_numeric_constant(context.enable_side_effects)
                            .is_some_and(|predicate| predicate.is_one());
                        if always_runs {
                            for (vector, length) in
                                constant_vector_lengths(context.dfg, intrinsic, arguments)
                            {
                                self.vector_sizes.insert(vector, length);
                            }
                        }

                        for flow in
                            vector_capacity_flows(context.dfg, intrinsic, arguments, results)
                        {
                            self.set_capacity(context.dfg, flow.input, flow.output, |capacity| {
                                // Growing the capacity must increase it: it cannot wrap around
                                // or saturate.
                                flow.change.apply(capacity).expect("Vector capacity overflow")
                            });
                        }
                    }
                }
                // Track vector sizes through array set instructions
                Instruction::ArraySet { array, .. } => {
                    array_set_predicates.insert(instruction_id, context.enable_side_effects);

                    let [result] = context.dfg.instruction_result(instruction_id);
                    self.set_capacity(context.dfg, *array, result, |c| c);
                }
                _ => (),
            }
            Ok(())
        })
    }

    /// Set the capacity of the new vector based on the capacity of the old array/vector.
    fn set_capacity(
        &mut self,
        dfg: &DataFlowGraph,
        old: ValueId,
        new: ValueId,
        f: impl Fn(SemanticLength) -> SemanticLength,
    ) {
        // No need to store the capacity of arrays, only vectors.
        if !matches!(*dfg.type_of_value(new), Type::Vector(_)) {
            return;
        }

        // Track new's capacity if old's is known, on a best-effort basis.
        if let Some(capacity) = self.get_or_find_capacity(dfg, old) {
            self.vector_sizes.insert(new, f(capacity));
        }
    }

    /// Make sure the vector capacity is recorded.
    fn ensure_capacity(&mut self, dfg: &DataFlowGraph, vector: ValueId) {
        self.set_capacity(dfg, vector, vector, |c| c);
    }

    /// Get size of array/vectors, and track it in `vector_sizes`.
    fn get_or_find_capacity(
        &mut self,
        dfg: &DataFlowGraph,
        value: ValueId,
    ) -> Option<SemanticLength> {
        match self.vector_sizes.entry(value) {
            Entry::Occupied(entry) => Some(*entry.get()),
            Entry::Vacant(entry) => {
                dfg.try_get_vector_capacity(value).map(|len| *entry.insert(len))
            }
        }
    }
}

#[cfg(debug_assertions)]
fn remove_if_else_pre_check(func: &Function) {
    // flatten_cfg must have run
    super::checks::assert_cfg_is_flattened(func);
    // IfElse should only be on arrays/vectors, not numeric types
    super::checks::for_each_instruction(func, |instruction, dfg| {
        super::checks::assert_not_if_else_on_numeric(instruction, dfg);
    });
}

/// Post-check condition for [`Function::remove_if_else`].
///
/// Succeeds if:
///   - `func` is a Brillig function, OR
///   - `func` does not contain any if-else instructions.
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn remove_if_else_post_check(func: &Function) {
    // All IfElse instructions should be removed
    super::checks::for_each_instruction(func, |instruction, _dfg| {
        super::checks::assert_not_if_else(instruction);
    });
}

#[cfg(test)]
mod tests {
    use acvm::{AcirField, FieldElement};

    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{errors::InterpreterError, value::Value},
            opt::assert_pass_does_not_affect_execution,
            ssa_gen::Ssa,
        },
    };

    #[test]
    fn merge_basic_arrays() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 2;
        //         y[1] = 3;
        //     }
        //
        //     let z = y[0] + y[1];
        //     assert(z == 5);
        // }
        // ```
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 2
            v7 = array_set v5, index u32 1, value u32 3
            v8 = not v0
            v9 = if v0 then v7 else (if v8) v1
            enable_side_effects u1 1
            v11 = array_get v9, index u32 0 -> u32
            v12 = array_get v9, index u32 1 -> u32
            v13 = add v11, v12
            v15 = eq v13, u32 5
            constrain v13 == u32 5
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
            v5 = array_set v1, index u32 0, value u32 2
            v8 = array_set v5, index u32 1, value u32 3
            v9 = not v0
            enable_side_effects u1 1
            v11 = array_get v1, index u32 0 -> u32
            v12 = cast v0 as u32
            v13 = cast v9 as u32
            v14 = unchecked_mul v12, u32 2
            v15 = unchecked_mul v13, v11
            v16 = unchecked_add v14, v15
            v17 = array_get v1, index u32 1 -> u32
            v18 = cast v0 as u32
            v19 = cast v9 as u32
            v20 = unchecked_mul v18, u32 3
            v21 = unchecked_mul v19, v17
            v22 = unchecked_add v20, v21
            v23 = make_array [v16, v22] : [u32; 2]
            enable_side_effects v0
            enable_side_effects u1 1
            v24 = add v16, v22
            v26 = eq v24, u32 5
            constrain v24 == u32 5
            return
        }
        ");
    }

    #[test]
    fn merges_all_indices_even_if_they_did_not_change() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 2;
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
            v5 = array_set v1, index u32 0, value u32 2
            v6 = not v0
            v7 = if v0 then v5 else (if v6) v1
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> u32
            v10 = array_get v7, index u32 1 -> u32
            v11 = add v9, v10
            v12 = eq v11, u32 3
            constrain v11 == u32 3
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // In the past we used to optimize array mergers to only handle where an array was modified,
        // rather than merging the entire array.
        // However, that was removed in https://github.com/noir-lang/noir/pull/8142
        // Pending: investigate if this can be brought back: https://github.com/noir-lang/noir/issues/8145
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 2
            v6 = not v0
            enable_side_effects u1 1
            v8 = array_get v1, index u32 0 -> u32
            v9 = cast v0 as u32
            v10 = cast v6 as u32
            v11 = unchecked_mul v9, u32 2
            v12 = unchecked_mul v10, v8
            v13 = unchecked_add v11, v12
            v15 = array_get v1, index u32 1 -> u32
            v16 = array_get v1, index u32 1 -> u32
            v17 = cast v0 as u32
            v18 = cast v6 as u32
            v19 = unchecked_mul v17, v15
            v20 = unchecked_mul v18, v16
            v21 = unchecked_add v19, v20
            v22 = make_array [v13, v21] : [u32; 2]
            enable_side_effects v0
            enable_side_effects u1 1
            v23 = add v13, v21
            v25 = eq v23, u32 3
            constrain v23 == u32 3
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_push_back() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8 = call vector_push_back(v6, v3, v2) -> (u32, [Field])
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16 = call vector_push_back(v10, v12, v2) -> (u32, [Field])
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Merge vectors v3 (empty) and v8 ([v2]) into v12, directly using v13 as the first element
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v8, v9 = call vector_push_back(v6, v3, v2) -> (u32, [Field])
            v10 = not v0
            v11 = cast v0 as u32
            enable_side_effects u1 1
            v14 = array_get v9, index u32 0 -> Field
            v15 = make_array [v14] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v17 = add v11, u32 1
            v18 = make_array [v14, v2] : [Field]
            v19 = array_set v18, index v11, value v2
            v20 = array_get v19, index u32 0 -> Field
            constrain v20 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_push_front() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8 = call vector_push_front(v6, v3, v2) -> (u32, [Field])
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16 = call vector_push_front(v10, v12, v2) -> (u32, [Field])
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here v14 is the result of the merge (keep `[v13]`)
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v8, v9 = call vector_push_front(v6, v3, v2) -> (u32, [Field])
            v10 = not v0
            v11 = cast v0 as u32
            enable_side_effects u1 1
            v14 = array_get v9, index u32 0 -> Field
            v15 = make_array [v14] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v17 = add v11, u32 1
            v18 = make_array [v2, v14] : [Field]
            constrain v2 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_as_vector_and_vector_push_front() {
        // Same as the previous test, but using `as_vector` to prove that vector length tracking
        // is working correctly.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v102 = make_array [] : [Field; 0]
            v103, v3 = call as_vector(v102) -> (u32, [Field])
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8 = call vector_push_front(v6, v3, v2) -> (u32, [Field])
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16 = call vector_push_front(v10, v12, v2) -> (u32, [Field])
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here v17 is the result of the merge (keep `[v16]`)
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field; 0]
            v5, v6 = call as_vector(v3) -> (u32, [Field])
            v7 = allocate -> &mut u32
            v8 = allocate -> &mut [Field]
            enable_side_effects v0
            v9 = cast v0 as u32
            v11, v12 = call vector_push_front(v9, v6, v2) -> (u32, [Field])
            v13 = not v0
            v14 = cast v0 as u32
            enable_side_effects u1 1
            v17 = array_get v12, index u32 0 -> Field
            v18 = make_array [v17] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v20 = add v14, u32 1
            v21 = make_array [v2, v17] : [Field]
            constrain v2 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_insert() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8 = call vector_insert(v6, v3, u32 0, v2) -> (u32, [Field])
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16 = call vector_insert(v10, v12, u32 0, v2) -> (u32, [Field])
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here v14 is the result of the merge (keep `[v13]`)
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v9, v10 = call vector_insert(v6, v3, u32 0, v2) -> (u32, [Field])
            v11 = not v0
            v12 = cast v0 as u32
            enable_side_effects u1 1
            v14 = array_get v10, index u32 0 -> Field
            v15 = make_array [v14] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v17 = add v12, u32 1
            v18 = make_array [v2, v14] : [Field]
            constrain v2 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_pop_back() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [Field 2, Field 3] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8, v100 = call vector_pop_back(v6, v3) -> (u32, [Field], Field)
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16, v101 = call vector_pop_back(v10, v12) -> (u32, [Field], Field)
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here [v21, Field 3] is the result of merging the original vector (`[Field 2, Field 3]`)
        // with the other vector, where `v21` merges the two values.
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v5 = make_array [Field 2, Field 3] : [Field]
            v6 = allocate -> &mut u32
            v7 = allocate -> &mut [Field]
            enable_side_effects v0
            v8 = cast v0 as u32
            v10, v11, v12 = call vector_pop_back(v8, v5) -> (u32, [Field], Field)
            v13 = not v0
            v14 = cast v0 as u32
            enable_side_effects u1 1
            v17 = array_get v11, index u32 0 -> Field
            v18 = cast v0 as Field
            v19 = cast v13 as Field
            v20 = mul v18, v17
            v21 = mul v19, Field 2
            v22 = add v20, v21
            v23 = make_array [v22, Field 3] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v24, v25, v26 = call vector_pop_back(v14, v23) -> (u32, [Field], Field)
            v27 = array_get v25, index u32 0 -> Field
            constrain v27 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_pop_front() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [Field 2, Field 3] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v100, v7, v8 = call vector_pop_front(v6, v3) -> (Field, u32, [Field])
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v101, v15, v16 = call vector_pop_front(v10, v12) -> (Field, u32, [Field])
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here [v21, Field 3] is the result of merging the original vector (`[Field 2, Field 3]`)
        // where for v21 it's the merged value.
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v5 = make_array [Field 2, Field 3] : [Field]
            v6 = allocate -> &mut u32
            v7 = allocate -> &mut [Field]
            enable_side_effects v0
            v8 = cast v0 as u32
            v10, v11, v12 = call vector_pop_front(v8, v5) -> (Field, u32, [Field])
            v13 = not v0
            v14 = cast v0 as u32
            enable_side_effects u1 1
            v17 = array_get v12, index u32 0 -> Field
            v18 = cast v0 as Field
            v19 = cast v13 as Field
            v20 = mul v18, v17
            v21 = mul v19, Field 2
            v22 = add v20, v21
            v23 = make_array [v22, Field 3] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v24, v25, v26 = call vector_pop_front(v14, v23) -> (Field, u32, [Field])
            v27 = array_get v26, index u32 0 -> Field
            constrain v27 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_remove() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v3 = make_array [Field 2, Field 3] : [Field]
            v4 = allocate -> &mut u32
            v5 = allocate -> &mut [Field]
            enable_side_effects v0
            v6 = cast v0 as u32
            v7, v8, v100 = call vector_remove(v6, v3, u32 0) -> (u32, [Field], Field)
            v9 = not v0
            v10 = cast v0 as u32
            v12 = if v0 then v8 else (if v9) v3
            enable_side_effects u1 1
            v15, v16, v101 = call vector_remove(v10, v12, u32 0) -> (u32, [Field], Field)
            v17 = array_get v16, index u32 0 -> Field
            constrain v17 == Field 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else().unwrap();

        // Here [v21, Field 3] is the result of merging the original vector (`[Field 2, Field 3]`)
        // where for v21 it's the merged value.
        assert_ssa_snapshot!(ssa, @"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: Field, v2: Field):
            v5 = make_array [Field 2, Field 3] : [Field]
            v6 = allocate -> &mut u32
            v7 = allocate -> &mut [Field]
            enable_side_effects v0
            v8 = cast v0 as u32
            v11, v12, v13 = call vector_remove(v8, v5, u32 0) -> (u32, [Field], Field)
            v14 = not v0
            v15 = cast v0 as u32
            enable_side_effects u1 1
            v17 = array_get v12, index u32 0 -> Field
            v18 = cast v0 as Field
            v19 = cast v14 as Field
            v20 = mul v18, v17
            v21 = mul v19, Field 2
            v22 = add v20, v21
            v23 = make_array [v22, Field 3] : [Field]
            enable_side_effects v0
            enable_side_effects u1 1
            v24, v25, v26 = call vector_remove(v15, v23, u32 0) -> (u32, [Field], Field)
            v27 = array_get v25, index u32 0 -> Field
            constrain v27 == Field 1
            return
        }
        ");
    }

    #[test]
    fn can_handle_vector_with_zero_size_elements() {
        let src = "
        acir(inline) pure fn main f0 {
            b0(v0: u32):
                v3 = make_array [] : [()]
                v4 = make_array [] : [()]
                v6 = eq v0, u32 4
                jmpif v6 then: b1(), else: b2()
            b1():
                jmp b3(u32 1, v3)
            b2():
                jmp b3(u32 2, v4)
            b3(v1: u32, v2: [()]):
                return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.flatten_cfg().remove_if_else().unwrap();
        assert_ssa_snapshot!(ssa, @"
        acir(inline) pure fn main f0 {
          b0(v0: u32):
            v1 = make_array [] : [()]
            v2 = make_array [] : [()]
            v4 = eq v0, u32 4
            enable_side_effects v4
            v5 = not v4
            enable_side_effects u1 1
            v7 = cast v4 as u32
            v8 = cast v5 as u32
            v10 = unchecked_mul v8, u32 2
            v11 = unchecked_add v7, v10
            v12 = make_array [] : [()]
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_capacity_larger_than_length() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = make_array [v0, u32 2] : [(u32, u32)]
            v5 = allocate -> &mut u32
            v6 = allocate -> &mut [(u32, u32)]
            v8 = lt v2, u32 10
            enable_side_effects v8
            v12, v13 = call vector_push_back(u32 0, v4, v1, u32 4) -> (u32, [(u32, u32)])
            v14 = not v8
            v15 = cast v8 as u32
            v16 = cast v14 as u32
            v17 = unchecked_mul v15, v12
            v18 = unchecked_add v17, v16
            v19 = if v8 then v13 else (if v14) v4
            enable_side_effects u1 1
            v21 = lt v2, v18
            constrain v21 == u1 1, "Index out of bounds"
            v22 = unchecked_mul v2, u32 2
            v23 = array_get v19, index v22 -> u32
            v25 = unchecked_add v22, u32 1
            v26 = array_get v19, index v25 -> u32
            return v23, v26, v18
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_if_else().unwrap();

        let args = vec![Value::u32(5), Value::u32(10), Value::u32(0)];
        let result = ssa.interpret(args).unwrap();
        assert_eq!(result, vec![Value::u32(10), Value::u32(4), Value::u32(1)]);

        let args = vec![Value::u32(5), Value::u32(10), Value::u32(20)];
        let result = ssa.interpret(args).unwrap_err();
        let InterpreterError::ConstrainEqFailed { msg, .. } = result else {
            panic!("Expected a constrain failure on the final vector access");
        };
        assert_eq!(msg, Some("Index out of bounds".to_string()));

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = make_array [v0, u32 2] : [(u32, u32)]
            v5 = allocate -> &mut u32
            v6 = allocate -> &mut [(u32, u32)]
            v8 = lt v2, u32 10
            enable_side_effects v8
            v12, v13 = call vector_push_back(u32 0, v4, v1, u32 4) -> (u32, [(u32, u32)])
            v14 = not v8
            v15 = cast v8 as u32
            v16 = cast v14 as u32
            v17 = unchecked_mul v15, v12
            v18 = unchecked_add v17, v16
            enable_side_effects u1 1
            v20 = array_get v13, index u32 0 -> u32
            v21 = cast v8 as u32
            v22 = cast v14 as u32
            v23 = unchecked_mul v21, v20
            v24 = unchecked_mul v22, v0
            v25 = unchecked_add v23, v24
            v27 = array_get v13, index u32 1 -> u32
            v28 = cast v8 as u32
            v29 = cast v14 as u32
            v30 = unchecked_mul v28, v27
            v31 = unchecked_mul v29, u32 2
            v32 = unchecked_add v30, v31
            v33 = array_get v13, index u32 2 -> u32
            v35 = array_get v13, index u32 3 -> u32
            v36 = make_array [v25, v32, v33, v35] : [(u32, u32)]
            enable_side_effects v8
            enable_side_effects u1 1
            v37 = lt v2, v18
            constrain v37 == u1 1, "Index out of bounds"
            v38 = unchecked_mul v2, u32 2
            v39 = array_get v36, index v38 -> u32
            v40 = unchecked_add v38, u32 1
            v41 = array_get v36, index v40 -> u32
            return v39, v41, v18
        }
        "#);
    }

    // Regression test for an over-read of a `vector_pop_back` result after merging two vectors
    // of unequal capacity. The source program is:
    // ```
    // fn main(choose: bool, do_pop: bool) -> pub Field {
    //     let mut v: [Field] = if choose { [1].as_vector() } else { [2, 3].as_vector() };
    //     if do_pop {
    //         let (new_v, _) = v.pop_back();
    //         v = new_v;
    //     }
    //     if v.len() == 0 { 0 } else { v[0] }
    // }
    // ```
    // With `choose = true` the merged vector has semantic length 1 but backing capacity 2, so
    // popping it yields an empty vector. The merge of the pop result must not over-read it.
    #[test]
    fn merge_vector_pop_back_with_smaller_semantic_length_than_capacity() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v3 = make_array [Field 1] : [Field]
            v4 = not v0
            enable_side_effects v4
            v7 = make_array [Field 2, Field 3] : [Field]
            enable_side_effects u1 1
            v9 = cast v0 as u32
            v10 = cast v4 as u32
            v12 = unchecked_mul v10, u32 2
            v13 = unchecked_add v9, v12
            v14 = if v0 then v3 else (if v4) v7
            enable_side_effects v1
            v16, v17, v18 = call vector_pop_back(v13, v14) -> (u32, [Field], Field)
            v19 = not v1
            enable_side_effects u1 1
            v20 = cast v1 as u32
            v21 = cast v19 as u32
            v22 = unchecked_mul v20, v16
            v23 = unchecked_mul v21, v13
            v24 = unchecked_add v22, v23
            v25 = if v1 then v17 else (if v19) v14
            v27 = eq v24, u32 0
            enable_side_effects v27
            v28 = not v27
            enable_side_effects v28
            v29 = unchecked_mul v27, v28
            constrain v29 == u1 0, "Index out of bounds"
            v31 = array_get v25, index u32 0 -> Field
            enable_side_effects u1 1
            v32 = cast v27 as Field
            v33 = cast v28 as Field
            v34 = mul v33, v31
            return v34
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_if_else().unwrap();

        // choose = true, do_pop = true: the merged vector is `[1]` (length 1), popping it leaves an
        // empty vector, so `v.len() == 0` holds and the result is `0`. This must not fail with an
        // out-of-bounds read on the popped vector.
        let args = vec![Value::bool(true), Value::bool(true)];
        let result = ssa.interpret(args).unwrap();
        assert_eq!(result, vec![Value::field(FieldElement::zero())]);
    }

    // Regression test for https://github.com/noir-lang/noir/issues/10978
    // The remove_if_else pass should panic due to a checked addition overflow
    // when processing arrays with capacity u32::MAX.
    #[test]
    #[should_panic(expected = "Vector capacity overflow")]
    fn regression_10978() {
        // This is the SSA for the Noir program described in the issue,
        // before the remove if-else pass.
        let src = "
       acir(inline) impure fn main f0 {
        b0(v0: u1):
            v2 = call f1() -> [Field; 4294967295]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            v9, v10 = call vector_push_back(u32 4294967295, v5, Field 1) -> (u32, [Field])
            v11, v12 = call vector_push_back(v9, v10, Field 1) -> (u32, [Field])
            enable_side_effects v0
            v13 = not v0
            enable_side_effects u1 1
            v15 = cast v0 as u32
            v16 = cast v13 as u32
            v17 = unchecked_mul v15, v9
            v18 = unchecked_mul v16, v11
            v19 = unchecked_add v17, v18
            v20 = if v0 then v10 else (if v13) v12
            v22, v23 = call black_box(v19, v20) -> (u32, [Field])
            return
        }
        brillig(inline) impure fn void_to_array f1 {
        b0():
            v1 = call void_to_array_oracle() -> [Field; 4294967295]
            return v1
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.remove_if_else();
    }

    // The pass only tracks vector capacities through vector intrinsics and `array_set`,
    // so it cannot recover the size of a vector produced by a `load` (or a non-intrinsic
    // call). Such SSA is not produced by the frontend, but the SSA fuzzer and `noir-ssa`
    // can feed it in. When such a vector reaches an `if_else` merge the size is needed but
    // unavailable; the pass must surface a graceful error rather than panicking.
    #[test]
    fn merge_vector_without_determinable_size_errors() {
        let src = "
        acir(inline) impure fn main f0 {
          b0(v0: u1, v1: &mut [Field]):
            v2 = make_array [] : [Field]
            v3 = load v1 -> [Field]
            v4 = not v0
            v5 = if v0 then v3 else (if v4) v2
            enable_side_effects u1 1
            v6 = array_get v5, index u32 0 -> Field
            constrain v6 == Field 1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let Err(err) = ssa.remove_if_else() else {
            panic!("expected remove_if_else to error on a vector with no determinable size");
        };
        assert!(
            format!("{err}").contains("without a determinable size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn merge_vector_returned_by_black_box_hint() {
        // The hint returns its vector argument unchanged, so the merged vector needs its capacity
        // even though the length passed alongside it is not a constant.
        let src = "
        acir(inline) impure fn main f0 {
          b0(v0: u1, v1: u32):
            v2 = make_array [Field 1, Field 2] : [Field]
            v3 = make_array [Field 3] : [Field]
            v4, v5 = call black_box(v1, v2) -> (u32, [Field])
            v6 = not v0
            v7 = if v0 then v5 else (if v6) v3
            v8 = array_get v7, index u32 1 -> Field
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (_, result) = assert_pass_does_not_affect_execution(
            ssa,
            vec![Value::bool(true), Value::u32(2)],
            |ssa| ssa.remove_if_else().unwrap(),
        );
        assert_eq!(result, Ok(vec![Value::field(2_u128.into())]));
    }

    #[test]
    fn constant_length_of_a_disabled_call_does_not_shrink_its_input_elsewhere() {
        // The `push_back` only runs when `v0` is true, so its constant length says nothing about
        // `v1` when `v0` is false, and the merge still has to cover both elements of `v1`.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [Field 1, Field 2] : [Field]
            enable_side_effects v0
            v4, v5 = call vector_push_back(u32 0, v1, Field 3) -> (u32, [Field])
            v6 = not v0
            enable_side_effects u1 1
            v7 = if v0 then v5 else (if v6) v1
            v8 = array_get v7, index u32 1 -> Field
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (_, result) =
            assert_pass_does_not_affect_execution(ssa, vec![Value::bool(false)], |ssa| {
                ssa.remove_if_else().unwrap()
            });
        assert_eq!(result, Ok(vec![Value::field(2_u128.into())]));
    }

    #[test]
    fn merge_disabled_vector_pop_back_result() {
        // The disabled `pop_back` returns a zeroed vector of capacity 2, which is merged with a
        // vector of capacity 1.
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v1 = make_array [Field 1, Field 2, Field 3] : [Field]
            v2 = make_array [Field 4] : [Field]
            enable_side_effects v0
            v5, v6, v7 = call vector_pop_back(u32 3, v1) -> (u32, [Field], Field)
            v8 = not v0
            enable_side_effects u1 1
            v9 = if v0 then v6 else (if v8) v2
            v10 = array_get v9, index u32 0 -> Field
            return v10
        }
        ";
        for (input, expected) in [(true, 1_u128), (false, 4)] {
            let ssa = Ssa::from_str(src).unwrap();
            let (_, result) =
                assert_pass_does_not_affect_execution(ssa, vec![Value::bool(input)], |ssa| {
                    ssa.remove_if_else().unwrap()
                });
            assert_eq!(result, Ok(vec![Value::field(expected.into())]));
        }
    }
}
