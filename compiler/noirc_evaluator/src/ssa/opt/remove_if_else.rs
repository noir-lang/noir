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
//! at the basic block level into simple ternary operations returning a value, using these IfElse instructions,
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
//! The IfElse instruction is then replaced by these instruction during the remove if-else pass:
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
use crate::ssa::opt::simple_optimization::SimpleOptimizationContext;
use crate::ssa::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Hint, Instruction, Intrinsic},
        types::Type,
        value::{Value, ValueId},
    },
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
                    let mut value_merger =
                        ValueMerger::new(context.dfg, block, &self.vector_sizes, call_stack);

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

                        self.vector_constant_size_override(context.dfg, intrinsic, arguments);

                        let size_change =
                            self.vector_capacity_change(context.dfg, intrinsic, arguments, results);

                        self.change_size(size_change, context);
                    }
                }
                // Track vector sizes through array set instructions
                Instruction::ArraySet { array, .. } => {
                    let [result] = context.dfg.instruction_result(instruction_id);
                    self.set_capacity(context.dfg, *array, result, |c| c);
                }
                _ => (),
            }
            Ok(())
        })
    }

    fn change_size(&mut self, size_change: SizeChange, context: &mut SimpleOptimizationContext) {
        match size_change {
            SizeChange::None => (),
            SizeChange::SetTo { old, new } => {
                self.set_capacity(context.dfg, old, new, |c| c);
            }
            SizeChange::Inc { old, new } => {
                self.set_capacity(context.dfg, old, new, |c| {
                    // Checked addition because increasing the capacity must increase it (cannot wrap around or saturate).
                    SemanticLength(c.0.checked_add(1).expect("Vector capacity overflow"))
                });
            }
            SizeChange::Dec { old, new } => {
                // We use a saturating sub here as calling `pop_front` or `pop_back` on a zero-length vector
                // would otherwise underflow.
                self.set_capacity(context.dfg, old, new, |c| SemanticLength(c.0.saturating_sub(1)));
            }
            SizeChange::Many(changes) => {
                for change in changes {
                    self.change_size(change, context);
                }
            }
        }
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
        if !matches!(dfg.type_of_value(new), Type::Vector(_)) {
            return;
        }
        let capacity = self.get_or_find_capacity(dfg, old);
        self.vector_sizes.insert(new, f(capacity));
    }

    /// Make sure the vector capacity is recorded.
    fn ensure_capacity(&mut self, dfg: &DataFlowGraph, vector: ValueId) {
        self.set_capacity(dfg, vector, vector, |c| c);
    }

    /// Get the tracked size of array/vectors, or retrieve (and track) it for arrays.
    fn get_or_find_capacity(&mut self, dfg: &DataFlowGraph, value: ValueId) -> SemanticLength {
        match self.vector_sizes.entry(value) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                if let Some(length) = dfg.try_get_vector_capacity(value) {
                    return *entry.insert(length);
                }
                // For non-constant vectors we can't tell the size, which would mean we can't merge it.
                let dbg_value = &dfg[value];
                unreachable!("ICE: No size for vector {value} = {dbg_value:?}")
            }
        }
    }

    /// If we have already determined a constant for the vector length, we can override the backing capacity
    /// of the vector contents. There is no need to use the backing capacity if we have already determined the actual length of the vector.
    /// In these situations, using the capacity over the vector length would require laying down more instructions to handle the extra padding
    /// while preventing downstream passes or runtimes from implementing optimizations using the vector length.
    fn vector_constant_size_override(
        &mut self,
        dfg: &DataFlowGraph,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
    ) {
        match intrinsic {
            Intrinsic::VectorPushBack
            | Intrinsic::VectorPushFront
            | Intrinsic::VectorInsert
            | Intrinsic::VectorPopBack
            | Intrinsic::VectorRemove
            | Intrinsic::VectorPopFront => {
                if let Some(const_len) = dfg.get_numeric_constant(arguments[0]) {
                    self.vector_sizes.insert(
                        arguments[1],
                        SemanticLength(const_len.try_to_u32().expect("Type should be u32")),
                    );
                }
            }
            Intrinsic::Hint(Hint::BlackBox) => {
                // Try to set the length of any vector argument to be that of the preceding constant.
                let arguments_types =
                    arguments.iter().map(|x| dfg.type_of_value(*x)).collect::<Vec<_>>();

                for (i, argument) in arguments.iter().enumerate().skip(1) {
                    if !matches!(arguments_types[i], Type::Vector(_)) {
                        continue;
                    }
                    assert!(matches!(arguments_types[i - 1], Type::Numeric(_)));
                    if let Some(const_len) = dfg.get_numeric_constant(arguments[i - 1]) {
                        self.vector_sizes.insert(
                            *argument,
                            SemanticLength(const_len.try_to_u32().expect("Type should be u32")),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Find the change to a vector's capacity an instruction would have
    fn vector_capacity_change(
        &self,
        dfg: &DataFlowGraph,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        results: &[ValueId],
    ) -> SizeChange {
        match intrinsic {
            Intrinsic::VectorPushBack | Intrinsic::VectorPushFront | Intrinsic::VectorInsert => {
                // All of these return `Self` (the vector), we are expecting: len, vector = ...
                assert_eq!(results.len(), 2);
                let old = arguments[1];
                let new = results[1];
                assert!(matches!(dfg.type_of_value(old), Type::Vector(_)));
                assert!(matches!(dfg.type_of_value(new), Type::Vector(_)));
                SizeChange::Inc { old, new }
            }

            Intrinsic::VectorPopBack | Intrinsic::VectorRemove => {
                // fn pop_back(self) -> (Self, T)
                // fn remove(self, index: u32) -> (Self, T)
                //
                // These functions return the vector as the result `(len, vector, ...item)`,
                // so the vector is the second result.
                let old = arguments[1];
                let new = results[1];
                assert!(matches!(dfg.type_of_value(old), Type::Vector(_)));
                assert!(matches!(dfg.type_of_value(new), Type::Vector(_)));
                SizeChange::Dec { old, new }
            }

            Intrinsic::VectorPopFront => {
                // fn pop_front(self) -> (T, Self)
                //
                // These functions return the vector as the result `(...item, len, vector)`,
                // so the vector is the last result.
                let old = arguments[1];
                let new = results[results.len() - 1];
                assert!(matches!(dfg.type_of_value(old), Type::Vector(_)));
                assert!(matches!(dfg.type_of_value(new), Type::Vector(_)));
                SizeChange::Dec { old, new }
            }

            Intrinsic::AsVector => {
                assert_eq!(arguments.len(), 1);
                assert_eq!(results.len(), 2);
                let old = arguments[0];
                let new = results[1];
                assert!(matches!(dfg.type_of_value(old), Type::Array(_, _)));
                assert!(matches!(dfg.type_of_value(new), Type::Vector(_)));
                SizeChange::SetTo { old, new }
            }

            Intrinsic::Hint(Hint::BlackBox) => {
                assert_eq!(arguments.len(), results.len());
                let arguments_types =
                    arguments.iter().map(|x| dfg.type_of_value(*x)).collect::<Vec<_>>();
                let results_types =
                    results.iter().map(|x| dfg.type_of_value(*x)).collect::<Vec<_>>();

                assert_eq!(arguments_types, results_types);

                let mut changes = Vec::new();
                for (i, argument) in arguments.iter().enumerate() {
                    if self.vector_sizes.contains_key(argument)
                        && matches!(arguments_types[i], Type::Vector(_))
                    {
                        assert!(matches!(arguments_types[i - 1], Type::Numeric(_)));
                        let new = results[i];
                        changes.push(SizeChange::SetTo { old: *argument, new });
                    }
                }

                SizeChange::Many(changes)
            }

            // These cases don't affect vector capacities
            Intrinsic::AssertConstant
            | Intrinsic::StaticAssert
            | Intrinsic::ApplyRangeConstraint
            | Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::StrAsBytes
            | Intrinsic::BlackBox(_)
            | Intrinsic::AsWitness
            | Intrinsic::IsUnconstrained
            | Intrinsic::DerivePedersenGenerators
            | Intrinsic::ToBits(_)
            | Intrinsic::ToRadix(_)
            | Intrinsic::ArrayRefCount
            | Intrinsic::VectorRefCount
            | Intrinsic::FieldLessThan => SizeChange::None,
        }
    }
}

enum SizeChange {
    None,
    /// Make the size of the new vector equal to the old array.
    SetTo {
        old: ValueId,
        new: ValueId,
    },
    /// Make the size of the new vector equal to old+1.
    Inc {
        old: ValueId,
        new: ValueId,
    },
    /// Make the size of the new vector equal to old-1.
    Dec {
        old: ValueId,
        new: ValueId,
    },
    Many(Vec<SizeChange>),
}

#[cfg(debug_assertions)]
fn remove_if_else_pre_check(func: &Function) {
    // This pass should only run post-flattening.
    super::flatten_cfg::flatten_cfg_post_check(func);

    // We expect to only encounter `IfElse` instructions on array and vector types.
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();

        for instruction_id in instruction_ids {
            if let Instruction::IfElse { then_value, .. } = &func.dfg[*instruction_id] {
                // We generally expect that all the results at this point will be either arrays or vectors,
                // however the flattening makes no guarantee of this: if it needs to merge references or functions
                // it will do so using IfElse. The ValueMerger already returns appropriate RuntimeErrors to point
                // at the problem, so we don't assert this expectation.

                // We do expect that numeric values are not used though.
                let typ = func.dfg.type_of_value(*then_value);
                assert!(
                    !matches!(typ, Type::Numeric(_)),
                    "Numeric values should have been handled during flattening"
                );
            }
        }
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
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{errors::InterpreterError, value::Value},
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
            v17 = eq v11, u32 1
            v18 = not v17
            v19 = add v11, u32 1
            v20 = make_array [v14, v2] : [Field]
            v21 = array_set v20, index v11, value v2
            v22 = array_get v21, index u32 0 -> Field
            v23 = cast v18 as Field
            v24 = cast v17 as Field
            v25 = mul v23, v22
            v26 = mul v24, v14
            v27 = add v25, v26
            v28 = array_get v21, index u32 1 -> Field
            v29 = cast v18 as Field
            v30 = cast v17 as Field
            v31 = mul v29, v28
            v32 = mul v30, v2
            v33 = add v31, v32
            v34 = make_array [v27, v33] : [Field]
            constrain v27 == Field 1
            return
        }
        ");
    }

    #[test]
    fn merge_vector_with_vector_push_front() {
        let src = "
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
        acir(inline) impure fn main f0 {
            b0(v0: u32):
                v3 = make_array [] : [()]
                v4 = make_array [] : [()]
                v6 = eq v0, u32 4
                jmpif v6 then: b1, else: b2
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
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
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
            v22 = array_get v13, index u32 1 -> u32
            v23 = make_array [v20, v22] : [(u32, u32)]
            enable_side_effects v8
            enable_side_effects u1 1
            v24 = lt v2, v18
            constrain v24 == u1 1, "Index out of bounds"
            v25 = unchecked_mul v2, u32 2
            v26 = array_get v23, index v25 -> u32
            v27 = unchecked_add v25, u32 1
            v28 = array_get v23, index v27 -> u32
            return v26, v28, v18
        }
        "#);
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
}
