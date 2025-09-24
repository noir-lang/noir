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
//! [BlockId]. This module provides helpers for translating SSA array
//! operations into ACIR memory reads and writes.
//!
//! ACIR generation use twos different array types for representing arrays:
//!
//! [Constant arrays][AcirValue::Array]
//!   - Known at compile time.  
//!   - Reads and writes may be folded into an [AcirValue] where possible.
//!   - Useful for optimization (e.g., constant element lookups do not require laying down opcodes)  
//!
//! [Dynamic arrays][AcirValue::DynamicArray]
//!   - Referenced by a [unique identifier][BlockId]
//!   - Must be explicitly initialized using an [opcode][acvm::acir::circuit::opcodes::Opcode::MemoryInit]
//!   - Reads and writes must lower to at least an explicit [memory opcode][acvm::acir::circuit::opcodes::Opcode::MemoryOp].
//!   - Required for arrays accessed by dynamic indices (witness inputs) or function parameters (the array is itself a witness)
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
//! We can determine what the starting offset of the outer array object `x[3]` is by first dividing the index by the element size.
//! In this case we have `11 / 3 = 3`. To get the flattened outer offset we will multiply `3` by the flattened size of a single element.
//! We can see that `Foo` contains `7` elements, thus we multiply by `7`. So now we have a flattened index of `21`.
//!
//! We could use the modulo of `11 % 3 = 2` to determine which object within x we are attempting to access.
//! However, since we have a flattened structure we cannot simply add `2` to `21`. Instead we generate an array that holds the
//! respective inner offset for accessing each element.
//!
//! In the example above we will generate this internal type array: [0, 1, 4].
//! The start of `bar` in `Foo` is the fourth flattened index. We then use the modulo we previously computed to access
//! the type array and fetch `4`. Finally we can do `21 + 4 = 25` to get the true starting index of `x[3].bar.inner`.
//! We then use the resulting type to increment the index appropriately and fetch ever element of `bar.inner`.
//!
//! This element type sizes array is dynamic as we still need to access it based upon the index which itself can be dynamic.
//! The module will also attempt to not create this array when possible (e.g., we have a simple homogenous array).
//!
//! ### Side effects and Predication
//!
//! This module uses the special [side effects variable][Context::current_side_effects_enabled_var] to guard
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
//! When the predicate is not a constant one, instead of actually overwriting memory, we compute a "dummy value".
//! The dummy value is fetched from the same array at the requested `predicate_index`.
//! The store value of an array write is then converted from a `store_value` to `predicate * store_value + (1 - predicate) * dummy`
//! This ensures the memory remains unchanged when the write is disabled. In the case of a false predicate, the value stored will be itself.
//!
//! #### Reads
//!
//! If we perform an array read under a false predicate we will read from `offset`. As arrays are not always homogenous
//! the result at index `offset` may contain a value that will overflow the resulting type of the array read.
//! When we read a value from a non-homogenous array we multiply any resulting [AcirValue::Var] by the predicate
//! to avoid any possible mismatch. In the case of a false predicate, the value will now be zero.
//! For homogenous arrays, the fallback `offset` will produce a value with a compatible type.
//!
//! ### Zero-Length Arrays
//!
//! Arrays of length 0 are valid in the SSA but must never generate ACIR
//! memory operations as they may produce runtime errors. These operations are special cased to always fail with an
//! index out of bounds error (with respect to side effects) and ensures they do not produce illegal memory accesses.
use acvm::acir::{circuit::opcodes::BlockType, native_types::Witness};
use acvm::{FieldElement, acir::AcirField, acir::circuit::opcodes::BlockId};
use iter_extended::{try_vecmap, vecmap};

use crate::errors::{InternalError, RuntimeError};
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Instruction, InstructionId},
    types::Type,
    value::{Value, ValueId},
};

use super::{
    AcirVar, Context,
    types::{AcirDynamicArray, AcirType, AcirValue},
};

impl Context<'_> {
    /// Get the BlockId corresponding to the ValueId
    /// If there is no matching BlockId, we create a new one.
    pub(super) fn block_id(&mut self, value: ValueId) -> BlockId {
        *self.memory_blocks.entry(value).or_insert_with(|| {
            let block_id = BlockId(self.max_block_id);
            self.max_block_id += 1;
            block_id
        })
    }

    /// Get the next BlockId for internal memory
    /// used during ACIR generation.
    /// This is useful for referencing information that can
    /// only be computed dynamically, such as the type structure
    /// of non-homogenous arrays.
    fn internal_block_id(&mut self, value: ValueId) -> BlockId {
        *self.element_type_sizes_blocks.entry(value).or_insert_with(|| {
            let block_id = BlockId(self.max_block_id);
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
        if let Some(return_data) = self.data_bus.return_data {
            let block_id = self.block_id(return_data);
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

    /// Handles an ArrayGet or ArraySet instruction.
    /// To set an index of the array (and create a new array in doing so), pass Some(value) for
    /// store_value. To just retrieve an index of the array, pass None for store_value.
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

        let array_typ = dfg.type_of_value(array);
        let offset = self.compute_offset(instruction, dfg, &array_typ);
        let (new_index, new_value) = self.convert_array_operation_inputs(
            array,
            dfg,
            index,
            store_value,
            offset.unwrap_or_default(),
        )?;

        if let Some(new_value) = new_value {
            self.array_set(instruction, new_index, new_value, dfg, mutable)?;
        } else {
            self.array_get(instruction, array, new_index, dfg, offset.is_none())?;
        }

        Ok(())
    }

    /// For 0-length arrays and slices, even the disabled memory operations would cause runtime failures.
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
        if self.has_zero_length(array, dfg) {
            // Zero result.
            let result_ids = dfg.instruction_results(instruction);
            for result_id in result_ids {
                let res_typ = dfg.type_of_value(*result_id);
                let zero_value = self.array_zero_value(&res_typ)?;
                self.ssa_values.insert(*result_id, zero_value);
            }
            // Make sure this code is disabled, or fail with "Index out of bounds".
            let msg = "Index out of bounds, array has size 0".to_string();
            let msg = self.acir_context.generate_assertion_message_payload(msg);
            let zero = self.acir_context.add_constant(FieldElement::zero());
            self.acir_context.assert_eq_var(
                self.current_side_effects_enabled_var,
                zero,
                Some(msg),
            )?;
            Ok(true)
        } else {
            Ok(false)
        }
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
            AcirValue::Array(array) => {
                // `AcirValue::Array` supports reading/writing to constant indices at compile-time in some cases.
                if let Some(constant_index) = dfg.get_numeric_constant(index) {
                    let store_value = store_value.map(|value| self.convert_value(value, dfg));
                    self.handle_constant_index(instruction, dfg, array, constant_index, store_value)
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

    /// See [Self::handle_constant_index_wrapper]
    fn handle_constant_index(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array: im::Vector<AcirValue>,
        index: FieldElement,
        store_value: Option<AcirValue>,
    ) -> Result<bool, RuntimeError> {
        let array_size: usize = array.len();
        let index = match index.try_to_u64() {
            Some(index_const) => index_const as usize,
            None => {
                let call_stack = self.acir_context.get_call_stack();
                return Err(RuntimeError::TypeConversion {
                    from: "array index".to_string(),
                    into: "u64".to_string(),
                    call_stack,
                });
            }
        };

        if index >= array_size {
            return Ok(false);
        }

        if let Some(store_value) = store_value {
            let side_effects_always_enabled =
                self.acir_context.is_constant_one(&self.current_side_effects_enabled_var);

            if side_effects_always_enabled {
                // If we know that this write will always occur then we can perform it at compile time.
                let value = AcirValue::Array(array.update(index, store_value));
                self.define_result(dfg, instruction, value);
                Ok(true)
            } else {
                // If a predicate is applied however we must wait until runtime.
                Ok(false)
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

    /// Get an offset such that the type of the array at the offset is the same as the type at the 'index'
    /// If we find one, we will use it when computing the index under the enable_side_effect predicate
    /// If not, array_get(..) will use a fallback costing one multiplication in the worst case.
    /// cf. <https://github.com/noir-lang/noir/pull/4971>
    /// For simplicity we compute the offset only for simple arrays
    fn compute_offset(
        &mut self,
        instruction: InstructionId,
        dfg: &DataFlowGraph,
        array_typ: &Type,
    ) -> Option<usize> {
        let is_simple_array = dfg.instruction_results(instruction).len() == 1
            && (array_has_constant_element_size(array_typ) == Some(1));
        if is_simple_array {
            let result_type = dfg.type_of_value(dfg.instruction_results(instruction)[0]);
            match array_typ {
                Type::Array(item_type, _) | Type::Slice(item_type) => item_type
                    .iter()
                    .enumerate()
                    .find_map(|(index, typ)| (result_type == *typ).then_some(index)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// We need to properly setup the inputs for array operations in ACIR.
    /// From the original SSA values we compute the following AcirVars:
    /// - `index_var` is the index of the array. ACIR memory operations work with a flat memory, so we fully flattened the specified index
    ///   in case we have a nested array. The index for SSA array operations only represents the flattened index of the current array.
    ///   Thus internal array element type sizes need to be computed to accurately transform the index.
    ///
    /// - If the predicate is known to be true or the array access is guaranteed to be safe, we can directly return `index_var`
    ///   Otherwise, `predicate_index` is a fallback offset set by [Self::predicated_index].
    ///
    /// - `new_value` is the optional value when the operation is an array_set.
    ///   The value used in an array_set is also dependent upon the predicate and is set in [Self::predicated_store_value]
    fn convert_array_operation_inputs(
        &mut self,
        array_id: ValueId,
        dfg: &DataFlowGraph,
        index: ValueId,
        store_value: Option<ValueId>,
        offset: usize,
    ) -> Result<(AcirVar, Option<AcirValue>), RuntimeError> {
        let array_typ = dfg.type_of_value(array_id);
        let block_id = self.ensure_array_is_initialized(array_id, dfg)?;

        let index_var = self.convert_numeric_value(index, dfg)?;
        let index_var = self.get_flattened_index(&array_typ, array_id, index_var, dfg)?;

        // Side-effects are always enabled so we do not need to do any predication
        if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var) {
            let store_value = store_value.map(|store| self.convert_value(store, dfg));
            return Ok((index_var, store_value));
        }

        let predicate_index = self.predicated_index(index_var, index, array_id, dfg, offset)?;

        // Handle the predicated store value
        let new_value = store_value
            .map(|store| self.predicated_store_value(store, dfg, block_id, predicate_index))
            .transpose()?;

        Ok((predicate_index, new_value))
    }

    /// Computes the predicated index for an array access.
    /// If the index is always safe, it is returned directly.
    /// Otherwise, we compute `predicate * index + (1 - predicate) * offset`.
    fn predicated_index(
        &mut self,
        index_var: AcirVar,
        index: ValueId,
        array_id: ValueId,
        dfg: &DataFlowGraph,
        offset: usize,
    ) -> Result<AcirVar, RuntimeError> {
        if dfg.is_safe_index(index, array_id) {
            Ok(index_var)
        } else {
            let offset = self.acir_context.add_constant(offset);
            let sub = self.acir_context.sub_var(index_var, offset)?;
            let pred = self.acir_context.mul_var(sub, self.current_side_effects_enabled_var)?;
            self.acir_context.add_var(pred, offset)
        }
    }

    /// When there is a predicate, the store value is predicate*value + (1-predicate)*dummy, where dummy is the value of the array at the requested index.
    /// It is a dummy value because in the case of a false predicate, the value stored at the requested index will be itself.
    fn predicated_store_value(
        &mut self,
        store: ValueId,
        dfg: &DataFlowGraph,
        block_id: BlockId,
        mut dummy_predicate_index: AcirVar,
    ) -> Result<AcirValue, RuntimeError> {
        let store_value = self.convert_value(store, dfg);
        let store_type = dfg.type_of_value(store);
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
            (AcirValue::Var(store_var, _), AcirValue::Var(dummy_var, _)) => {
                let true_pred =
                    self.acir_context.mul_var(*store_var, self.current_side_effects_enabled_var)?;
                let one = self.acir_context.add_constant(FieldElement::one());
                let not_pred =
                    self.acir_context.sub_var(one, self.current_side_effects_enabled_var)?;
                let false_pred = self.acir_context.mul_var(not_pred, *dummy_var)?;
                // predicate*value + (1-predicate)*dummy
                let new_value = self.acir_context.add_var(true_pred, false_pred)?;
                Ok(AcirValue::Var(new_value, AcirType::field()))
            }
            (AcirValue::Array(values), AcirValue::Array(dummy_values)) => {
                let mut elements = im::Vector::new();

                assert_eq!(
                    values.len(),
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );
                for (val, dummy_val) in values.iter().zip(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (
                AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }),
                AcirValue::Array(dummy_values),
            ) => {
                let dummy_values = dummy_values
                    .into_iter()
                    .flat_map(|val| val.clone().flatten())
                    .map(|(var, typ)| AcirValue::Var(var, typ))
                    .collect::<Vec<_>>();

                assert_eq!(
                    *len,
                    dummy_values.len(),
                    "ICE: The store value and dummy must have the same number of inner values"
                );

                let values = self.read_dynamic_array(*block_id, *len)?;
                let mut elements = im::Vector::new();
                for (val, dummy_val) in values.iter().zip(dummy_values) {
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
                let mut result = im::Vector::new();
                for _i in 0..*len {
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

    /// Generates a read opcode for the array
    /// `index_side_effect == false` means that we ensured `var_index` will have a type matching the value in the array
    fn array_get(
        &mut self,
        instruction: InstructionId,
        array: ValueId,
        var_index: AcirVar,
        dfg: &DataFlowGraph,
        index_side_effect: bool,
    ) -> Result<(), RuntimeError> {
        let block_id = self.ensure_array_is_initialized(array, dfg)?;
        let results = dfg.instruction_results(instruction);
        let res_typ = dfg.type_of_value(results[0]);

        let value = self.load_array_value(array, block_id, var_index, &res_typ, dfg)?;

        let value = self.apply_index_side_effects(array, value, index_side_effect, dfg)?;

        self.define_result(dfg, instruction, value);
        Ok(())
    }

    /// Loads a value either from call-data bus or from memory.
    fn load_array_value(
        &mut self,
        array: ValueId,
        block_id: BlockId,
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
            let call_data_block = self.ensure_array_is_initialized(array_id, dfg)?;
            let bus_index = self.acir_context.add_constant(FieldElement::from(bus_index as i128));
            let mut current_index = self.acir_context.add_var(bus_index, var_index)?;
            self.get_from_call_data(&mut current_index, call_data_block, res_typ)
        } else {
            self.array_get_value(res_typ, block_id, &mut var_index)
        }
    }

    /// Applies predication logic on the result in case the read under a false predicate
    /// returns a value with a larger type that may later trigger an overflow.
    /// Ensures values read under false predicate are zeroed out if types don’t align.
    fn apply_index_side_effects(
        &mut self,
        array: ValueId,
        mut value: AcirValue,
        mut index_side_effect: bool,
        dfg: &DataFlowGraph,
    ) -> Result<AcirValue, RuntimeError> {
        if let AcirValue::Var(value_var, typ) = &value {
            let array_typ = dfg.type_of_value(array);
            if let (Type::Numeric(numeric_type), AcirType::NumericType(num)) =
                (array_typ.first(), typ)
            {
                if numeric_type.bit_size() <= num.bit_size() {
                    // first element is compatible
                    index_side_effect = false;
                }
            }

            if index_side_effect {
                value = AcirValue::Var(
                    self.acir_context.mul_var(*value_var, self.current_side_effects_enabled_var)?,
                    typ.clone(),
                );
            }
        }
        Ok(value)
    }

    pub(super) fn array_get_value(
        &mut self,
        ssa_type: &Type,
        block_id: BlockId,
        var_index: &mut AcirVar,
    ) -> Result<AcirValue, RuntimeError> {
        let one = self.acir_context.add_constant(FieldElement::one());
        match ssa_type.clone() {
            Type::Numeric(numeric_type) => {
                // Read the value from the array at the specified index
                let read = self.acir_context.read_from_memory(block_id, var_index)?;

                // Increment the var_index in case of a nested array
                *var_index = self.acir_context.add_var(*var_index, one)?;

                let typ = AcirType::NumericType(numeric_type);
                Ok(AcirValue::Var(read, typ))
            }
            Type::Array(element_types, len) => {
                let mut values = im::Vector::new();
                for _ in 0..len {
                    for typ in element_types.as_ref() {
                        values.push_back(self.array_get_value(typ, block_id, var_index)?);
                    }
                }
                Ok(AcirValue::Array(values))
            }
            Type::Reference(reference_type) => {
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
                let typ = AcirType::NumericType(numeric_type);
                Ok(AcirValue::Var(zero, typ))
            }
            Type::Array(element_types, len) => {
                let mut values = im::Vector::new();
                for _ in 0..len {
                    for typ in element_types.as_ref() {
                        values.push_back(self.array_zero_value(typ)?);
                    }
                }
                Ok(AcirValue::Array(values))
            }
            Type::Reference(reference_type) => self.array_zero_value(reference_type.as_ref()),
            _ => unreachable!("ICE: Expected an array or numeric but got {ssa_type:?}"),
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
        let array = match dfg[instruction] {
            Instruction::ArraySet { array, .. } => array,
            _ => {
                return Err(InternalError::Unexpected {
                    expected: "Instruction should be an ArraySet".to_owned(),
                    found: format!("Instead got {:?}", dfg[instruction]),
                    call_stack: self.acir_context.get_call_stack(),
                }
                .into());
            }
        };

        let result_id = *dfg
            .instruction_results(instruction)
            .first()
            .expect("Array set does not have one result");
        let block_id = self.resolve_array_set_block(array, result_id, dfg, mutate_array)?;

        self.array_set_value(&store_value, block_id, &mut var_index)?;

        let result_value = self.make_array_set_result_value(array, block_id, dfg)?;

        self.define_result(dfg, instruction, result_value);
        Ok(())
    }

    // Since array_set creates a new array, we create a new block ID for this
    // array, unless mutate_array is true. In that case, we operate directly on block_id
    // and we do not create a new block ID.
    fn resolve_array_set_block(
        &mut self,
        array: ValueId,
        result: ValueId,
        dfg: &DataFlowGraph,
        mutate_array: bool,
    ) -> Result<BlockId, RuntimeError> {
        let block_id = self.ensure_array_is_initialized(array, dfg)?;
        if mutate_array {
            self.memory_blocks.insert(result, block_id);
            Ok(block_id)
        } else {
            let new_block = self.block_id(result);
            self.copy_array(array, new_block, dfg)?;
            Ok(new_block)
        }
    }

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
                // Increment the var_index in case of a nested array
                *var_index = self.acir_context.add_var(*var_index, one)?;
            }
            AcirValue::Array(values) => {
                for value in values {
                    self.array_set_value(value, block_id, var_index)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id: inner_block_id, len, .. }) => {
                let values = try_vecmap(0..*len, |i| {
                    let index_var = self.acir_context.add_constant(i);

                    let read = self.acir_context.read_from_memory(*inner_block_id, &index_var)?;
                    Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
                })?;
                self.array_set_value(&AcirValue::Array(values.into()), block_id, var_index)?;
            }
        }
        Ok(())
    }

    /// Construct the [AcirValue::DynamicArray] that represents the result of an [Instruction::ArraySet].
    ///
    /// In SSA, an array set always yields a new array value (even if the operation
    /// mutates in place). At the ACIR level, this corresponds to a [AcirValue::DynamicArray] whose
    /// memory block has already been resolved by [Self::resolve_array_set_block].
    ///
    /// # Purpose
    /// - Initializes the optional [AcirDynamicArray::element_type_sizes] helper array for when elements are non-homogenous.
    /// - Populates the `value_types` vector. See [AcirDynamicArray::value_types] for more information.
    fn make_array_set_result_value(
        &mut self,
        array: ValueId,
        block_id: BlockId,
        dfg: &DataFlowGraph,
    ) -> Result<AcirValue, RuntimeError> {
        // Every array has a length in its type, so we fetch that from
        // the SSA IR.
        //
        // A slice's size must be fetched from the SSA value that represents the slice.
        // However, this size is simply the capacity of a slice. The capacity is dependent upon the witness
        // and may contain data for which we want to restrict access. The true slice length is tracked in a
        // a separate SSA value and restrictions on slice indices should be generated elsewhere in the SSA.
        let array_typ = dfg.type_of_value(array);
        let len = self.flattened_size(array, dfg);

        let element_type_sizes = if array_has_constant_element_size(&array_typ).is_none() {
            let acir_value = self.convert_value(array, dfg);
            Some(self.init_element_type_sizes_array(&array_typ, array, Some(&acir_value), dfg)?)
        } else {
            None
        };

        let value_types = self.convert_value(array, dfg).flat_numeric_types();
        // Compiler sanity check
        assert_eq!(
            value_types.len(),
            len,
            "ICE: The length of the flattened type array should match the length of the dynamic array"
        );

        Ok(AcirValue::DynamicArray(AcirDynamicArray {
            block_id,
            len,
            value_types,
            element_type_sizes,
        }))
    }

    pub(super) fn init_element_type_sizes_array(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        supplied_acir_value: Option<&AcirValue>,
        dfg: &DataFlowGraph,
    ) -> Result<BlockId, RuntimeError> {
        let element_type_sizes = self.internal_block_id(array_id);
        // Check whether an internal type sizes array has already been initialized
        // Need to look into how to optimize for slices as this could lead to different element type sizes
        // for different slices that do not have consistent sizes
        if self.initialized_arrays.contains(&element_type_sizes) {
            return Ok(element_type_sizes);
        }

        if !matches!(array_typ, Type::Array(_, _) | Type::Slice(_)) {
            return Err(InternalError::Unexpected {
                expected: "array or slice".to_owned(),
                found: array_typ.to_string(),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        }

        if !matches!(&dfg[array_id], Value::Instruction { .. } | Value::Param { .. }) {
            return Err(InternalError::Unexpected {
                expected: "array or instruction".to_owned(),
                found: format!("{:?}", &dfg[array_id]),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into());
        }

        // An instruction representing the slice means it has been processed previously during ACIR gen.
        // Use the previously defined result of an array operation to fetch the internal type information.
        let array_acir_value = &self.convert_value(array_id, dfg);
        let array_acir_value = supplied_acir_value.unwrap_or(array_acir_value);
        match array_acir_value {
            AcirValue::Array(values) => {
                let flat_elem_type_sizes = calculate_element_type_sizes_array(values);

                // The final array should will the flattened index at each outer array index
                let init_values = vecmap(flat_elem_type_sizes, |type_size| {
                    let var = self.acir_context.add_constant(type_size);
                    AcirValue::Var(var, AcirType::field())
                });
                let element_type_sizes_len = init_values.len();
                self.initialize_array(
                    element_type_sizes,
                    element_type_sizes_len,
                    Some(AcirValue::Array(init_values.into())),
                )?;

                Ok(element_type_sizes)
            }

            AcirValue::DynamicArray(AcirDynamicArray {
                element_type_sizes: inner_elem_type_sizes,
                ..
            }) => {
                let Some(inner_elem_type_sizes) = inner_elem_type_sizes else {
                    return Err(InternalError::General {
                        message: format!("Array {array_id}'s inner element type sizes array should be initialized"),
                        call_stack: self.acir_context.get_call_stack(),
                    }
                    .into());
                };

                if !self.initialized_arrays.contains(inner_elem_type_sizes) {
                    // We're copying the element type sizes array from another array so we expect it to be initialized.
                    unreachable!("ICE: element type size arrays are expected to be initialized");
                }

                // We can safely overwrite the memory block from the initial call to `self.internal_block_id(&array_id)` here.
                // The type sizes array is never mutated so we can re-use it.
                self.element_type_sizes_blocks.insert(array_id, *inner_elem_type_sizes);
                Ok(*inner_elem_type_sizes)
            }
            _ => Err(InternalError::Unexpected {
                expected: "AcirValue::DynamicArray or AcirValue::Array".to_owned(),
                found: format!("{array_acir_value:?}"),
                call_stack: self.acir_context.get_call_stack(),
            }
            .into()),
        }
    }

    pub(super) fn read_array(
        &mut self,
        array: AcirValue,
    ) -> Result<im::Vector<AcirValue>, RuntimeError> {
        match array {
            AcirValue::Var(_, _) => unreachable!("ICE: attempting to copy a non-array value"),
            AcirValue::Array(vars) => Ok(vars),
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }) => {
                self.read_dynamic_array(block_id, len)
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
            AcirValue::DynamicArray(source) => {
                self.copy_dynamic_array(source.block_id, destination, source.len)
            }
        }
    }

    fn read_dynamic_array(
        &mut self,
        source: BlockId,
        array_len: usize,
    ) -> Result<im::Vector<AcirValue>, RuntimeError> {
        let init_values = try_vecmap(0..array_len, |i| {
            let index_var = self.acir_context.add_constant(i);

            let read = self.acir_context.read_from_memory(source, &index_var)?;
            Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
        })?;
        Ok(init_values.into())
    }

    fn copy_dynamic_array(
        &mut self,
        source: BlockId,
        destination: BlockId,
        array_len: usize,
    ) -> Result<(), RuntimeError> {
        let array = self.read_dynamic_array(source, array_len)?;
        self.initialize_array(destination, array_len, Some(AcirValue::Array(array)))?;
        Ok(())
    }

    pub(super) fn get_flattened_index(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        var_index: AcirVar,
        dfg: &DataFlowGraph,
    ) -> Result<AcirVar, RuntimeError> {
        if let Some(step_size) = array_has_constant_element_size(array_typ) {
            let step_size = self.acir_context.add_constant(step_size);
            self.acir_context.mul_var(var_index, step_size)
        } else {
            let element_type_sizes =
                self.init_element_type_sizes_array(array_typ, array_id, None, dfg)?;

            let predicate_index =
                self.acir_context.mul_var(var_index, self.current_side_effects_enabled_var)?;

            self.acir_context
                .read_from_memory(element_type_sizes, &predicate_index)
                .map_err(RuntimeError::from)
        }
    }

    pub(super) fn flattened_size(&mut self, array: ValueId, dfg: &DataFlowGraph) -> usize {
        let array_typ = dfg.type_of_value(array);
        if !array_typ.contains_slice_element() {
            array_typ.flattened_size() as usize
        } else {
            match &dfg[array] {
                Value::NumericConstant { .. } => 1,
                Value::Instruction { .. } | Value::Param { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    flattened_value_size(&array_acir_value)
                }
                _ => {
                    unreachable!("ICE: Unexpected SSA value when computing the slice size");
                }
            }
        }
    }

    /// Check if the array or slice has 0 length.
    ///
    /// This is different from `flattened_size` in that a non-zero length
    /// array containing zero length arrays has zero size, but we can still
    /// access its elements.
    fn has_zero_length(&mut self, array: ValueId, dfg: &DataFlowGraph) -> bool {
        if let Type::Array(_, size) = dfg.type_of_value(array) {
            size == 0
        } else {
            match &dfg[array] {
                Value::Instruction { .. } | Value::Param { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    match array_acir_value {
                        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => len == 0,
                        AcirValue::Array(values) => values.is_empty(),
                        AcirValue::Var(_, _) => {
                            unreachable!(
                                "ICE: Unexpected ACIR value for array or slice: {array_acir_value:?}"
                            )
                        }
                    }
                }
                other => {
                    unreachable!(
                        "ICE: Unexpected SSA value when computing the slice size: {other:?}"
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
                Value::Instruction { .. } => {
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
        len: usize,
        value: Option<AcirValue>,
    ) -> Result<(), InternalError> {
        let mut databus = BlockType::Memory;
        if self.data_bus.return_data.is_some()
            && self.block_id(self.data_bus.return_data.unwrap()) == array
        {
            databus = BlockType::ReturnData;
        }
        for (call_data_id, array_id) in self.data_bus.call_data_array() {
            if self.block_id(array_id) == array {
                assert!(databus == BlockType::Memory);
                databus = BlockType::CallData(call_data_id);
                break;
            }
        }

        self.acir_context.initialize_array(array, len, value, databus)?;
        if self.initialized_arrays.insert(array) {
            Ok(())
        } else {
            Err(InternalError::General {
                message: "Attempted to initialize memory block twice".to_owned(),
                call_stack: self.acir_context.get_call_stack(),
            })
        }
    }
}

fn calculate_element_type_sizes_array(array: &im::Vector<AcirValue>) -> Vec<usize> {
    let mut flat_elem_type_sizes = Vec::new();
    flat_elem_type_sizes.push(0);
    for (i, value) in array.iter().enumerate() {
        flat_elem_type_sizes.push(flattened_value_size(value) + flat_elem_type_sizes[i]);
    }

    flat_elem_type_sizes
}

pub(super) fn flattened_value_size(value: &AcirValue) -> usize {
    let mut size = 0;
    match value {
        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => {
            size += len;
        }
        AcirValue::Var(_, _) => {
            size += 1;
        }
        AcirValue::Array(values) => {
            for value in values {
                size += flattened_value_size(value);
            }
        }
    }
    size
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
    let types = match array_typ {
        Type::Array(types, _) | Type::Slice(types) => types,
        _ => panic!("ICE: expected array or slice type"),
    };

    let mut element_sizes = types.iter().map(|typ| typ.flattened_size());
    let element_size = element_sizes.next().expect("must have at least one element");

    if element_sizes.all(|size| size == element_size) { Some(element_size) } else { None }
}
