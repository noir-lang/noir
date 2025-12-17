use crate::acir::types::{flat_element_types, flat_numeric_types};
use crate::acir::{AcirDynamicArray, AcirValue};
use crate::errors::RuntimeError;
use crate::ssa::ir::types::{NumericType, Type};
use crate::ssa::ir::{dfg::DataFlowGraph, value::ValueId};
use acvm::{AcirField, FieldElement};

use super::Context;

impl Context<'_> {
    /// Pushes one or more elements to the back of a non-nested list.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `arguments[2..]` - Elements to push back
    /// * `result_ids[0]` - Updated list length
    /// * `result_ids[1]` - Update list contents
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated list length (incremented by one)
    /// 2. New list with elements appended
    pub(super) fn convert_list_push_back(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let list_length = self.convert_value(arguments[0], dfg).into_var()?;
        let list_contents = arguments[1];
        let elements_to_push = &arguments[2..];

        // Increase the list length by one to enable accessing more elements in the list.
        let one = self.acir_context.add_constant(FieldElement::one());

        let list = self.convert_value(list_contents, dfg);
        let list_typ = dfg.type_of_value(list_contents);
        let new_list_val = if let Some(len_const) = dfg.get_numeric_constant(arguments[0]) {
            // Length is known at compile time - we can precisely determine where to write
            let mut new_list = self.read_array_with_type(list, &list_typ)?;
            // length of Acir Values list
            let len = len_const.to_u128() as usize * elements_to_push.len();
            for (i, elem) in elements_to_push.iter().enumerate() {
                let element = self.convert_value(*elem, dfg);
                let write_index = len + i;

                // If the array is already large enough, replace the element at the write position.
                // Otherwise, append to the end.
                if write_index < new_list.len() {
                    new_list[write_index] = element;
                } else {
                    new_list.push_back(element);
                }
            }
            AcirValue::Array(new_list)
        } else {
            // Length is not known, we are going to:
            // 1. Push dummy data to the list, so that it's capacity covers for the push_back
            // 2. Generate a DynamicArray corresponding to the new list flattened content
            // 3. Write the elements to push to this array at the correct length
            let value_types = flat_element_types(&list_typ);
            let Type::List(list_types) = &list_typ else {
                unreachable!("ICE: list operation on a non list type");
            };

            let mut elements_var = Vec::new();
            let mut element_size = 0;
            let mut new_list = self.read_array_with_type(list.clone(), &list_typ)?;
            let zero = self.acir_context.add_constant(FieldElement::zero());

            // 1. Convert the elements-to-push into flattened acir_var and at the same time
            // push_back corresponding dummy zero values to the AcirValues list.
            for (elem, ssa_typ) in elements_to_push.iter().zip(list_types.to_vec()) {
                let element = self.convert_value(*elem, dfg);
                element_size += super::arrays::flattened_value_size(&element);
                match element {
                    AcirValue::Var(acir_var, acir_type) => {
                        new_list.push_back(AcirValue::Var(zero, acir_type));
                        elements_var.push(acir_var);
                    }
                    AcirValue::Array(vector) => {
                        let zero_value = self.array_zero_value(&ssa_typ)?;
                        new_list.push_back(zero_value);
                        for acir_value in vector {
                            let acir_vars = self.flatten(&acir_value)?;
                            elements_var.extend(acir_vars);
                        }
                    }
                    AcirValue::DynamicArray(_) => {
                        unimplemented!("pushing a dynamic array into a list is not yet supported");
                    }
                }
            }

            // The actual flattened size of new_list after the dummy push_back
            let new_list_array = AcirValue::Array(new_list);
            let len = super::arrays::flattened_value_size(&new_list_array);

            // 2. Copy the list into an AcirDynamicArray
            // Generates the element_type_sizes array
            let element_type_sizes =
                if super::arrays::array_has_constant_element_size(&list_typ).is_none() {
                    Some(self.init_element_type_sizes_array(
                        &list_typ,
                        list_contents,
                        Some(&new_list_array),
                        dfg,
                    )?)
                } else {
                    None
                };

            // The block ID for the new list is the one for the resulting list
            let block_id = self.block_id(result_ids[1]);
            self.initialize_array(block_id, len, Some(new_list_array))?;
            let flattened_dynamic_array =
                AcirDynamicArray { block_id, len, value_types, element_type_sizes };

            // 3. Write to the dynamic array

            // 3.1 Computes the flatten_idx where to write into the dynamic array:
            // Use element_type_size if it exists; convert the user index (list_length) into the AcirValues index,
            // and then flatten it with element_type_size
            let mut flatten_idx = if let Some(element_type_sizes) = element_type_sizes {
                let predicate_index = self
                    .acir_context
                    .mul_var(list_length, self.current_side_effects_enabled_var)?;
                let acir_element_size = self.acir_context.add_constant(elements_to_push.len());
                let acir_value_index =
                    self.acir_context.mul_var(predicate_index, acir_element_size)?;
                self.acir_context
                    .read_from_memory(element_type_sizes, &acir_value_index)
                    .map_err(RuntimeError::from)?
            } else {
                // If it does not exist; the array is homogenous and we can simply multiply by size of the array elements
                let element_size_var = self.acir_context.add_constant(element_size);
                self.acir_context.mul_var(list_length, element_size_var)?
            };
            // Write the elements to the dynamic array
            for element in &elements_var {
                self.acir_context.write_to_memory(block_id, &flatten_idx, element)?;
                flatten_idx = self.acir_context.add_var(flatten_idx, one)?;
            }
            AcirValue::DynamicArray(flattened_dynamic_array)
        };

        let new_list_length = self.acir_context.add_var(list_length, one)?;
        Ok(vec![AcirValue::Var(new_list_length, NumericType::length_type()), new_list_val])
    }

    /// Pushes one or more elements to the front of a non-nested list.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `arguments[2..]` - Elements to push to the front
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated list length (incremented by one)
    /// 2. New list with elements prepended
    pub(super) fn convert_list_push_front(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let list_length = self.convert_value(arguments[0], dfg).into_var()?;
        let list_contents = arguments[1];
        let elements_to_push = &arguments[2..];

        // Increase the list length by one to enable accessing more elements in the list.
        let one = self.acir_context.add_constant(FieldElement::one());
        let new_list_length = self.acir_context.add_var(list_length, one)?;

        let list = self.convert_value(list_contents, dfg);
        let list_type = dfg.type_of_value(list_contents);
        let mut new_list = self.read_array_with_type(list, &list_type)?;

        // We must directly push front elements for non-nested lists
        for elem in elements_to_push.iter().rev() {
            let element = self.convert_value(*elem, dfg);
            new_list.push_front(element);
        }

        let new_list_val = AcirValue::Array(new_list);

        Ok(vec![AcirValue::Var(new_list_length, NumericType::length_type()), new_list_val])
    }

    /// Removes and returns one or more elements from the back of a non-nested list.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `result_ids[0]` - Updated list length
    /// * `result_ids[1]` - Update list contents
    /// * `result_ids[2..]` - Locations where popped elements will be stored
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated list length (decremented by one)
    /// 2. Updated list contents with the back elements removed
    /// 3. Popped elements in order
    ///
    /// # Design
    ///
    /// The list is represented in **flattened form** in memory. Popping the back
    /// involves:
    ///
    /// 1. Decrementing the list length by one.
    /// 2. Using the decremented length as an offset for the elements to remove.
    /// 3. Read out the elements located at that offset.
    ///
    /// The `result_ids` provided by the SSA to fetch the appropriate type information to be popped.
    /// The `result_ids` encode the type/shape of the removed element.
    ///
    /// # Empty List Handling
    ///
    /// If the list has zero length, this function skips the memory read and returns zero values.
    /// It asserts that the current side effects must be disabled (predicate = 0), otherwise fails
    /// with "cannot pop from a list with length 0". This prevents reading from empty memory blocks
    /// which would cause "Index out of bounds" errors.
    pub(super) fn convert_list_pop_back(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let list_length_var = arguments[0];
        let list_contents = arguments[1];

        let list_value = self.convert_value(list_length_var, dfg);
        let list_length = list_value.clone().into_var()?;
        let block_id = self.ensure_array_is_initialized(list_contents, dfg)?;
        let list = self.convert_value(list_contents, dfg);

        if self.has_zero_length(list_contents, dfg) {
            // Make sure this code is disabled, or fail with "Index out of bounds".
            let msg = "cannot pop from a list with length 0".to_string();
            self.acir_context.assert_zero_var(self.current_side_effects_enabled_var, msg)?;

            // Fill the result with default values.
            let mut results = Vec::with_capacity(result_ids.len());

            // The results shall be: [new len, new list, ...popped]
            results.push(list_value);
            results.push(list);

            for result_id in &result_ids[2..] {
                let result_type = dfg.type_of_value(*result_id);
                let result_zero = self.array_zero_value(&result_type)?;
                results.push(result_zero);
            }

            return Ok(results);
        }

        // For unknown length under a side effect variable, we want to multiply with the side effect variable
        // to ensure we don't end up trying to look up an item at index -1, when the semantic length is 0.
        let is_unknown_length = dfg.get_numeric_constant(list_length_var).is_none();

        let one = self.acir_context.add_constant(FieldElement::one());
        let mut new_list_length = self.acir_context.sub_var(list_length, one)?;

        if is_unknown_length {
            new_list_length = self
                .acir_context
                .mul_var(new_list_length, self.current_side_effects_enabled_var)?;
        }

        // For a pop back operation we want to fetch from the `length - 1` as this is the
        // last valid index that can be accessed in a list. After the pop back operation
        // the elements stored at that index will no longer be able to be accessed.
        let mut var_index = new_list_length;

        let list_type = dfg.type_of_value(list_contents);
        let item_size = list_type.element_types();
        // Must read from the flattened last index of the list in case the list contains nested arrays.
        let flat_item_size: u32 = item_size.iter().map(|typ| typ.flattened_size()).sum();
        let item_size = self.acir_context.add_constant(flat_item_size);
        var_index = self.acir_context.mul_var(var_index, item_size)?;

        let mut popped_elements = Vec::new();
        for res in &result_ids[2..] {
            let elem = self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
            popped_elements.push(elem);
        }

        let mut new_list = self.read_array_with_type(list, &list_type)?;
        for _ in 0..popped_elements.len() {
            new_list.pop_back();
        }

        let mut results = vec![
            AcirValue::Var(new_list_length, NumericType::length_type()),
            AcirValue::Array(new_list),
        ];
        results.append(&mut popped_elements);

        Ok(results)
    }

    /// Removes and returns one or more elements from the front of a non-nested list.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `result_ids[..element_size]` - Locations for the popped elements
    /// * `result_ids[element_size]` - Updated list length
    /// * `result_ids[element_size + 1]` - Updated list contents
    ///
    /// `element_size` refers to the result of [crate::ssa::ir::types::Type::element_size].
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Popped elements in order
    /// 2. Updated list length (decremented by one)
    /// 3. Updated list contents with the front elements removed
    ///
    /// # Design
    ///
    /// Lists are stored in **flattened form** in memory. To pop from the front:
    ///
    /// 1. Decrement the list length by the size of one element.
    /// 2. Read out the first `element_size` values at index `0`.
    /// 3. Shift the update list's memory forward by `element_size` slots to represent the updated list.
    ///
    /// Unlike in [Self::convert_list_pop_back], the returned list contents differ from the input:
    /// the underlying array is logically truncated at the *front* rather than
    /// the back. The `result_ids` ensure that this logical shift is applied
    /// consistently with the element's type.
    ///
    /// # Empty List Handling
    ///
    /// If the list has zero length, this function skips the memory read and returns zero values.
    /// It asserts that the current side effects must be disabled (predicate = 0), otherwise fails
    /// with "cannot pop from a list with length 0". This prevents reading from empty memory blocks
    /// which would cause "Index out of bounds" errors.
    pub(super) fn convert_list_pop_front(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let list_length = self.convert_value(arguments[0], dfg).into_var()?;
        let list_contents = arguments[1];

        let list_typ = dfg.type_of_value(list_contents);
        let block_id = self.ensure_array_is_initialized(list_contents, dfg)?;

        // Check if we're trying to pop from an empty list
        if self.has_zero_length(list_contents, dfg) {
            // Make sure this code is disabled, or fail with "Index out of bounds".
            let msg = "cannot pop from a list with length 0".to_string();
            self.acir_context.assert_zero_var(self.current_side_effects_enabled_var, msg)?;

            // Fill the result with default values.
            let mut results = Vec::with_capacity(result_ids.len());

            let element_size = list_typ.element_size();
            // For pop_front, results order is: [popped_elements..., new_len, new_list]
            for result_id in &result_ids[..element_size] {
                let result_type = dfg.type_of_value(*result_id);
                let result_zero = self.array_zero_value(&result_type)?;
                results.push(result_zero);
            }

            let list_value = self.convert_value(arguments[0], dfg);
            results.push(list_value);

            let list = self.convert_value(list_contents, dfg);
            results.push(list);

            return Ok(results);
        }

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_list_length = self.acir_context.sub_var(list_length, one)?;

        let list = self.convert_value(list_contents, dfg);

        let mut new_list = self.read_array_with_type(list, &list_typ)?;
        let element_size = list_typ.element_size();

        let mut popped_elements: Vec<AcirValue> = Vec::new();
        let mut var_index = self.acir_context.add_constant(FieldElement::zero());
        // Fetch the values we are popping off of the list.
        // In the case of non-nested list the logic is simple as we do not
        // need to account for the internal list sizes or flattening the index.
        for res in &result_ids[..element_size] {
            let element =
                self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
            popped_elements.push(element);
        }

        let popped_elements_size = popped_elements.len();

        new_list = new_list.slice(popped_elements_size..);
        popped_elements.push(AcirValue::Var(new_list_length, NumericType::length_type()));
        popped_elements.push(AcirValue::Array(new_list));

        Ok(popped_elements)
    }

    /// Inserts one or more elements into a list at a given index.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `arguments[2]` - Insert index (logical element index, not flattened)
    /// * `arguments[3..]` - Elements to insert
    /// * `result_ids[0]` - Updated list length
    /// * `result_ids[1]` - Updated list contents
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated list length (incremented by one)
    /// 2. Updated list contents with the new elements inserted at the given index
    ///
    /// # Design
    ///
    /// Lists are represented in **flattened form** in memory. Inserting requires
    /// shifting a contiguous region of elements upward to make room for the new ones:
    ///
    /// 1. Compute the flattened insert index:
    ///    - Multiply the logical insert index by the element size.
    ///    - Adjust for non-homogenous structures via [Self::get_flattened_index].
    /// 2. Flatten the new elements (`flattened_elements`)
    /// 3. For each position in the result list:
    ///    - If below the insert index, copy from the original list.
    ///    - If within the insertion window, write values from `flattened_elements`.
    ///    - If above the window, shift elements upward by the size of the inserted data.
    /// 4. Initialize a new memory block for the resulting list, ensuring its type information is preserved.
    ///
    /// # Empty List Handling
    ///
    /// If the list has zero length, this function skips the memory read and returns zero values.
    /// It asserts that the current side effects must be disabled (predicate = 0), otherwise fails
    /// with "Index out of bounds, list has size 0". This prevents reading from empty memory blocks
    /// which would cause "Index out of bounds" errors.
    pub(super) fn convert_list_insert(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let list_length = self.convert_value(arguments[0], dfg).into_var()?;
        let list_contents = arguments[1];

        let list_typ = dfg.type_of_value(list_contents);
        let block_id = self.ensure_array_is_initialized(list_contents, dfg)?;

        // Check if we're trying to insert into an empty list
        if self.has_zero_length(list_contents, dfg) {
            // Make sure this code is disabled, or fail with "Index out of bounds".
            let msg = "Index out of bounds, list has size 0".to_string();
            self.acir_context.assert_zero_var(self.current_side_effects_enabled_var, msg)?;

            // Fill the result with default values.
            let mut results = Vec::with_capacity(result_ids.len());

            // For insert, results are: [new_len, new_list]
            let list_length_value = self.convert_value(arguments[0], dfg);
            results.push(list_length_value);

            let list = self.convert_value(list_contents, dfg);
            results.push(list);

            return Ok(results);
        }

        let list = self.convert_value(list_contents, dfg);
        let insert_index = self.convert_value(arguments[2], dfg).into_var()?;

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_list_length = self.acir_context.add_var(list_length, one)?;

        let mut list_size = super::arrays::flattened_value_size(&list);

        let elements_to_insert = &arguments[3..];

        // Fetch the flattened index from the user provided index argument.
        let item_size = self.acir_context.add_constant(elements_to_insert.len());
        let insert_index = self.acir_context.mul_var(insert_index, item_size)?;
        let flat_user_index =
            self.get_flattened_index(&list_typ, list_contents, insert_index, dfg)?;

        // Determine the elements we need to write into our resulting dynamic array.
        // We need to a fully flat list of AcirVar's as a dynamic array is represented with flat memory.
        let mut inner_elem_size_usize = 0;
        let mut flattened_elements = Vec::new();
        for elem in elements_to_insert {
            let element = self.convert_value(*elem, dfg);
            // Flatten into (AcirVar, NumericType) pairs
            let flat_element = self.flatten(&element)?;
            let elem_size = flat_element.len();
            inner_elem_size_usize += elem_size;
            list_size += elem_size;
            for var in flat_element {
                flattened_elements.push(var);
            }
        }
        let inner_elem_size = self.acir_context.add_constant(inner_elem_size_usize);
        // Set the maximum flattened index at which a new element should be inserted.
        let max_flat_user_index = self.acir_context.add_var(flat_user_index, inner_elem_size)?;

        // Go through the entire list argument and determine what value should be written to the new list.
        // 1. If we are below the starting insertion index we should insert the value that was already
        //    in the original list.
        // 2. If we are above the starting insertion index but below the max insertion index we should insert
        //    the flattened element arguments.
        // 3. If we are above the max insertion index we should insert the previous value from the original list,
        //    as during an insertion we want to shift all elements after the insertion up an index.
        let result_block_id = self.block_id(result_ids[1]);
        self.initialize_array(result_block_id, list_size, None)?;
        let mut current_insert_index = 0;

        // This caches each `is_after_insert` var for each index for an optimization that is
        // explained below, above `is_after_insert`.
        let mut cached_is_after_inserts = Vec::with_capacity(list_size);

        for i in 0..list_size {
            let current_index = self.acir_context.add_constant(i);

            // Check that we are above the lower bound of the insertion index
            let is_after_insert =
                self.acir_context.more_than_eq_var(current_index, flat_user_index, 64)?;
            cached_is_after_inserts.push(is_after_insert);

            // Check that we are below the upper bound of the insertion index
            let is_before_insert = if i >= inner_elem_size_usize {
                // Optimization: we first note that `max_flat_user_index = flat_user_index + inner_elem_size`.
                // Then we note that at each index we do these comparisons:
                // - is_after_insert: `i >= flat_user_index`
                // - is_before_insert: `i < (flat_user_index + inner_elem_size)`
                //
                // As `i` is incremented, for example to `i + n`, we get:
                // - is_before_insert: `i + n < (flat_user_index + inner_elem_size)`
                // If `n == inner_elem_size` then we have:
                // - is_before_insert: `i + n < (flat_user_index + n)` which is equivalent to:
                // - is_before_insert: `i < flat_user_index`
                // Then we note that this is the opposite of `i >= flat_user_index`.
                // So once `i >= inner_elem_size` we can use the previously made comparisons, negated,
                // instead of performing them again (for dynamic indexes they incur a brillig call).
                let cached_is_after_insert = cached_is_after_inserts[i - inner_elem_size_usize];
                self.acir_context.sub_var(one, cached_is_after_insert)?
            } else {
                self.acir_context.less_than_var(current_index, max_flat_user_index, 64)?
            };

            // Read from the original list the value we want to insert into our new list.
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

        let element_type_sizes = if super::arrays::array_has_constant_element_size(&list_typ)
            .is_none()
        {
            Some(self.init_element_type_sizes_array(&list_typ, list_contents, Some(&list), dfg)?)
        } else {
            None
        };

        let value_types = flat_numeric_types(&list_typ);

        let result = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: list_size,
            value_types,
            element_type_sizes,
        });

        Ok(vec![AcirValue::Var(new_list_length, NumericType::length_type()), result])
    }

    /// Removes one or more elements from a list at a given index.
    ///
    /// # Arguments
    ///
    /// * `arguments[0]` - Current list length
    /// * `arguments[1]` - List contents
    /// * `arguments[2]` - Removal index (logical element index, not flattened)
    /// * `result_ids[0]` - Updated list length
    /// * `result_ids[1]` - Updated list contents
    /// * `result_ids[2..]` - Locations for the removed elements
    ///
    /// # Returns
    ///
    /// A vector of [AcirValue]s containing:
    /// 1. Updated list length (decremented by one)
    /// 2. Updated list contents with the target elements removed
    /// 3. The removed elements, in order
    ///
    /// # Design
    ///
    /// Lists are stored in **flattened form** in memory. Removing requires
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
    /// 3. For each index in the result list:
    ///   - If the index is below the remove index, copy directly.
    ///   - If the index is at or beyond the removed element, fetch the value from `index + popped_elements_size`
    ///     in the original list and write it to the current index.
    ///   - If `index + popped_elements_size` would exceed the list length we do nothing. This ensures safe access at the tail of the array
    ///     and is safe to do as we are decreasing the list length which gates list accesses.
    /// 4. Initialize a new memory block for the resulting list, ensuring its type information is preserved.
    ///
    /// # Empty List Handling
    ///
    /// If the list has zero length, this function skips the memory read and returns zero values.
    /// It asserts that the current side effects must be disabled (predicate = 0), otherwise fails
    /// with "Index out of bounds, list has size 0". This prevents reading from empty memory blocks
    /// which would cause "Index out of bounds" errors.
    pub(super) fn convert_list_remove(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        // arguments = [list_length, list_contents, remove_index]
        let list_length = self.convert_value(arguments[0], dfg).into_var()?;
        let list_contents = arguments[1];

        let list_typ = dfg.type_of_value(list_contents);
        let block_id = self.ensure_array_is_initialized(list_contents, dfg)?;

        // Check if we're trying to remove from an empty list
        if self.has_zero_length(list_contents, dfg) {
            // Make sure this code is disabled, or fail with "Index out of bounds".
            let msg = "Index out of bounds, list has size 0".to_string();
            self.acir_context.assert_zero_var(self.current_side_effects_enabled_var, msg)?;

            // Fill the result with default values.
            let mut results = Vec::with_capacity(result_ids.len());

            // For remove, results are: [new_len, new_list, ...removed_elements]
            let list_length_value = self.convert_value(arguments[0], dfg);
            results.push(list_length_value);

            let list = self.convert_value(list_contents, dfg);
            results.push(list);

            // Add zero values for removed elements
            for result_id in &result_ids[2..] {
                let result_type = dfg.type_of_value(*result_id);
                let result_zero = self.array_zero_value(&result_type)?;
                results.push(result_zero);
            }

            return Ok(results);
        }

        let list = self.convert_value(list_contents, dfg);
        let remove_index = self.convert_value(arguments[2], dfg).into_var()?;

        let one = self.acir_context.add_constant(FieldElement::one());
        let new_list_length = self.acir_context.sub_var(list_length, one)?;

        let list_size = super::arrays::flattened_value_size(&list);

        let flat_list = self.flatten(&list)?;
        // Compiler sanity check
        assert_eq!(
            flat_list.len(),
            list_size,
            "ICE: The read flattened list should match the computed size"
        );

        let item_size = list_typ.element_size();
        let item_size = self.acir_context.add_constant(item_size);
        let remove_index = self.acir_context.mul_var(remove_index, item_size)?;

        // Fetch the flattened index from the user provided index argument.
        let flat_user_index =
            self.get_flattened_index(&list_typ, list_contents, remove_index, dfg)?;

        // Fetch the values we are remove from the list.
        // As we fetch the values we can determine the size of the removed values
        // which we will later use for writing the correct resulting list.
        let mut popped_elements = Vec::new();
        let mut popped_elements_size = 0;
        // Set a temp index just for fetching from the original list as `array_get_value` mutates
        // the index internally.
        let mut temp_index = flat_user_index;
        let element_size = list_typ.element_size();
        for res in &result_ids[2..(2 + element_size)] {
            let element =
                self.array_get_value(&dfg.type_of_value(*res), block_id, &mut temp_index)?;
            let elem_size = super::arrays::flattened_value_size(&element);
            popped_elements_size += elem_size;
            popped_elements.push(element);
        }

        // Go through the entire list argument and determine what value should be written to the new list.
        // 1. If the current index is greater than the removal index we must write the next value
        //    from the original list to the current index
        // 2. For indices beyond the range of the removed elements (i + popped_elements_size >= list_size),
        //    we skip shifting because there is no element to move.
        //    This prevents out-of-bounds reads from the original list.
        let result_block_id = self.block_id(result_ids[1]);
        // We expect a preceding check to have been laid down that the remove index is within bounds.
        // In practice `popped_elements_size` should never exceed the `list_size` but we do a saturating sub to be safe.
        let result_size = list_size.saturating_sub(popped_elements_size);
        self.initialize_array(result_block_id, result_size, None)?;
        for (i, current_value) in flat_list.iter().enumerate().take(result_size) {
            let current_index = self.acir_context.add_constant(i);

            let shifted_index = self.acir_context.add_constant(i + popped_elements_size);

            // Fetch the value from the initial list
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

        let element_type_sizes = if super::arrays::array_has_constant_element_size(&list_typ)
            .is_none()
        {
            Some(self.init_element_type_sizes_array(&list_typ, list_contents, Some(&list), dfg)?)
        } else {
            None
        };

        let value_types = flat_numeric_types(&list_typ);

        let result = AcirValue::DynamicArray(AcirDynamicArray {
            block_id: result_block_id,
            len: result_size,
            value_types,
            element_type_sizes,
        });

        let mut result = vec![AcirValue::Var(new_list_length, NumericType::length_type()), result];
        result.append(&mut popped_elements);

        Ok(result)
    }
}
