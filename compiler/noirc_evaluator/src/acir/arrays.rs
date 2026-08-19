//! Array handling in ACIR.
//!
//! This modules how Noir's SSA array semantics are lowered into ACIR's flat memory model.
//! Arrays in SSA can appear as constants or dynamically allocated blocks.
//! Our responsibility here is to preserve correctness while ensuring memory access is efficient.
//!
//! ## Design
//!
//! ACIR does not have a first-class array type. Instead, all arrays are
//! represented as contiguous regions in linear memory, identified by a
//! [`BlockId`]. This module provides helpers for translating SSA array
//! operations into ACIR memory reads and writes.
//!
//! ACIR generation use two different array types for representing arrays:
//!
//! [Constant arrays][AcirValue::Array]
//!   - A known sequence of element [`AcirValue`]s. The individual values may be witnesses (e.g. a
//!     function parameter array), but the array's length and structure are known at compile time.
//!   - Reads and writes may be folded into an [`AcirValue`] where possible.
//!   - Useful for optimization (e.g., constant element lookups do not require laying down opcodes)
//!
//! [Dynamic arrays][AcirValue::DynamicArray]
//!   - Referenced by a [unique identifier][BlockId]
//!   - Must be explicitly initialized using an [opcode][acvm::acir::circuit::opcodes::Opcode::MemoryInit]
//!   - Reads and writes must lower to at least an explicit [memory opcode][acvm::acir::circuit::opcodes::Opcode::MemoryOp].
//!   - Required once an array is accessed at a dynamic index, or written under a predicate. A
//!     function parameter array starts as a constant array and is only promoted to a dynamic
//!     array (lazily, by [`Context::ensure_array_is_initialized`]) when such an access occurs;
//!     a parameter only read at constant indices never needs a memory block.
//!
//! ### Array Flattening
//!
//! ACIR memory is flat, while SSA arrays may be multi-dimensional or
//! contain elements of varying size (we refer to these are non-homogenous arrays).
//! To reconcile this, each element's "flattened index" is computed relative to the array’s base pointer.
//! In some cases this requires consulting a side "element type sizes"
//! array to calculate offsets when elements have a non-homogenous layout.
//!
//! The following Noir program:
//! ```noir
//! struct Bar {
//!     inner: [Field; 3],
//! }
//! struct Foo {
//!     a: Field,
//!     b: [Field; 3],
//!     bar: Bar,
//! }
//! fn main(x: [Foo; 4]) -> pub [Field; 3] {
//!     x[3].bar.inner
//! }
//! ```
//! Will produce the following SSA:
//! ```text
//! acir(inline) pure fn main f0 {
//!   b0(v0: [(Field, [Field; 3], [Field; 3]); 4]):
//!     v2 = array_get v0, index u32 11 -> [Field; 3]
//!     return v2
//! }
//! ```
//! In the SSA above we see that we have an index of `11`. However, with a flat memory
//! the true starting index of `x[3].bar.inner` is `25`.
//!
//! To determine which field within the array we are attempting to access, we use an element type sizes array that stores
//! the flat starting index for each SSA field. Since tuples are flattened in SSA, each tuple field gets its own entry.
//! For an array like `[(Field, [Field; 3], [Field; 3]); 4]`, the `element_type_sizes` array would be:
//!
//! [0, 1, 4, 7, 8, 11, 14, 15, 18, 21, 22, 25]
//!
//! Where:
//! - Indices 0-2: element 0's three fields (Field at 0, [Field; 3] at 1, [Field; 3] at 4)
//! - Indices 3-5: element 1's three fields (Field at 7, [Field; 3] at 8, [Field; 3] at 11)
//! - Indices 6-8: element 2's three fields (Field at 14, [Field; 3] at 15, [Field; 3] at 18)
//! - Indices 9-11: element 3's three fields (Field at 21, [Field; 3] at 22, [Field; 3] at 25)
//!
//! We use the SSA index directly to look up `element_type_sizes[index]` to get the flat starting offset.
//! We then use the resulting type to increment the index appropriately and fetch every element.
//!
//! This element type sizes array is dynamic as we still need to access it based upon the index which itself can be dynamic.
//! The module will also attempt to not create this array when possible (e.g., when we have a simple homogenous array).
//!
//! ### Side effects and Predication
//!
//! This module uses the [side-effects predicate][Context::predicate] to guard
//! array operations that may not always be executed. This variable acts as a predicate.
//!
//! The goal is to preserve SSA semantics where some array operations are dominated by a branch condition.
//! We predicate the following:
//!
//! #### Index Predication
//!
//! Array indices themselves are guarded by the side-effect predicate.
//! If an SSA array operation is executed at runtime, then we must ensure any arithmetic that computes the index
//! and any memory reads/writes implied by that index are safe even when the predicate is false.
//! The only array operations not left to runtime are those with safe indices (constant index under the array length).
//!
//! To achieve this we compute a predicated index value (`predicate_index`) with the formula:
//! ```text
//! predicate_index = predicate * index + (1 - predicate) * offset
//! ```
//! where `offset` is a safe fallback index (chosen so the element type at that
//! offset matches the requested element type).
//! The offset is necessary to match the correct result type for array reads.
//!
//! #### Writes
//!
//! When the predicate is not a constant, instead of actually overwriting memory, we compute a "dummy value".
//! The dummy value is fetched from the same array at the requested `predicate_index`.
//! The store value of an array write is then converted from a `store_value` to `predicate * store_value + (1 - predicate) * dummy`
//! This ensures the memory remains unchanged when the write is disabled. In the case of a false predicate, the value stored will be itself.
//!
//! #### Reads
//!
//! If we perform an array read under a false predicate we will read from `offset`. As arrays are
//! not always homogenous, an arbitrary fallback slot could hold a value wider than the read's
//! declared result type. `offset` is therefore chosen per read as the flat slot of the element
//! field whose type matches the result type (see [`Context::compute_offset`]), so the dummy
//! value a disabled read returns always fits its declared type. The surrounding predication
//! discards that dummy value.
//!
//! ### Zero-Length Arrays
//!
//! Arrays of length 0 are valid in the SSA but must never generate ACIR
//! memory operations as they may produce runtime errors. These operations are special cased to always fail with an
//! index out of bounds error (with respect to side effects) and ensures they do not produce illegal memory accesses.
use acvm::acir::brillig::lengths::{
    ElementTypesLength, ElementsFlattenedLength, FlattenedLength, SemanticLength,
};
use acvm::acir::{circuit::opcodes::BlockType, native_types::Witness};
use acvm::{FieldElement, acir::AcirField, acir::circuit::opcodes::BlockId};
use iter_extended::vecmap;
use itertools::Itertools;
use std::rc::Rc;

use crate::acir::types::flat_element_types;
use crate::brillig::assert_u32;
use crate::errors::{InternalError, RuntimeError};
use crate::ssa::ir::types::NumericType;
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Instruction, InstructionId},
    types::Type,
    value::{Value, ValueId},
};

use super::{
    AcirVar, Context,
    side_effects::{PredicateNotNeeded, StaleReadIsSafe},
    types::{AcirDynamicArray, AcirValue},
};

impl Context<'_> {
    /// Allocate a fresh, unique [`BlockId`] for a memory block.
    fn next_block_id(&mut self) -> BlockId {
        let block_id = BlockId::new(self.max_block_id);
        self.max_block_id += 1;
        block_id
    }

    /// Get the `BlockId` corresponding to the `ValueId`
    /// If there is no matching `BlockId`, we create a new one.
    pub(super) fn block_id(&mut self, value: ValueId) -> BlockId {
        *self.memory_blocks.entry(value).or_insert_with(|| {
            let block_id = BlockId::new(self.max_block_id);
            self.max_block_id += 1;
            block_id
        })
    }

    pub(crate) fn return_data_block_id(&mut self) -> BlockId {
        self.return_data_block_id.unwrap_or_else(|| {
            let block_id = self.next_block_id();
            self.return_data_block_id = Some(block_id);
            block_id
        })
    }

    /// Get the next [`BlockId`] for the internal element type sizes array.
    /// This is useful for referencing information that can
    /// only be accessed dynamically, such as the type structure
    /// of non-homogenous arrays.
    fn type_sizes_block_id(&mut self, value: ValueId) -> BlockId {
        *self.element_type_sizes_blocks.entry(value).or_insert_with(|| {
            let block_id = BlockId::new(self.max_block_id);
            self.max_block_id += 1;
            block_id
        })
    }

    pub(super) fn initialize_databus(
        &mut self,
        witnesses: &Vec<Witness>,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        // Initialize return_data using provided witnesses
        if self.data_bus.return_data.is_some() {
            assert!(!witnesses.is_empty(), "return data cannot be empty");
            let block_id = self.return_data_block_id();
            let already_initialized = self.initialized_arrays.contains(&block_id);
            if !already_initialized {
                // We hijack ensure_array_is_initialized() because we want the return data to use the return value witnesses,
                // but the databus contains the computed values instead, that have just been asserted to be equal to the return values.
                // We do not use initialize_array either for the case where a constant value is returned.
                // In that case, the constant value has already been assigned a witness and the returned acir vars will be
                // converted to it, instead of the corresponding return value witness.
                self.acir_context.initialize_return_data(block_id, witnesses.to_owned());
            }
        }

        // Initialize call_data
        let call_data_arrays: Vec<ValueId> =
            self.data_bus.call_data.iter().map(|cd| cd.array_id).collect();
        for call_data_array in call_data_arrays {
            self.ensure_array_is_initialized(call_data_array, dfg)?;
        }
        Ok(())
    }

    /// Handles an `ArrayGet` or `ArraySet` instruction.
    /// To set an index of the array (and create a new array in doing so), pass `Some(value)` for
    /// `store_value`. To just retrieve an index of the array, pass `None` for `store_value`.
    pub(super) fn handle_array_operation(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        // Pass the instruction between array methods rather than the internal fields themselves
        let (array, index, store_value, mutable) = match dfg[instruction] {
            Instruction::ArrayGet { array, index } => (array, index, None, false),
            Instruction::ArraySet { array, index, value, mutable } => {
                (array, index, Some(value), mutable)
            }
            _ => {
                return Err(InternalError::Unexpected {
                    expected: "Instruction should be an ArrayGet or ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into());
            }
        };

        if self.handle_zero_length_array(array, dfg, instruction)? {
            return Ok(());
        }

        if self.handle_constant_index_wrapper(instruction, dfg, array, index, store_value)? {
            return Ok(());
        }

        if self.handle_disabled_array_operation(instruction, dfg, array, index, store_value)? {
            return Ok(());
        }

        let array_typ = dfg.type_of_value(array);
        // A disabled `array_set` writes the read-back dummy value to the very slots it was read
        // from, leaving memory unchanged whatever those slots' types are — there is nothing for
        // a fallback offset to protect, so only reads need one.
        let offset = if store_value.is_none() {
            self.compute_offset(instruction, dfg, &array_typ)
        } else {
            0
        };
        let (new_index, new_value) =
            self.convert_array_operation_inputs(array, dfg, index, store_value, offset)?;

        if let Some(new_value) = new_value {
            self.array_set(instruction, new_index, new_value, dfg, mutable)?;
        } else {
            self.array_get(instruction, array, new_index, dfg)?;
        }

        Ok(())
    }

    /// Resolves an array operation whose side-effects predicate is statically false.
    ///
    /// When the predicate is a compile-time zero the access is on a branch that is known to be
    /// disabled, so the runtime memory-op path would only lay down a predicated read/write whose
    /// index the predicate gates down to a constant `0`. For a never-written block that leaves a
    /// constant, in-bounds read that could not be resolved by [`Self::handle_constant_index`] (the
    /// SSA index is not a numeric constant, e.g. an offset multiplication that overflowed its type),
    /// so it should be resolved here instead. A disabled read yields a don't-care value (zeroed to
    /// match its result type, which the surrounding predication masks to zero anyway) and a disabled
    /// write leaves the array unchanged.
    ///
    /// Reads at a statically safe index are excluded: the predicate in scope is not necessarily
    /// theirs, so their value is not a don't-care. See the comment on that check below.
    ///
    /// # Returns
    /// `true` if the operation was resolved as disabled
    /// `false` if the predicate is not statically false, or does not belong to this instruction
    fn handle_disabled_array_operation(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array: ValueId,
        index: ValueId,
        store_value: Option<ValueId>,
    ) -> Result<bool, RuntimeError> {
        // The side-effects predicate is only this instruction's own for the instructions
        // [`crate::ssa::opt::remove_enable_side_effects`] fences, that is those reporting
        // `Instruction::requires_acir_gen_predicate == true`. A read at a statically safe index
        // reports `false`, so that pass is free to move an `EnableSideEffectsIf` past it and the
        // predicate reaching here can belong to an unrelated region: zeroing such a read would
        // discard a live value, along with every constraint written over it.
        //
        // Resolving it as disabled is also unnecessary. A safe index is in bounds by construction,
        // so it never needs the predicate's fallback to a valid slot and the ordinary path emits
        // exactly the read the program asked for.
        //
        // This check runs before the predicate is inspected: a safe read's outcome here is
        // "not handled" regardless of the predicate's value, so it is not a predicate read.
        if store_value.is_none() && dfg.is_safe_index(index, array) {
            return Ok(false);
        }

        let predicate = self.predicate();
        if !self.acir_context.is_constant_zero(&predicate) {
            return Ok(false);
        }

        let value = if store_value.is_some() {
            self.convert_value(array, dfg)
        } else {
            let [result] = dfg.instruction_result(instruction);
            self.array_zero_value(&dfg.type_of_value(result))?
        };
        self.define_result(dfg, instruction, value);
        Ok(true)
    }

    /// For 0-length arrays and vectors, even the disabled memory operations would cause runtime failures.
    /// Set the result to a zero value that matches the type then bypass the rest of the operation,
    /// leaving an assertion that the side effect variable must be false.
    ///
    /// # Returns
    /// `true` if we have a zero length array
    /// `false` if we do not have a zero length array
    fn handle_zero_length_array(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
        instruction: InstructionId,
    ) -> Result<bool, RuntimeError> {
        if !self.has_zero_length(array, dfg) {
            return Ok(false);
        }

        // Zero result.
        let result_ids = dfg.instruction_results(instruction);
        for result_id in result_ids {
            let res_typ = dfg.type_of_value(*result_id);
            let zero_value = self.array_zero_value(&res_typ)?;
            self.ssa_values.insert(*result_id, zero_value);
        }
        // Make sure this code is disabled, or fail with "Index out of bounds".
        let msg = "Index out of bounds, array has size 0".to_string();
        let predicate = self.predicate();
        self.acir_context.assert_zero_var(predicate, msg)?;
        Ok(true)
    }

    /// Attempts a compile-time read/write from an array.
    ///
    /// This relies on all previous operations on this array being done at known indices so that the `AcirValue` at each
    /// position is known (even if the value of this `AcirValue` is unknown). This can then be done only for
    /// `AcirValue::Array` as an `AcirValue::DynamicArray` has been mutated at an unknown index.
    ///
    /// # Returns
    /// `true` if we performed a compile-time read/write
    /// `false` if we did not perform a compile-time read/write
    fn handle_constant_index_wrapper(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array: ValueId,
        index: ValueId,
        store_value: Option<ValueId>,
    ) -> Result<bool, RuntimeError> {
        match self.convert_value(array, dfg) {
            AcirValue::Var(acir_var, _) => {
                Err(RuntimeError::InternalError(InternalError::Unexpected {
                    expected: "an array value".to_string(),
                    found: format!("{acir_var:?}"),
                    call_stack: self.acir_context.get_call_stack(),
                }))
            }
            AcirValue::Array(array_value) => {
                // `AcirValue::Array` supports reading/writing to constant indices at compile-time in some cases.
                if let Some(constant_index) = self.constant_index(index, dfg)? {
                    let store = store_value.map(|value| self.convert_value(value, dfg));
                    let resolved = self.handle_constant_index(
                        instruction,
                        dfg,
                        array_value,
                        constant_index,
                        store,
                    )?;
                    // A compile-time read at an index that is not statically safe reports
                    // `requires_acir_gen_predicate = true`, yet resolves optimistically
                    // without consulting the predicate: if the predicate were false the
                    // result is a don't-care that downstream predication masks anyway.
                    if resolved && store_value.is_none() && !dfg.is_safe_index(index, array) {
                        self.predicate_not_needed(
                            PredicateNotNeeded::ConstantIndexResolvedAtCompileTime,
                        );
                    }
                    Ok(resolved)
                } else {
                    Ok(false)
                }
            }
            AcirValue::DynamicArray(_) => {
                // We do not perform any compile-time reads/writes to dynamic arrays as we'd need to promote this into
                // a regular array by reading all of its elements. It's then better to defer to the dynamic index
                // codepath so we just issue a single read/write.
                Ok(false)
            }
        }
    }

    /// The compile-time value of an array index, if it has one.
    ///
    /// An SSA numeric constant is the obvious case, but ACIR gen tracks constants of its own which
    /// SSA cannot see: `constrain x == c` rewrites `x`'s [`AcirVar`] to the constant `c`
    /// (see `AcirContext::assert_eq_var`), and the arithmetic layered on top of it then folds in
    /// the ACIR expression algebra. An index that is constant only in that second sense must still
    /// take the compile-time path — the dynamic memory-op path would lay down a read at a constant
    /// index on a block that has never been written, which is a fully determined read that ACIR gen
    /// is required to fold (see `assert_constant_reads_are_folded`).
    ///
    /// That second kind of constant is only trustworthy as an index while it stays inside the range
    /// of an index type. ACIR folds over the field, so an expression such as `unchecked_mul u32
    /// 3000000000, u32 2` yields `6000000000` where the `u32` index it stands for is that value
    /// wrapped; the two agree exactly when the folded value fits a `u32`. A wider value is left for
    /// the paths that do not have to interpret it as a slot — a disabled access, or a runtime
    /// bounds failure.
    fn constant_index(
        &mut self,
        index: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<Option<FieldElement>, RuntimeError> {
        if let Some(constant_index) = dfg.get_numeric_constant(index) {
            return Ok(Some(constant_index));
        }
        let index_var = self.convert_numeric_value(index, dfg)?;
        let constant_index = self.acir_context.var_to_expression(index_var)?.to_const().copied();
        Ok(constant_index.filter(|index| index.try_to_u32().is_some()))
    }

    /// See [`Self::handle_constant_index_wrapper`]
    fn handle_constant_index(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array: imbl::Vector<AcirValue>,
        index: FieldElement,
        store_value: Option<AcirValue>,
    ) -> Result<bool, RuntimeError> {
        let array_size: usize = array.len();
        let index = match index.try_to_u32() {
            Some(index_const) => index_const as usize,
            None => {
                let call_stack = self.acir_context.get_call_stack();
                return Err(RuntimeError::TypeConversion {
                    from: "array index".to_string(),
                    into: "u32".to_string(),
                    call_stack,
                });
            }
        };

        if index >= array_size {
            return Ok(false);
        }

        if let Some(store_value) = store_value {
            let predicate = self.predicate();
            let side_effects_always_enabled = self.acir_context.is_constant_one(&predicate);

            if side_effects_always_enabled {
                // If we know that this write will always occur then we can perform it at compile time.
                let value = AcirValue::Array(array.update(index, store_value));
                self.define_result(dfg, instruction, value);
                Ok(true)
            } else if contains_dynamic_array(&store_value) || contains_dynamic_array(&array[index])
            {
                // The predicated value mixes the store value with the existing element as a dummy
                // (see below). We can only do this in-place while both are plain values; if either
                // side holds a nested dynamic array we'd have to read it back out of a memory block,
                // so we defer the whole operation to the runtime memory-op path instead.
                Ok(false)
            } else {
                // A predicate is active, but the index is a known in-bounds constant, so we can still
                // resolve the write at compile time. The existing element acts as the dummy value and
                // the stored element becomes `predicate * value + (1 - predicate) * dummy`: unchanged
                // when the predicate is false. Folding it into the `AcirValue::Array` this way avoids
                // initializing a memory block purely to read that dummy back out.
                let predicated_value =
                    self.convert_array_set_store_value(&store_value, &array[index])?;
                let value = AcirValue::Array(array.update(index, predicated_value));
                self.define_result(dfg, instruction, value);
                Ok(true)
            }
        } else {
            // If the index is not out of range, we can optimistically perform the read at compile time
            // as if the predicate were true. If the predicate were to resolve to false then
            // the result should not affect the rest of circuit execution.
            let value = array[index].clone();
            self.define_result(dfg, instruction, value);
            Ok(true)
        }
    }

    /// For an `ArrayGet`, finds the flat slot offset of the element field whose type matches
    /// the result: the read targets a field of the element, and that same field also exists in
    /// element 0, at this offset, with exactly the result's layout.
    ///
    /// [`Self::get_flattened_index`] biases the predicate-gated index by this offset
    /// (`offset * (1 - predicate)`), so under a false predicate the read lands on that field of
    /// element 0 and is type-compatible by construction: no leaf of the result can end up
    /// holding a value wider than its declared type. The offset must be in flat slot units, the
    /// units of the index it biases: an item ordinal diverges from it as soon as a multi-slot
    /// field precedes the matched one, sending the fallback read to slots of unrelated types.
    ///
    /// A match always exists for SSA generated from Noir source: an `array_get`'s result is
    /// one of the element's fields. No match is an ICE (reachable only through hand-written
    /// SSA, which the SSA validator currently accepts).
    ///
    /// cf. <https://github.com/noir-lang/noir/pull/4971>
    fn compute_offset(
        &self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array_typ: &Type,
    ) -> usize {
        let (Type::Array(element_types, _) | Type::Vector(element_types)) = array_typ else {
            unreachable!("ICE: array_get must operate on an array or vector, got {array_typ}")
        };

        let [result] = dfg.instruction_result(instruction);
        let result_type = dfg.type_of_value(result);

        let mut offset = 0;
        for typ in element_types.iter() {
            if *typ == *result_type {
                return offset;
            }
            offset += typ.flattened_size().0 as usize;
        }
        unreachable!(
            "ICE: array_get result type {result_type} is not a field of the array element type ({array_typ})"
        )
    }

    /// Sets up the inputs for an `ArrayGet` / `ArraySet` instruction.
    ///
    /// Returns the flat memory index to read/write at and, for `ArraySet`, the
    /// (predicated) value to store.
    ///
    /// [`Self::get_flattened_index`] gates the returned index by the side-effects predicate
    /// where necessary and biases the disabled-branch fallback to `offset`, so the dummy value
    /// a disabled read returns is type-compatible with the read's result type
    /// (see [`Self::compute_offset`]).
    fn convert_array_operation_inputs(
        &mut self,
        array_id: ValueId,
        dfg: &DataFlowGraph,
        index: ValueId,
        store_value: Option<ValueId>,
        offset: usize,
    ) -> Result<(AcirVar, Option<AcirValue>), RuntimeError> {
        let array_typ = dfg.type_of_value(array_id);

        let shift = ElementTypeSizesArrayShift::None;
        let index_var = self.convert_numeric_value(index, dfg)?;
        let gating = if dfg.is_safe_index(index, array_id) {
            IndexGating::Safe
        } else {
            IndexGating::Gated { fallback_offset: offset }
        };
        let index_var =
            self.get_flattened_index(&array_typ, array_id, index_var, dfg, gating, shift)?;

        let predicate = if store_value.is_none() && dfg.is_safe_index(index, array_id) {
            self.out_of_scope_predicate(StaleReadIsSafe::OnlyToSkipGating)
        } else {
            self.predicate()
        };

        // Side-effects are always enabled so we do not need to do any predication
        if self.acir_context.is_constant_one(&predicate) {
            let store_value = store_value.map(|store| self.convert_value(store, dfg));
            return Ok((index_var, store_value));
        }

        let new_value = store_value
            .map(|store| self.predicated_store_value(store, dfg, array_id, index_var))
            .transpose()?;

        Ok((index_var, new_value))
    }

    /// When there is a predicate, the store value is predicate*value + (1-predicate)*dummy, where dummy is the value of the array at the requested index.
    /// It is a dummy value because in the case of a false predicate, the value stored at the requested index will be itself.
    fn predicated_store_value(
        &mut self,
        store: ValueId,
        dfg: &DataFlowGraph,
        array_id: ValueId,
        mut dummy_predicate_index: AcirVar,
    ) -> Result<AcirValue, RuntimeError> {
        let store_value = self.convert_value(store, dfg);
        let store_type = dfg.type_of_value(store);
        // Reading the dummy value is the first memory access to the array under a predicate, so the
        // backing block is initialized lazily here.
        let block_id = self.ensure_array_is_initialized(array_id, dfg)?;
        // We must setup the dummy value to match the type of the value we wish to store
        let dummy = self.array_get_value(&store_type, block_id, &mut dummy_predicate_index)?;
        self.convert_array_set_store_value(&store_value, &dummy)
    }

    fn convert_array_set_store_value(
        &mut self,
        store_value: &AcirValue,
        dummy_value: &AcirValue,
    ) -> Result<AcirValue, RuntimeError> {
        match (store_value, dummy_value) {
            (AcirValue::Var(store_var, typ), AcirValue::Var(dummy_var, _)) => {
                let predicate = self.predicate();
                let true_pred = self.acir_context.mul_var(*store_var, predicate)?;
                let one = self.acir_context.add_constant(FieldElement::one());
                let not_pred = self.acir_context.sub_var(one, predicate)?;
                let false_pred = self.acir_context.mul_var(not_pred, *dummy_var)?;
                // predicate*value + (1-predicate)*dummy
                let new_value = self.acir_context.add_var(true_pred, false_pred)?;
                Ok(AcirValue::Var(new_value, *typ))
            }
            (AcirValue::Array(values), AcirValue::Array(dummy_values)) => {
                let mut elements = imbl::Vector::new();

                assert_eq!(
                    values.len(),
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );
                for (val, dummy_val) in values.iter().zip_eq(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (
                AcirValue::DynamicArray(AcirDynamicArray { block_id, len, value_types, .. }),
                AcirValue::Array(dummy_values),
            ) => {
                let dummy_values = dummy_values
                    .into_iter()
                    .map(|val| val.clone().flatten())
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .map(|(var, typ)| AcirValue::Var(var, typ))
                    .collect::<Vec<_>>();

                assert_eq!(
                    len.to_usize(),
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );

                let values: Vec<_> = self
                    .read_dynamic_array(*block_id, *len, value_types)
                    .collect::<Result<_, _>>()?;
                let mut elements = imbl::Vector::new();
                for (val, dummy_val) in values.iter().zip_eq(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, &dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (_, AcirValue::DynamicArray(_)) => {
                unimplemented!("ICE: setting a dummy dynamic array not supported");
            }
            _ => {
                unreachable!("ICE: The store value and dummy value must match");
            }
        }
    }

    /// Returns the acir value at the provided databus offset
    fn get_from_call_data(
        &mut self,
        offset: &mut AcirVar,
        call_data_block: BlockId,
        typ: &Type,
    ) -> Result<AcirValue, RuntimeError> {
        match typ {
            Type::Numeric(_) => self.array_get_value(typ, call_data_block, offset),
            Type::Array(arc, len) => {
                let mut result = imbl::Vector::new();
                for _i in 0..len.0 {
                    for sub_type in arc.iter() {
                        let element = self.get_from_call_data(offset, call_data_block, sub_type)?;
                        result.push_back(element);
                    }
                }
                Ok(AcirValue::Array(result))
            }
            _ => unimplemented!("Unsupported type in databus"),
        }
    }

    /// Generates a read opcode for the array.
    ///
    /// `var_index` is already predicated: on a disabled branch it is gated and biased to the
    /// type-matching fallback offset (see [`Self::compute_offset`]), so the value read always
    /// fits the declared result type.
    fn array_get(
        &mut self,
        instruction: InstructionId,
        array: ValueId,
        var_index: AcirVar,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        let [result] = dfg.instruction_result(instruction);
        let res_typ = dfg.type_of_value(result);
        let value = self.load_array_value(array, var_index, &res_typ, dfg)?;
        self.define_result(dfg, instruction, value);
        Ok(())
    }

    /// Loads a value either from call-data bus or from memory.
    fn load_array_value(
        &mut self,
        array: ValueId,
        mut var_index: AcirVar,
        res_typ: &Type,
        dfg: &DataFlowGraph,
    ) -> Result<AcirValue, RuntimeError> {
        // Get operations to call-data parameters are replaced by a get to the call-data-bus array
        let call_data_info = self
            .data_bus
            .call_data
            .iter()
            .find_map(|cd| cd.index_map.get(&array).map(|idx| (cd.array_id, *idx)));
        if let Some((array_id, bus_index)) = call_data_info {
            // Get the length of the array we want to read:
            let array_typ = dfg.type_of_value(array);
            let flattened_len = array_typ.flattened_size();
            // Get the total call_data array length
            let call_data_typ = dfg.type_of_value(array_id);
            let call_data_len = call_data_typ.flattened_size();
            let is_last_in_call_data =
                bus_index + flattened_len.0 as usize == call_data_len.0 as usize;

            // Check index for out of bounds in the call_data because
            // the databus aggregates them into the call_data array.
            // This is not needed when we access the last element, because
            // we can benefit from the out-of-bound on call data.
            if !is_last_in_call_data {
                let length_var =
                    self.acir_context.add_constant(FieldElement::from(i128::from(flattened_len.0)));
                // Compute out-of-bounds value:
                let in_bound = self.acir_context.less_than_var(var_index, length_var, 32)?;
                // Add the out-of-bounds check:
                let assert_message = "Index out of bounds".to_string();
                let one = self.acir_context.add_constant(FieldElement::one());
                let message = self.acir_context.generate_assertion_message_payload(assert_message);
                self.acir_context.assert_eq_var(in_bound, one, Some(message))?;
            }

            let call_data_block = self.ensure_array_is_initialized(array_id, dfg)?;
            let bus_index = self.acir_context.add_constant(FieldElement::from(bus_index as i128));
            let mut current_index = self.acir_context.add_var(bus_index, var_index)?;
            self.get_from_call_data(&mut current_index, call_data_block, res_typ)
        } else if res_typ.flattened_size().0 == 0 {
            // Reading a zero-slot value (e.g. an empty nested array like `[u8; 0]`) emits no
            // `MemoryOp` reads, so initializing the source array's block here would leave an
            // orphan `MemoryInit` with no linked use (rejected by `acir_post_check`). There is
            // nothing to read, so skip initialization and return the empty value directly.
            self.array_zero_value(res_typ)
        } else {
            // A non-call-data read is the first access to the array's own memory block, so it is
            // initialized lazily here rather than for every `ArrayGet` (call-data reads are served
            // from the databus block and never touch this one).
            let block_id = self.ensure_array_is_initialized(array, dfg)?;
            self.array_get_value(res_typ, block_id, &mut var_index)
        }
    }

    /// Reads the value of type `ssa_type` at the flattened position `var_index` of the array backed
    /// by `block_id`, emitting a `MemoryOp::Read` per scalar and advancing `var_index` past it.
    pub(super) fn array_get_value(
        &mut self,
        ssa_type: &Type,
        block_id: BlockId,
        var_index: &mut AcirVar,
    ) -> Result<AcirValue, RuntimeError> {
        let one = self.acir_context.add_constant(FieldElement::one());
        match ssa_type {
            Type::Numeric(numeric_type) => {
                // Read the value from the array at the specified index
                let read = self.acir_context.read_from_memory(block_id, var_index)?;

                // Increment the var_index in case of a nested array
                *var_index = self.acir_context.add_var(*var_index, one)?;

                Ok(AcirValue::Var(read, *numeric_type))
            }
            Type::Array(element_types, len) => {
                let mut values = imbl::Vector::new();
                for _ in 0..len.0 {
                    for typ in element_types.as_ref() {
                        values.push_back(self.array_get_value(typ, block_id, var_index)?);
                    }
                }
                Ok(AcirValue::Array(values))
            }
            Type::Reference(reference_type, _) => {
                self.array_get_value(reference_type.as_ref(), block_id, var_index)
            }
            _ => unreachable!("ICE: Expected an array or numeric but got {ssa_type:?}"),
        }
    }

    /// Construct a value with all zero values, which we can use to provide a default value
    /// when we cannot use `array_get_value` because the array length itself is zero, yet
    /// we also don't want a memory operation to fail, because the operation will never
    /// actually run, because we know that the side effect variable is false.
    pub(super) fn array_zero_value(&mut self, ssa_type: &Type) -> Result<AcirValue, RuntimeError> {
        match ssa_type.clone() {
            Type::Numeric(numeric_type) => {
                let zero = self.acir_context.add_constant(FieldElement::zero());
                Ok(AcirValue::Var(zero, numeric_type))
            }
            Type::Array(element_types, len) => {
                let mut values = imbl::Vector::new();
                for _ in 0..len.0 {
                    for typ in element_types.as_ref() {
                        values.push_back(self.array_zero_value(typ)?);
                    }
                }
                Ok(AcirValue::Array(values))
            }
            Type::Vector(_) => Ok(AcirValue::Array(imbl::Vector::new())),
            Type::Reference(reference_type, _) => self.array_zero_value(reference_type.as_ref()),
            Type::Function => {
                unreachable!("ICE: unexpected Function type in array_zero_value")
            }
        }
    }

    /// If `mutate_array` is:
    /// - `true`: Mutate the array directly
    /// - `false`: Copy the array and generates a write opcode on the new array. This is
    ///   generally very inefficient and should be avoided if possible. Currently
    ///   this is controlled by SSA's array set optimization pass.
    fn array_set(
        &mut self,
        instruction: InstructionId,
        mut var_index: AcirVar,
        store_value: AcirValue,
        dfg: &DataFlowGraph,
        mutate_array: bool,
    ) -> Result<(), RuntimeError> {
        // Pass the instruction between array methods rather than the internal fields themselves
        let Instruction::ArraySet { array, .. } = dfg[instruction] else {
            return Err(InternalError::Unexpected {
                expected: "Instruction should be an ArraySet".to_owned(),
                found: format!("Instead got {:?}", dfg[instruction]),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        };

        let [result_id] = dfg.instruction_result(instruction);
        let block_id = self.resolve_array_set_block(array, result_id, dfg, mutate_array)?;

        self.array_set_value(&store_value, block_id, &mut var_index)?;

        let result_value = self.make_array_set_result_value(array, block_id, dfg)?;

        self.define_result(dfg, instruction, result_value);
        Ok(())
    }

    /// Since `array_set` creates a new array, we create a block ID for it:
    /// * if `mutate_array` is `true`, we return the existing initialized `block_id`
    /// * otherwise create a new block ID and copy the existing contents into it
    fn resolve_array_set_block(
        &mut self,
        array: ValueId,
        result: ValueId,
        dfg: &DataFlowGraph,
        mutate_array: bool,
    ) -> Result<BlockId, RuntimeError> {
        if mutate_array {
            let block_id = self.ensure_array_is_initialized(array, dfg)?;
            self.memory_blocks.insert(result, block_id);
            Ok(block_id)
        } else {
            // The copy reads the source through its `AcirValue` (inline contents, or the already
            // initialized block of a `DynamicArray`), so we do not force the source into a memory
            // block here: a constant-only source that is copied and then discarded must not leave
            // an orphaned `MemoryInit` behind.
            let new_block = self.block_id(result);
            self.copy_array(array, new_block, dfg)?;
            Ok(new_block)
        }
    }

    /// Set a value at a specific index in the memory block of an array.
    pub(super) fn array_set_value(
        &mut self,
        value: &AcirValue,
        block_id: BlockId,
        var_index: &mut AcirVar,
    ) -> Result<(), RuntimeError> {
        let one = self.acir_context.add_constant(FieldElement::one());
        match value {
            AcirValue::Var(store_var, _) => {
                // Write the new value into the new array at the specified index
                self.acir_context.write_to_memory(block_id, var_index, store_var)?;
                // Increment the var_index in case of a nested array, that is,
                // if `value` itself was an array, write each item at increasing indexes.
                *var_index = self.acir_context.add_var(*var_index, one)?;
            }
            AcirValue::Array(values) => {
                // The value is an array, with items in the data structure;
                // write them one after the other into the target memory block.
                for value in values {
                    self.array_set_value(value, block_id, var_index)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray {
                block_id: inner_block_id,
                len,
                value_types,
                ..
            }) => {
                // The value is an array with items in a different memory block;
                // read all values from the source memory block into an Array structure,
                // then store that into the target memory block.
                let values = self
                    .read_dynamic_array(*inner_block_id, *len, value_types)
                    .collect::<Result<_, _>>()?;
                self.array_set_value(&AcirValue::Array(values), block_id, var_index)?;
            }
        }
        Ok(())
    }

    /// Construct the [`AcirValue::DynamicArray`] that represents the result of an [`Instruction::ArraySet`].
    ///
    /// In SSA, an array set always yields a new array value (even if the operation
    /// mutates in place). At the ACIR level, this corresponds to a [`AcirValue::DynamicArray`] whose
    /// memory block has already been resolved by [`Self::resolve_array_set_block`].
    ///
    /// # Purpose
    /// - Populates the `value_types` vector. See [`AcirDynamicArray::value_types`] for more information.
    pub(super) fn make_array_set_result_value(
        &mut self,
        array: ValueId,
        block_id: BlockId,
        dfg: &DataFlowGraph,
    ) -> Result<AcirValue, RuntimeError> {
        // Every array has a length in its type, so we fetch that from the SSA IR.
        //
        // A vector's size must be fetched from the SSA value that represents the vector.
        // However, this size is simply the capacity of a vector. The capacity is dependent upon the witness
        // and may contain data for which we want to restrict access. The true vector length is tracked in a
        // a separate SSA value and restrictions on vector indices should be generated elsewhere in the SSA.
        let array_typ = dfg.type_of_value(array);
        let len = self.flattened_size(array, dfg);

        let value_types = flat_element_types(&array_typ);
        if value_types.is_empty() {
            // An element type with zero flattened width (e.g. `str<0>` or `[T; 0]`)
            // forces the whole array to flatten to nothing, so there is no per-element
            // stride to check the length against.
            assert_eq!(len.to_usize(), 0, "zero-width elements imply a zero flattened length");
        } else {
            assert_eq!(len.to_usize() % value_types.len(), 0);
        }

        Ok(AcirValue::DynamicArray(AcirDynamicArray { block_id, len, value_types }))
    }

    /// Initializes the element types sizes array to enable indexing of non-homogenous SSA arrays
    /// in a flat memory environment.
    ///
    /// ACIR memory is flat, while SSA arrays may be multi-dimensional or
    /// contain elements of varying size (we refer to these are non-homogenous arrays).
    /// To reconcile this, each element's "flattened index" is computed relative to the array’s base pointer.
    /// In some cases this requires consulting a side "element type sizes"
    /// array to calculate offsets when elements have a non-homogenous layout.
    ///
    /// See [self] for a more concrete example of how this types sizes array is used.
    pub(super) fn init_element_type_sizes_array(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        supplied_acir_value: Option<AcirValue>,
        dfg: &DataFlowGraph,
        shift: ElementTypeSizesArrayShift,
    ) -> Result<BlockId, RuntimeError> {
        let base_block = self.type_sizes_block_id(array_id);

        // A non-shifted request can reuse a table already initialized for this value as-is.
        if self.initialized_arrays.contains(&base_block)
            && matches!(shift, ElementTypeSizesArrayShift::None)
        {
            return Ok(base_block);
        }

        let table = self
            .element_type_sizes_table(array_typ, array_id, supplied_acir_value, dfg, shift)?
            .as_ref()
            .clone();

        let block = self.reuse_or_init_element_type_sizes(table, base_block)?;

        // Record which block backs this value's helper table. This also keeps the post-ACIR check's
        // set of element-type-sizes blocks complete (it scans these values to forbid writes to them).
        self.element_type_sizes_blocks.insert(array_id, block);
        Ok(block)
    }

    /// Returns the element-type-sizes table (a cumulative prefix-offset array) for a non-homogenous
    /// array or vector, without allocating or initializing any memory block for it.
    ///
    /// This is the computation shared by [`Self::init_element_type_sizes_array`] (which then
    /// materializes the table into a memory block) and the constant-index fast path in
    /// [`Self::get_flattened_index`] (which indexes the table directly at compile time and never
    /// needs a block).
    ///
    /// The table for a given `(value, shift)` is fixed (an SSA value's type and length are
    /// immutable), so it is computed once and cached, keeping repeated constant-index accesses into
    /// the same non-homogenous array (e.g. an unrolled loop) from rebuilding it. The cache is keyed
    /// only by `(array_id, shift)`; `supplied_acir_value` is a shortcut used to size the table on a
    /// miss and is ignored on a hit (it is always the same value's ACIR representation, so it has
    /// the same flattened length as `convert_value(array_id)`).
    fn element_type_sizes_table(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        supplied_acir_value: Option<AcirValue>,
        dfg: &DataFlowGraph,
        shift: ElementTypeSizesArrayShift,
    ) -> Result<Rc<Vec<u32>>, RuntimeError> {
        if let Some(table) = self.element_type_sizes_tables.get(&(array_id, shift)) {
            return Ok(table.clone());
        }

        if !matches!(array_typ, Type::Array(_, _) | Type::Vector(_)) {
            return Err(InternalError::Unexpected {
                expected: "array or vector".to_owned(),
                found: array_typ.to_string(),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        }

        if !matches!(&dfg[array_id], Value::Instruction { .. } | Value::Param { .. }) {
            return Err(InternalError::Unexpected {
                expected: "array or instruction".to_owned(),
                found: format!("{:?}", dfg[array_id]),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        }

        // An instruction/param representing the array means it has been processed previously
        // during ACIR gen. Use that result to recover its flattened length, then apply the
        // requested shift (e.g. one extra element for a vector insert) to size the table.
        let array_acir_value =
            supplied_acir_value.unwrap_or_else(|| self.convert_value(array_id, dfg));
        if !matches!(array_acir_value, AcirValue::Array(_) | AcirValue::DynamicArray(_)) {
            return Err(InternalError::Unexpected {
                expected: "AcirValue::DynamicArray or AcirValue::Array".to_owned(),
                found: format!("{array_acir_value:?}"),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        }
        let flattened_len = flattened_value_size(&array_acir_value);
        let table = Rc::new(calculate_element_type_sizes_array(array_typ, flattened_len, shift));

        self.element_type_sizes_tables.insert((array_id, shift), table.clone());
        Ok(table)
    }

    /// Returns a memory block holding the given element-type-sizes `table`, reusing an existing one
    /// when possible.
    ///
    /// Reuses the smallest already-initialized table that is at least as large and shares `table`'s
    /// prefix (the table is never mutated and is a cumulative prefix-offset array, so reading only
    /// the first entries of a larger table yields the same offsets). Otherwise initializes a new
    /// table into `preferred_block`, or a fresh block when `preferred_block` is already in use (it
    /// may hold this value's smaller, non-shifted table).
    fn reuse_or_init_element_type_sizes(
        &mut self,
        table: Vec<u32>,
        preferred_block: BlockId,
    ) -> Result<BlockId, RuntimeError> {
        if !table.is_empty()
            && let Some((existing, block)) = self.type_sizes_to_blocks.range(table.clone()..).next()
            && existing.starts_with(&table)
        {
            return Ok(*block);
        }

        let block = if self.initialized_arrays.contains(&preferred_block) {
            self.next_block_id()
        } else {
            preferred_block
        };

        // The final array contains the flattened index at each outer array index.
        let init_values = vecmap(table.clone(), |type_size| {
            let var = self.acir_context.add_constant(type_size);
            AcirValue::Var(var, NumericType::NativeField)
        });
        let len = FlattenedLength(assert_u32(init_values.len()));
        self.initialize_array(block, len, Some(AcirValue::Array(init_values.into())))?;

        self.type_sizes_to_blocks.insert(table, block);
        Ok(block)
    }

    /// Read an array and reconstruct its structure based on the SSA type.
    /// For `DynamicArrays` with nested arrays, this preserves the nested structure
    /// instead of returning a flat array.
    pub(super) fn read_array_with_type(
        &mut self,
        array: AcirValue,
        array_typ: &Type,
    ) -> Result<imbl::Vector<AcirValue>, RuntimeError> {
        match array {
            AcirValue::Var(_, _) => unreachable!("ICE: attempting to read a non-array value"),
            //Array are already structured
            AcirValue::Array(vars) => Ok(vars),
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }) => {
                // For vectors/arrays, reconstruct the structure based on the element type
                let element_types = match array_typ {
                    Type::Vector(types) | Type::Array(types, _) => types.as_ref(),
                    _ => unreachable!("ICE: reading array into a non array type"),
                };

                // Calculate how many elements we have (number of outer array elements)
                let element_flat_size: FlattenedLength =
                    element_types.iter().map(|t| t.flattened_size()).sum();
                assert_ne!(element_flat_size.0, 0, "ICE: array elements are empty");
                let num_elements = len / ElementsFlattenedLength::from(element_flat_size);

                let mut result = imbl::Vector::new();
                let mut var_index = self.acir_context.add_constant(FieldElement::zero());
                // Reconstruct each element with its proper structure
                for _ in 0..num_elements.0 {
                    for element_typ in element_types {
                        let element =
                            self.array_get_value(element_typ, block_id, &mut var_index)?;
                        result.push_back(element);
                    }
                }

                Ok(result)
            }
        }
    }

    pub(super) fn copy_array(
        &mut self,
        source: ValueId,
        destination: BlockId,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        let original_array = self.convert_value(source, dfg);

        match original_array {
            AcirValue::Var(_, _) => unreachable!("ICE: attempting to copy a non-array value"),
            array @ AcirValue::Array(_) => {
                let array_len = self.flattened_size(source, dfg);
                Ok(self.initialize_array(destination, array_len, Some(array))?)
            }
            AcirValue::DynamicArray(source) => self.copy_dynamic_array(
                source.block_id,
                destination,
                source.len,
                &source.value_types,
            ),
        }
    }

    pub(super) fn read_dynamic_array(
        &mut self,
        source: BlockId,
        array_len: FlattenedLength,
        value_types: &[NumericType],
    ) -> impl Iterator<Item = Result<AcirValue, RuntimeError>> {
        (0..array_len.to_usize()).map(move |i| {
            let index_var = self.acir_context.add_constant(i);

            let read = self.acir_context.read_from_memory(source, &index_var)?;
            let typ = value_types[i % value_types.len()];

            Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, typ))
        })
    }

    fn copy_dynamic_array(
        &mut self,
        source: BlockId,
        destination: BlockId,
        array_len: FlattenedLength,
        value_types: &[NumericType],
    ) -> Result<(), RuntimeError> {
        let array =
            self.read_dynamic_array(source, array_len, value_types).collect::<Result<_, _>>()?;
        self.initialize_array(destination, array_len, Some(AcirValue::Array(array)))?;
        Ok(())
    }

    /// The gating for an access that addresses whole elements, and so has no field of the element
    /// to fall back on: a disabled branch collapses it to the start of the block.
    ///
    /// A statically safe index is used as-is, so a lowering whose only predicated operand is this
    /// index reads no predicate at all. That acknowledgment is recorded here, where the decision
    /// is made, rather than left to each caller to remember.
    pub(super) fn index_gating_without_fallback(&self, is_safe_index: bool) -> IndexGating {
        if is_safe_index {
            self.predicate_not_needed(PredicateNotNeeded::StaticallySafeIndex);
            IndexGating::Safe
        } else {
            IndexGating::Gated { fallback_offset: 0 }
        }
    }

    /// Convert an SSA array index into a flat ACIR array index.
    ///
    /// ACIR memory is flat, while SSA arrays may be multi-dimensional or
    /// contain elements of varying size (we refer to these as non-homogenous arrays).
    /// To reconcile this, each element's "flattened index" is computed relative to the array’s base pointer.
    /// In some cases this requires consulting a side ["element type sizes"][Self::init_element_type_sizes_array]
    /// array to calculate offsets when elements have a non-homogenous layout.
    ///
    /// For an [`IndexGating::Gated`] index the returned index is gated by the side-effects
    /// predicate and `fallback_offset * (1 - predicate)` is added on top, so that on a disabled
    /// branch the access collapses to `fallback_offset` (see [`IndexGating`]). The bias's
    /// precondition is exactly "the index collapses to `0` under a false predicate", so it is
    /// applied here, on the gated path and nowhere else: an index that is not gated — a safe
    /// index, or a constant resolved through the element-type-sizes table below — stays on its
    /// true slots whatever the predicate is, and must not be biased.
    ///
    /// See [self] for a more concrete example of how flattened indices are computed.
    pub(super) fn get_flattened_index(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        var_index: AcirVar,
        dfg: &DataFlowGraph,
        gating: IndexGating,
        shift: ElementTypeSizesArrayShift,
    ) -> Result<AcirVar, RuntimeError> {
        // For a non-homogenous layout a statically-known, in-bounds index resolves to a fixed
        // flattened offset held in the element-type-sizes table. That offset is independent of the
        // side-effects predicate, so compute the table directly and emit the offset as a constant
        // rather than allocating a memory block and reading the (never-written) table at runtime. We
        // use the original index here rather than the predicated one below, since gating can turn a
        // constant into a witness and hide its value. An out-of-bounds constant index (no table
        // entry) falls through to the runtime path, which defers the bounds failure to execution.
        //
        // This resolved index is in bounds and ungated even when the caller asked for gating
        // ([`DataFlowGraph::is_safe_index`] cannot see it: it holds vector indices to the vector's
        // unknown semantic length, so it is `false` for every vector). The access reads the slots
        // the program asked for, so no fallback bias applies.
        if array_has_constant_element_size(array_typ).is_none()
            && let Some(index) = self
                .acir_context
                .var_to_expression(var_index)?
                .to_const()
                .and_then(|c| c.try_to_u32())
            && let Some(offset) = self
                .element_type_sizes_table(array_typ, array_id, None, dfg, shift)?
                .get(index as usize)
                .copied()
        {
            if matches!(gating, IndexGating::Gated { .. }) {
                self.predicate_not_needed(PredicateNotNeeded::ConstantFlattenedOffset);
            }
            return Ok(self.acir_context.add_constant(offset));
        }

        // Gate the input by the side-effects predicate when the index isn't statically
        // known to be in range. Without this, callers that consume the returned index
        // (memory reads/writes, comparisons, etc.) would fail the ACVM bounds check on
        // a disabled branch with an OOB user-supplied index. `mul_var` constant-folds
        // when the predicate is `0` or `1`, so this is free in those cases.
        let var_index = match gating {
            IndexGating::Safe => var_index,
            IndexGating::Gated { .. } => {
                let predicate = self.predicate();
                self.acir_context.mul_var(var_index, predicate)?
            }
        };

        let flat_index = if let Some(step_size) = array_has_constant_element_size(array_typ) {
            let step_size = self.acir_context.add_constant(step_size);
            self.acir_context.mul_var(var_index, step_size)?
        } else {
            let element_type_sizes =
                self.init_element_type_sizes_array(array_typ, array_id, None, dfg, shift)?;

            self.acir_context.read_from_memory(element_type_sizes, &var_index)?
        };

        // The gated flat index is `0` on a disabled branch; bias it to the fallback slot.
        // `raw_index * predicate + fallback_offset * (1 - predicate)` yields the raw index when
        // the predicate is `1` and `fallback_offset` when it is `0`.
        match gating {
            IndexGating::Gated { fallback_offset } if fallback_offset != 0 => {
                let one = self.acir_context.add_constant(FieldElement::one());
                let predicate = self.predicate();
                let not_pred = self.acir_context.sub_var(one, predicate)?;
                let offset_var = self.acir_context.add_constant(fallback_offset);
                let offset_term = self.acir_context.mul_var(offset_var, not_pred)?;
                Ok(self.acir_context.add_var(flat_index, offset_term)?)
            }
            IndexGating::Safe | IndexGating::Gated { .. } => Ok(flat_index),
        }
    }

    /// Calculate the flattened size of a value.
    ///
    /// For vectors this will be based on the capacity, not semantic length.
    pub(super) fn flattened_size(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
    ) -> FlattenedLength {
        let array_typ = dfg.type_of_value(array);
        if !array_typ.contains_vector_element() {
            array_typ.flattened_size()
        } else {
            match &dfg[array] {
                Value::NumericConstant { .. } => FlattenedLength(1),
                Value::Instruction { .. } | Value::Param { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    flattened_value_size(&array_acir_value)
                }
                _ => {
                    unreachable!("ICE: Unexpected SSA value when computing the vector size");
                }
            }
        }
    }

    /// Check if the array or vector has 0 length.
    ///
    /// This is different from `flattened_size` in that a non-zero length
    /// array containing zero length arrays has zero size, but we can still
    /// access its elements.
    pub(super) fn has_zero_length(&mut self, array: ValueId, dfg: &DataFlowGraph) -> bool {
        if let Type::Array(_, size) = &*dfg.type_of_value(array) {
            size.0 == 0
        } else {
            match &dfg[array] {
                Value::Instruction { .. } | Value::Param { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    match array_acir_value {
                        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => len.0 == 0,
                        AcirValue::Array(values) => values.is_empty(),
                        AcirValue::Var(_, _) => {
                            unreachable!(
                                "ICE: Unexpected ACIR value for array or vector: {array_acir_value:?}"
                            )
                        }
                    }
                }
                other => {
                    unreachable!(
                        "ICE: Unexpected SSA value when computing the vector size: {other:?}"
                    );
                }
            }
        }
    }

    pub(super) fn ensure_array_is_initialized(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<BlockId, RuntimeError> {
        // Use the SSA ID to get or create its block ID
        let block_id = self.block_id(array);

        // Check if the array has already been initialized in ACIR gen
        // if not, we initialize it using the values from SSA
        let already_initialized = self.initialized_arrays.contains(&block_id);
        if !already_initialized {
            let value = &dfg[array];
            match value {
                Value::Instruction { .. } | Value::Param { .. } => {
                    let value = self.convert_value(array, dfg);
                    let len = self.flattened_size(array, dfg);
                    self.initialize_array(block_id, len, Some(value))?;
                }
                _ => {
                    return Err(InternalError::General {
                        message: format!("Array {array} should be initialized"),
                        call_stack: self.acir_context.get_call_stack(),
                    }
                    .into());
                }
            }
        }

        Ok(block_id)
    }

    /// Initializes an array with the given values and caches the fact that we
    /// have initialized this array.
    pub(super) fn initialize_array(
        &mut self,
        array: BlockId,
        len: FlattenedLength,
        value: Option<AcirValue>,
    ) -> Result<(), InternalError> {
        // Record the block as initialized even for a zero-length array so that downstream
        // bookkeeping stays consistent, but emit no `MemoryInit` opcode for it: an empty block has
        // no slots to read or write, so any access to it is out of bounds and the opcode would only
        // ever describe an orphan block.
        if !self.initialized_arrays.insert(array) {
            return Err(InternalError::General {
                message: "Attempted to initialize memory block twice".to_owned(),
                call_stack: self.acir_context.get_call_stack(),
            });
        }
        if len.to_usize() == 0 {
            return Ok(());
        }

        let mut databus = BlockType::Memory;
        for (call_data_id, array_id) in self.data_bus.call_data_array() {
            if self.block_id(array_id) == array {
                databus = BlockType::CallData(call_data_id);
                break;
            }
        }

        self.acir_context.initialize_array(array, len, value, databus)?;
        Ok(())
    }
}

/// How an index is treated on a branch the side-effects predicate disables.
///
/// The two cases are one decision, not two independent knobs: a fallback slot is only reachable
/// because gating collapsed the index to `0` first, so an ungated index has no fallback to speak
/// of. [`Context::get_flattened_index`] is the only place that can tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IndexGating {
    /// The index is statically known to be in range for the access, so it is used as-is: the
    /// access stays on the slots the program asked for whatever the predicate is.
    Safe,
    /// The index is not known to be in range, so it is gated by the predicate — a disabled
    /// branch would otherwise fail the ACVM bounds check on an out-of-bounds user-supplied
    /// index. The gated index is `0`, and `fallback_offset` moves the access from there onto a
    /// slot whose type is compatible with it (see [`Context::compute_offset`]); `0` for an
    /// access that has no such slot to land on and only needs to be in bounds.
    Gated { fallback_offset: usize },
}

/// Represents a shift in the size of the element type sizes array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ElementTypeSizesArrayShift {
    /// No shift is needed.
    None,
    /// The element type sizes array needs to grow by one (semantic length).
    /// This is used for vector insert operations.
    Increase,
}

/// Calculates the element type sizes lookup array for heterogeneous arrays/vectors.
///
/// # Parameters
///
/// * `array_typ` - Type of the array/vector for which we are generating an element types sizes array
/// * `flattened_length` - The total flattened size of the array data. For [`AcirValue::Array`],
///   this is computed via [`flattened_value_size`]. For [`AcirValue::DynamicArray`], this is
///   stored in the `len` field.
/// * `shift` - Extra logical elements to allocate space for (e.g., for growth operations such as vector insert)
///
/// # Returns
///
/// A vector where each entry represents the flattened offset for a field in the composite type.
pub(super) fn calculate_element_type_sizes_array(
    array_typ: &Type,
    flattened_length: FlattenedLength,
    shift: ElementTypeSizesArrayShift,
) -> Vec<u32> {
    let (Type::Array(element_types, _) | Type::Vector(element_types)) = array_typ else {
        panic!("ICE: expected array or vector type");
    };
    if element_types.is_empty() {
        return vec![];
    }

    let element_type_sizes = vecmap(element_types.iter(), |typ| typ.flattened_size());
    let element_flattened_size: FlattenedLength = element_type_sizes.iter().copied().sum();
    let mut non_flattened_elements =
        flattened_length / ElementsFlattenedLength::from(element_flattened_size);

    // Capacity is the number of entries in element_type_sizes array
    // One entry per field per logical element.
    match shift {
        ElementTypeSizesArrayShift::None => {}
        ElementTypeSizesArrayShift::Increase => {
            non_flattened_elements += SemanticLength(1);
        }
    }

    let capacity = non_flattened_elements * ElementTypesLength(assert_u32(element_types.len()));
    let capacity = capacity.to_usize();

    let mut flat_elem_type_sizes = Vec::with_capacity(capacity);
    let mut total_size = 0;
    for index in 0..capacity {
        flat_elem_type_sizes.push(total_size);
        total_size += element_type_sizes[index % element_types.len()].0;
    }
    flat_elem_type_sizes
}

/// Returns whether `value` contains an [`AcirValue::DynamicArray`] anywhere within it.
///
/// Such values are backed by a memory block rather than being held inline, so they cannot be
/// folded into another [`AcirValue::Array`] without reading them back out of that block.
fn contains_dynamic_array(value: &AcirValue) -> bool {
    match value {
        AcirValue::Var(_, _) => false,
        AcirValue::Array(values) => values.iter().any(contains_dynamic_array),
        AcirValue::DynamicArray(_) => true,
    }
}

/// Calculates the total flattened size of an [`AcirValue`].
///
/// Unlike [`Type::flattened_size`], this handles vectors, represented by [`AcirDynamicArray`], returning their capacity.
pub(super) fn flattened_value_size(value: &AcirValue) -> FlattenedLength {
    match value {
        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => *len,
        AcirValue::Var(_, _) => FlattenedLength(1),
        AcirValue::Array(values) => {
            let mut size = FlattenedLength(0);
            for value in values {
                size += flattened_value_size(value);
            }
            size
        }
    }
}

/// Returns whether the array's elements have a constant size.
///
/// This is useful as it then allows us to calculate the flattened index by multiplying by this constant
/// size.
///
/// # Returns
///
/// If the array's element types are all the same size then `array_has_constant_element_size` will return
/// `Some(element_size)` where `element_size` is the size of `array`'s elements. Otherwise returns `None`.
pub(super) fn array_has_constant_element_size(array_typ: &Type) -> Option<u32> {
    let (Type::Array(types, _) | Type::Vector(types)) = array_typ else {
        panic!("ICE: expected array or vector type");
    };

    let mut element_sizes = types.iter().map(|typ| typ.flattened_size());
    if let Some(element_size) = element_sizes.next() {
        if element_sizes.all(|size| size == element_size) { Some(element_size.0) } else { None }
    } else {
        // If the array has no types in it it can be because it's something like `[(); 3]` where `()` is represented
        // as "no types". And in this case the array has constant element size because it's zero.
        Some(0)
    }
}
