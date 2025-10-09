use crate::acir::{AcirDynamicArray, AcirType, AcirValue};
use crate::errors::RuntimeError;
use crate::ssa::ir::{dfg::DataFlowGraph, value::ValueId};
use acvm::{AcirField, FieldElement};

use super::Context;

impl Context<'_> {
    /// Pushes one or more elements to the back of a non-nested slice.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length
    /// * `arguments[1]` - Slice contents
    /// * `arguments[2..]` - Elements to push back
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated slice length (incremented by one)
    /// 2. New slice with elements appended
    pub(super) fn convert_slice_push_back(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];
        let elements_to_push = &arguments[2..];

        // Increase the slice length by one to enable accessing more elements in the slice.
        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.add_var(slice_length, one)?;

        let slice = self.convert_value(slice_contents, dfg);
        let mut new_slice = self.read_array(slice)?;

        // We must directly push back elements for non-nested slices
        for elem in elements_to_push {
            let element = self.convert_value(*elem, dfg);
            new_slice.push_back(element);
        }

        let new_slice_val = AcirValue::Array(new_slice);

        Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
    }

    /// Pushes one or more elements to the front of a non-nested slice.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length
    /// * `arguments[1]` - Slice contents
    /// * `arguments[2..]` - Elements to push to the front
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated slice length (incremented by one)
    /// 2. New slice with elements prepended
    pub(super) fn convert_slice_push_front(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];
        let elements_to_push = &arguments[2..];

        // Increase the slice length by one to enable accessing more elements in the slice.
        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.add_var(slice_length, one)?;

        let slice = self.convert_value(slice_contents, dfg);
        let mut new_slice = self.read_array(slice)?;

        // We must directly push front elements for non-nested slices
        for elem in elements_to_push.iter().rev() {
            let element = self.convert_value(*elem, dfg);
            new_slice.push_front(element);
        }

        let new_slice_val = AcirValue::Array(new_slice);

        Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
    }

    /// Removes and returns one or more elements from the back of a non-nested slice.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length
    /// * `arguments[1]` - Slice contents
    /// * `result_ids[0]` - Updated slice length
    /// * `result_ids[1]` - Update slice contents
    /// * `result_ids[2..]` - Locations where popped elements will be stored
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated slice length (decremented by one)
    /// 2. Slice contents (unchanged; accesses are always gated by the dynamic length)
    /// 3. Popped elements in order
    ///
    /// # Design
    ///
    /// The slice is represented in **flattened form** in memory. Popping the back
    /// involves:
    ///
    /// 1. Decrementing the slice length by one.
    /// 2. Using the decremented length as an offset for the elements to remove.
    /// 3. Read out the elements located at that offset.
    ///
    /// The `result_ids` provided by the SSA to fetch the appropriate type information to be popped.
    /// The `result_ids` encode the type/shape of the removed element.
    pub(super) fn convert_slice_pop_back(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.sub_var(slice_length, one)?;
        // For a pop back operation we want to fetch from the `length - 1` as this is the
        // last valid index that can be accessed in a slice. After the pop back operation
        // the elements stored at that index will no longer be able to be accessed.
        let mut var_index = new_slice_length;

        let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

        let mut popped_elements = Vec::new();
        for res in &result_ids[2..] {
            let elem = self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
            popped_elements.push(elem);
        }

        let slice = self.convert_value(slice_contents, dfg);
        let new_slice = self.read_array(slice)?;

        let mut results =
            vec![AcirValue::Var(new_slice_length, AcirType::field()), AcirValue::Array(new_slice)];
        results.append(&mut popped_elements);

        Ok(results)
    }

    /// Removes and returns one or more elements from the front of a non-nested slice.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length
    /// * `arguments[1]` - Slice contents
    /// * `result_ids[..element_size]` - Locations for the popped elements
    /// * `result_ids[element_size]` - Updated slice length
    /// * `result_ids[element_size + 1]` - Updated slice contents
    ///
    /// `element_size` refers to the result of [crate::ssa::ir::types::Type::element_size].
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Popped elements in order
    /// 2. Updated slice length (decremented by one)
    /// 3. Updated slice contents with the front elements removed
    ///
    /// # Design
    ///
    /// Slices are stored in **flattened form** in memory. To pop from the front:
    ///
    /// 1. Decrement the slice length by the size of one element.
    /// 2. Read out the first `element_size` values at index `0`.
    /// 3. Shift the update slice's memory forward by `element_size` slots to represent the updated slice.
    ///
    /// Unlike in [Self::convert_slice_pop_back], the returned slice contents differ from the input:
    /// the underlying array is logically truncated at the *front* rather than
    /// the back. The `result_ids` ensure that this logical shift is applied
    /// consistently with the element’s type.
    pub(super) fn convert_slice_pop_front(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];

        let slice_typ = dfg.type_of_value(slice_contents);
        let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

        let slice = self.convert_value(slice_contents, dfg);
        let is_dynamic = matches!(slice, AcirValue::DynamicArray(_));

        let mut new_slice = self.read_array(slice)?;

        let element_size = slice_typ.element_size();

        let mut popped_elements: Vec<AcirValue> = Vec::new();
        let mut popped_elements_size = 0;
        let mut var_index = self.acir_context.add_constant(FieldElement::zero());
        // Fetch the values we are popping off of the slice.
        // In the case of non-nested slice the logic is simple as we do not
        // need to account for the internal slice sizes or flattening the index.
        for res in &result_ids[..element_size] {
            let element =
                self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
            let elem_size = super::arrays::flattened_value_size(&element);
            popped_elements_size += elem_size;
            popped_elements.push(element);
        }

        // It is expected that the `popped_elements_size` is the flattened size of the elements,
        // as the input slice should be a dynamic array which is represented by flat memory.
        // However in some cases the input slice is an Array with a nested structure,
        // in which case we only need to pop the items that represent a single entry.
        let popped_elements_size = if is_dynamic { popped_elements_size } else { element_size };

        new_slice = new_slice.slice(popped_elements_size..);

        popped_elements.push(AcirValue::Var(new_slice_length, AcirType::field()));
        popped_elements.push(AcirValue::Array(new_slice));

        Ok(popped_elements)
    }

    /// Inserts one or more elements into a slice at a given index.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length  
    /// * `arguments[1]` - Slice contents  
    /// * `arguments[2]` - Insert index (logical element index, not flattened)  
    /// * `arguments[3..]` - Elements to insert  
    /// * `result_ids[0]` - Updated slice length  
    /// * `result_ids[1]` - Updated slice contents  
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated slice length (incremented by one)  
    /// 2. Updated slice contents with the new elements inserted at the given index  
    ///
    /// # Design
    ///
    /// Slices are represented in **flattened form** in memory. Inserting requires
    /// shifting a contiguous region of elements upward to make room for the new ones:
    ///
    /// 1. Compute the flattened insert index:  
    ///    - Multiply the logical insert index by the element size.  
    ///    - Adjust for non-homogenous structures via [Self::get_flattened_index].
    /// 2. Flatten the new elements (`flattened_elements`)
    /// 3. For each position in the result slice:  
    ///    - If below the insert index, copy from the original slice.  
    ///    - If within the insertion window, write values from `flattened_elements`.  
    ///    - If above the window, shift elements upward by the size of the inserted data.  
    /// 4. Initialize a new memory block for the resulting slice, ensuring its type information is preserved.  
    pub(super) fn convert_slice_insert(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];

        let slice_typ = dfg.type_of_value(slice_contents);
        let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

        let slice = self.convert_value(slice_contents, dfg);
        let insert_index = self.convert_value(arguments[2], dfg).into_var()?;

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.add_var(slice_length, one)?;

        let mut slice_size = super::arrays::flattened_value_size(&slice);

        // Fetch the flattened index from the user provided index argument.
        let flat_user_index =
            self.get_flattened_index(&slice_typ, slice_contents, insert_index, dfg)?;

        let elements_to_insert = &arguments[3..];
        // Determine the elements we need to write into our resulting dynamic array.
        // We need to a fully flat list of AcirVar's as a dynamic array is represented with flat memory.
        let mut inner_elem_size_usize = 0;
        let mut flattened_elements = Vec::new();
        let mut new_value_types = Vec::new();
        for elem in elements_to_insert {
            let element = self.convert_value(*elem, dfg);
            // Flatten into (AcirVar, NumericType) pairs
            let flat_element = self.flatten(&element)?;
            let elem_size = flat_element.len();
            inner_elem_size_usize += elem_size;
            slice_size += elem_size;
            for (var, typ) in flat_element {
                flattened_elements.push(var);
                new_value_types.push(typ);
            }
        }
        let inner_elem_size = self.acir_context.add_constant(inner_elem_size_usize);
        // Set the maximum flattened index at which a new element should be inserted.
        let max_flat_user_index = self.acir_context.add_var(flat_user_index, inner_elem_size)?;

        // Go through the entire slice argument and determine what value should be written to the new slice.
        // 1. If we are below the starting insertion index we should insert the value that was already
        //    in the original slice.
        // 2. If we are above the starting insertion index but below the max insertion index we should insert
        //    the flattened element arguments.
        // 3. If we are above the max insertion index we should insert the previous value from the original slice,
        //    as during an insertion we want to shift all elements after the insertion up an index.
        let result_block_id = self.block_id(result_ids[1]);
        self.initialize_array(result_block_id, slice_size, None)?;
        let mut current_insert_index = 0;
        for i in 0..slice_size {
            let current_index = self.acir_context.add_constant(i);

            // Check that we are above the lower bound of the insertion index
            let is_after_insert =
                self.acir_context.more_than_eq_var(current_index, flat_user_index, 64)?;
            // Check that we are below the upper bound of the insertion index
            let is_before_insert =
                self.acir_context.less_than_var(current_index, max_flat_user_index, 64)?;

            // Read from the original slice the value we want to insert into our new slice.
            // We need to make sure that we read the previous element when our current index is greater than insertion index.
            // If the index for the previous element is out of the array bounds we can avoid the check for whether
            // the current index is over the insertion index.
            let shifted_index = if i < inner_elem_size_usize {
                current_index
            } else {
                let index_minus_elem_size =
                    self.acir_context.add_constant(i - inner_elem_size_usize);

                let use_shifted_index_pred =
                    self.acir_context.mul_var(index_minus_elem_size, is_after_insert)?;

                let not_pred = self.acir_context.sub_var(one, is_after_insert)?;
                let use_current_index_pred = self.acir_context.mul_var(not_pred, current_index)?;

                self.acir_context.add_var(use_shifted_index_pred, use_current_index_pred)?
            };

            let value_shifted_index =
                self.acir_context.read_from_memory(block_id, &shifted_index)?;

            // Final predicate to determine whether we are within the insertion bounds
            let should_insert_value_pred =
                self.acir_context.mul_var(is_after_insert, is_before_insert)?;
            let insert_value_pred = self
                .acir_context
                .mul_var(flattened_elements[current_insert_index], should_insert_value_pred)?;

            let not_pred = self.acir_context.sub_var(one, should_insert_value_pred)?;
            let shifted_value_pred = self.acir_context.mul_var(not_pred, value_shifted_index)?;

            let new_value = self.acir_context.add_var(insert_value_pred, shifted_value_pred)?;

            self.acir_context.write_to_memory(result_block_id, &current_index, &new_value)?;

            current_insert_index += 1;
            if inner_elem_size_usize == current_insert_index {
                current_insert_index = 0;
            }
        }

        let AcirValue::DynamicArray(dymamic_array) = slice else {
            panic!("ICE: The input slice should be a dynamic array");
        };

        let result = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: slice_size,
            value_types: dymamic_array.value_types,
            element_type_sizes: dymamic_array.element_type_sizes,
        });

        Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), result])
    }

    /// Removes one or more elements from a slice at a given index.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current slice length  
    /// * `arguments[1]` - Slice contents  
    /// * `arguments[2]` - Removal index (logical element index, not flattened)  
    /// * `result_ids[0]` - Updated slice length  
    /// * `result_ids[1]` - Updated slice contents  
    /// * `result_ids[2..]` - Locations for the removed elements  
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated slice length (decremented by one)  
    /// 2. Updated slice contents with the target elements removed  
    /// 3. The removed elements, in order  
    ///
    /// # Design
    ///
    /// Slices are stored in **flattened form** in memory. Removing requires
    /// shifting a contiguous region of elements downward to overwrite the removed values:
    ///
    /// 1. Compute the flattened remove index:  
    ///    - Multiply the logical remove index by the element size.  
    ///    - Adjust for non-homogenous structures via [Self::get_flattened_index].
    /// 2. Read out the element(s) to be removed:  
    ///    - Iterate over `result_ids[2..(2 + element_size)]`
    ///    - `element_size` refers to the result of [crate::ssa::ir::types::Type::element_size].
    ///    - Use these IDs to fetch the appropriate type information for the values to remove and drive `array_get_value`.
    ///      While extracting the values to remove we compute the total `popped_elements_size` (the flattened width of the removed data).  
    /// 3. For each index in the result slice:  
    ///   - If the index is below the remove index, copy directly.  
    ///   - If the index is at or beyond the removed element, fetch the value from `index + popped_elements_size`
    ///     in the original slice and write it to the current index.  
    ///   - If `index + popped_elements_size` would exceed the slice length we do nothing. This ensures safe access at the tail of the array
    ///     and is safe to do as we are decreasing the slice length which gates slice accesses.
    /// 4. Initialize a new memory block for the resulting slice, ensuring its type information is preserved.  
    pub(super) fn convert_slice_remove(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        // arguments = [slice_length, slice_contents, remove_index]
        let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
        let slice_contents = arguments[1];

        let slice_typ = dfg.type_of_value(slice_contents);
        let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

        let slice = self.convert_value(slice_contents, dfg);
        let remove_index = self.convert_value(arguments[2], dfg).into_var()?;

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

        let slice_size = super::arrays::flattened_value_size(&slice);

        let flat_slice = self.flatten(&slice)?;
        // Compiler sanity check
        assert_eq!(
            flat_slice.len(),
            slice_size,
            "ICE: The read flattened slice should match the computed size"
        );

        // Fetch the flattened index from the user provided index argument.
        let flat_user_index =
            self.get_flattened_index(&slice_typ, slice_contents, remove_index, dfg)?;

        // Fetch the values we are remove from the slice.
        // As we fetch the values we can determine the size of the removed values
        // which we will later use for writing the correct resulting slice.
        let mut popped_elements = Vec::new();
        let mut popped_elements_size = 0;
        // Set a temp index just for fetching from the original slice as `array_get_value` mutates
        // the index internally.
        let mut temp_index = flat_user_index;
        let element_size = slice_typ.element_size();
        for res in &result_ids[2..(2 + element_size)] {
            let element =
                self.array_get_value(&dfg.type_of_value(*res), block_id, &mut temp_index)?;
            let elem_size = super::arrays::flattened_value_size(&element);
            popped_elements_size += elem_size;
            popped_elements.push(element);
        }

        // Go through the entire slice argument and determine what value should be written to the new slice.
        // 1. If the current index is greater than the removal index we must write the next value
        //    from the original slice to the current index
        // 2. For indices beyond the range of the removed elements (i + popped_elements_size >= slice_size),
        //    we skip shifting because there is no element to move.
        //    This prevents out-of-bounds reads from the original slice.
        let result_block_id = self.block_id(result_ids[1]);
        // We expect a preceding check to have been laid down that the remove index is within bounds.
        // In practice `popped_elements_size` should never exceed the `slice_size` but we do a saturating sub to be safe.
        let result_size = slice_size.saturating_sub(popped_elements_size);
        self.initialize_array(result_block_id, result_size, None)?;
        for (i, (current_value, _)) in flat_slice.iter().enumerate().take(result_size) {
            let current_index = self.acir_context.add_constant(i);

            let shifted_index = self.acir_context.add_constant(i + popped_elements_size);

            // Fetch the value from the initial slice
            let value_shifted_index =
                self.acir_context.read_from_memory(block_id, &shifted_index)?;

            let use_shifted_value =
                self.acir_context.more_than_eq_var(current_index, flat_user_index, 64)?;

            let shifted_value_pred =
                self.acir_context.mul_var(value_shifted_index, use_shifted_value)?;
            let not_pred = self.acir_context.sub_var(one, use_shifted_value)?;
            let current_value_pred = self.acir_context.mul_var(not_pred, *current_value)?;

            let new_value = self.acir_context.add_var(shifted_value_pred, current_value_pred)?;

            self.acir_context.write_to_memory(result_block_id, &current_index, &new_value)?;
        }

        let AcirValue::DynamicArray(dymamic_array) = slice else {
            panic!("ICE: The input slice should be a dynamic array");
        };

        let result = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: result_size,
            value_types: dymamic_array.value_types,
            element_type_sizes: dymamic_array.element_type_sizes,
        });

        let mut result = vec![AcirValue::Var(new_slice_length, AcirType::field()), result];
        result.append(&mut popped_elements);

        Ok(result)
    }
}
