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
    pub(super) fn block_id(&mut self, value: &ValueId) -> BlockId {
        if let Some(block_id) = self.memory_blocks.get(value) {
            return *block_id;
        }
        let block_id = BlockId(self.max_block_id);
        self.max_block_id += 1;
        self.memory_blocks.insert(*value, block_id);
        block_id
    }

    /// Get the next BlockId for internal memory
    /// used during ACIR generation.
    /// This is useful for referencing information that can
    /// only be computed dynamically, such as the type structure
    /// of non-homogenous arrays.
    fn internal_block_id(&mut self, value: &ValueId) -> BlockId {
        if let Some(block_id) = self.internal_memory_blocks.get(value) {
            return *block_id;
        }
        let block_id = BlockId(self.max_block_id);
        self.max_block_id += 1;
        self.internal_memory_blocks.insert(*value, block_id);
        block_id
    }

    pub(super) fn initialize_databus(
        &mut self,
        witnesses: &Vec<Witness>,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        // Initialize return_data using provided witnesses
        if let Some(return_data) = self.data_bus.return_data {
            let block_id = self.block_id(&return_data);
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
        let mut mutable_array_set = false;

        // Pass the instruction between array methods rather than the internal fields themselves
        let (array, index, store_value) = match dfg[instruction] {
            Instruction::ArrayGet { array, index, offset: _ } => (array, index, None),
            Instruction::ArraySet { array, index, value, mutable, offset: _ } => {
                mutable_array_set = mutable;
                (array, index, Some(value))
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

        let array_typ = dfg.type_of_value(array);
        // Compiler sanity checks
        assert!(!array_typ.is_nested_slice(), "ICE: Nested slice type has reached ACIR generation");
        let (Type::Array(_, _) | Type::Slice(_)) = &array_typ else {
            unreachable!("ICE: expected array or slice type");
        };

        match &array_typ {
            // Replace Array operations on 0-length arrays by asserting that enable_side_effects are false
            Type::Array(_, len) => {
                let zero_var = self.acir_context.add_constant(FieldElement::zero());
                let result_ids = dfg.instruction_results(instruction);
                if result_ids.len() == 1 && *len == 0 {
                    let acir_typ: AcirType = dfg.type_of_value(result_ids[0]).into();
                    let zero_value = AcirValue::Var(zero_var, acir_typ);
                    self.define_result(dfg, instruction, zero_value);
                    let msg = format!("Index out of bounds, array has size 0");
                    let msg = self.acir_context.generate_assertion_message_payload(msg);
                    return self.acir_context.assert_eq_var(
                        self.current_side_effects_enabled_var,
                        zero_var,
                        Some(msg),
                    );
                }
            }
            Type::Slice(_) => (),
            _ => unreachable!("ICE: expected array or slice type"),
        }

        if self.handle_constant_index_wrapper(instruction, dfg, array, index, store_value)? {
            return Ok(());
        }

        // Get an offset such that the type of the array at the offset is the same as the type at the 'index'
        // If we find one, we will use it when computing the index under the enable_side_effect predicate
        // If not, array_get(..) will use a fallback costing one multiplication in the worst case.
        // cf. https://github.com/noir-lang/noir/pull/4971
        // For simplicity we compute the offset only for simple arrays
        let is_simple_array = dfg.instruction_results(instruction).len() == 1
            && (array_has_constant_element_size(&array_typ) == Some(1));
        let offset = if is_simple_array {
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
        };
        let (new_index, new_value) = self.convert_array_operation_inputs(
            array,
            dfg,
            index,
            store_value,
            offset.unwrap_or_default(),
        )?;

        if let Some(new_value) = new_value {
            self.array_set(instruction, new_index, new_value, dfg, mutable_array_set)?;
        } else {
            self.array_get(instruction, array, new_index, dfg, offset.is_none())?;
        }

        Ok(())
    }

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
            AcirValue::DynamicArray(_) => Ok(false),
        }
    }

    /// Handle constant index: if there is no predicate and we have the array values,
    /// we can perform the operation directly on the array
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
            // as if the predicate were true. This is as if the predicate were to resolve to false then
            // the result should not affect the rest of circuit execution.
            let value = array[index].clone();
            self.define_result(dfg, instruction, value);
            Ok(true)
        }
    }

    /// We need to properly setup the inputs for array operations in ACIR.
    /// From the original SSA values we compute the following AcirVars:
    /// - new_index is the index of the array. ACIR memory operations work with a flat memory, so we fully flattened the specified index
    ///   in case we have a nested array. The index for SSA array operations only represents the flattened index of the current array.
    ///   Thus internal array element type sizes need to be computed to accurately transform the index.
    ///
    /// - predicate_index is offset, or the index if the predicate is true
    ///
    /// - new_value is the optional value when the operation is an array_set
    ///   When there is a predicate, it is predicate*value + (1-predicate)*dummy, where dummy is the value of the array at the requested index.
    ///   It is a dummy value because in the case of a false predicate, the value stored at the requested index will be itself.
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

        let predicate_index = if dfg.is_safe_index(index, array_id) {
            index_var
        } else {
            // index*predicate + (1-predicate)*offset
            let offset = self.acir_context.add_constant(offset);
            let sub = self.acir_context.sub_var(index_var, offset)?;
            let pred = self.acir_context.mul_var(sub, self.current_side_effects_enabled_var)?;
            self.acir_context.add_var(pred, offset)?
        };

        let new_value = if let Some(store) = store_value {
            let store_value = self.convert_value(store, dfg);
            if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var) {
                Some(store_value)
            } else {
                let store_type = dfg.type_of_value(store);

                let mut dummy_predicate_index = predicate_index;
                // We must setup the dummy value to match the type of the value we wish to store
                let dummy =
                    self.array_get_value(&store_type, block_id, &mut dummy_predicate_index)?;

                Some(self.convert_array_set_store_value(&store_value, &dummy)?)
            }
        } else {
            None
        };

        let new_index = if self.acir_context.is_constant_one(&self.current_side_effects_enabled_var)
        {
            index_var
        } else {
            predicate_index
        };

        Ok((new_index, new_value))
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

                let values = try_vecmap(0..*len, |i| {
                    let index_var = self.acir_context.add_constant(i);

                    let read = self.acir_context.read_from_memory(*block_id, &index_var)?;
                    Ok::<AcirValue, RuntimeError>(AcirValue::Var(read, AcirType::field()))
                })?;

                let mut elements = im::Vector::new();
                for (val, dummy_val) in values.iter().zip(dummy_values) {
                    elements.push_back(self.convert_array_set_store_value(val, &dummy_val)?);
                }

                Ok(AcirValue::Array(elements))
            }
            (AcirValue::DynamicArray(_), AcirValue::DynamicArray(_)) => {
                unimplemented!("ICE: setting a dynamic array not supported");
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
        mut var_index: AcirVar,
        dfg: &DataFlowGraph,
        mut index_side_effect: bool,
    ) -> Result<AcirValue, RuntimeError> {
        let block_id = self.ensure_array_is_initialized(array, dfg)?;
        let results = dfg.instruction_results(instruction);
        let res_typ = dfg.type_of_value(results[0]);
        // Get operations to call-data parameters are replaced by a get to the call-data-bus array
        let call_data =
            self.data_bus.call_data.iter().find(|cd| cd.index_map.contains_key(&array)).cloned();
        let mut value = if let Some(call_data) = call_data {
            let call_data_block = self.ensure_array_is_initialized(call_data.array_id, dfg)?;
            let bus_index = self
                .acir_context
                .add_constant(FieldElement::from(call_data.index_map[&array] as i128));
            let mut current_index = self.acir_context.add_var(bus_index, var_index)?;
            self.get_from_call_data(&mut current_index, call_data_block, &res_typ)?
        } else {
            // Compiler sanity check
            assert!(
                !res_typ.contains_slice_element(),
                "ICE: Nested slice result found during ACIR generation"
            );
            self.array_get_value(&res_typ, block_id, &mut var_index)?
        };

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

            // Fallback to multiplication if the index side_effects have not already been handled
            if index_side_effect {
                // Set the value to 0 if current_side_effects is 0, to ensure it fits in any value type
                value = AcirValue::Var(
                    self.acir_context.mul_var(*value_var, self.current_side_effects_enabled_var)?,
                    typ.clone(),
                );
            }
        }

        self.define_result(dfg, instruction, value.clone());

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

        let block_id = self.ensure_array_is_initialized(array, dfg)?;

        // Every array has a length in its type, so we fetch that from
        // the SSA IR.
        //
        // A slice's size must be fetched from the SSA value that represents the slice.
        // However, this size is simply the capacity of a slice. The capacity is dependent upon the witness
        // and may contain data for which we want to restrict access. The true slice length is tracked in a
        // a separate SSA value and restrictions on slice indices should be generated elsewhere in the SSA.
        let array_typ = dfg.type_of_value(array);
        let array_len = self.flattened_size(array, dfg);

        // Since array_set creates a new array, we create a new block ID for this
        // array, unless map_array is true. In that case, we operate directly on block_id
        // and we do not create a new block ID.
        let result_id = dfg
            .instruction_results(instruction)
            .first()
            .expect("Array set does not have one result");
        let result_block_id;
        if mutate_array {
            self.memory_blocks.insert(*result_id, block_id);
            result_block_id = block_id;
        } else {
            // Initialize the new array with the values from the old array
            result_block_id = self.block_id(result_id);
            self.copy_array(array, result_block_id, dfg)?;
        }

        self.array_set_value(&store_value, result_block_id, &mut var_index)?;

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
            array_len,
            "ICE: The length of the flattened type array should match the length of the dynamic array"
        );

        let result_value = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: array_len,
            value_types,
            element_type_sizes,
        });
        self.define_result(dfg, instruction, result_value);
        Ok(())
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

    pub(super) fn init_element_type_sizes_array(
        &mut self,
        array_typ: &Type,
        array_id: ValueId,
        supplied_acir_value: Option<&AcirValue>,
        dfg: &DataFlowGraph,
    ) -> Result<BlockId, RuntimeError> {
        let element_type_sizes = self.internal_block_id(&array_id);
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

                self.internal_mem_block_lengths.insert(element_type_sizes, element_type_sizes_len);
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

                let type_sizes_array_len = *self.internal_mem_block_lengths.get(inner_elem_type_sizes).ok_or_else(||
                                            InternalError::General {
                                                message: format!("Array {array_id}'s inner element type sizes array does not have a tracked length"),
                                                call_stack: self.acir_context.get_call_stack(),
                                            }
                                        )?;
                self.copy_dynamic_array(
                    *inner_elem_type_sizes,
                    element_type_sizes,
                    type_sizes_array_len,
                )?;
                self.internal_mem_block_lengths.insert(element_type_sizes, type_sizes_array_len);
                Ok(element_type_sizes)
            }
            _ => Err(InternalError::Unexpected {
                expected: "AcirValue::DynamicArray or AcirValue::Array".to_owned(),
                found: format!("{:?}", array_acir_value),
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
            let mut size = 0;
            match &dfg[array] {
                Value::NumericConstant { .. } => {
                    size += 1;
                }
                Value::Instruction { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    size += flattened_value_size(&array_acir_value);
                }
                Value::Param { .. } => {
                    let array_acir_value = self.convert_value(array, dfg);
                    size += flattened_value_size(&array_acir_value);
                }
                _ => {
                    unreachable!("ICE: Unexpected SSA value when computing the slice size");
                }
            }
            size
        }
    }

    pub(super) fn ensure_array_is_initialized(
        &mut self,
        array: ValueId,
        dfg: &DataFlowGraph,
    ) -> Result<BlockId, RuntimeError> {
        // Use the SSA ID to get or create its block ID
        let block_id = self.block_id(&array);

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
            && self.block_id(&self.data_bus.return_data.unwrap()) == array
        {
            databus = BlockType::ReturnData;
        }
        for (call_data_id, array_id) in self.data_bus.call_data_array() {
            if self.block_id(&array_id) == array {
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
