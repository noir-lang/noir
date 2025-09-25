use acvm::acir::circuit::opcodes::AcirFunctionId;
use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;

use crate::acir::AcirVar;
use crate::brillig::brillig_gen::brillig_fn::FunctionContext as BrilligFunctionContext;
use crate::brillig::brillig_gen::gen_brillig_for;
use crate::brillig::brillig_ir::artifact::BrilligParameter;
use crate::errors::{RuntimeError, SsaReport};
use crate::ssa::ir::instruction::Hint;
use crate::ssa::ir::value::Value;
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::{Function, FunctionId, RuntimeType},
    instruction::{Instruction, Intrinsic},
    types::{NumericType, Type},
    value::ValueId,
};
use crate::ssa::ssa_gen::Ssa;

use super::{
    Context, arrays,
    types::{AcirDynamicArray, AcirType, AcirValue},
};

impl Context<'_> {
    pub(super) fn convert_ssa_call(
        &mut self,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        result_ids: &[ValueId],
    ) -> Result<Vec<SsaReport>, RuntimeError> {
        let warnings = Vec::new();

        match instruction {
            Instruction::Call { func, arguments } => {
                let function_value = &dfg[*func];
                match function_value {
                    Value::Function(id) => {
                        let func = &ssa.functions[id];
                        match func.runtime() {
                            RuntimeType::Acir(inline_type) => {
                                assert!(
                                    inline_type.is_entry_point(),
                                    "ICE: Got a call to an ACIR function {} named {} that should have already been inlined",
                                    id,
                                    func.name()
                                );

                                self.handle_acir_function_call(
                                    id, arguments, result_ids, ssa, dfg,
                                )?;
                            }
                            RuntimeType::Brillig(_) => {
                                self.handle_brillig_function_call(
                                    func, arguments, result_ids, dfg,
                                )?;
                            }
                        }
                    }
                    Value::Intrinsic(intrinsic) => {
                        let outputs = self
                            .convert_ssa_intrinsic_call(*intrinsic, arguments, dfg, result_ids)?;

                        assert_eq!(result_ids.len(), outputs.len());
                        self.handle_ssa_call_outputs(result_ids, outputs, dfg)?;
                    }
                    Value::ForeignFunction(_) => unreachable!(
                        "Frontend should remove any oracle calls from constrained functions"
                    ),

                    _ => unreachable!("expected calling a function but got {function_value:?}"),
                }
            }
            _ => unreachable!("expected calling a call instruction"),
        }
        Ok(warnings)
    }

    fn handle_acir_function_call(
        &mut self,
        func_id: &FunctionId,
        arguments: &[ValueId],
        result_ids: &[ValueId],
        ssa: &Ssa,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        // Check that we are not attempting to return a slice from
        // an unconstrained runtime to a constrained runtime
        for result_id in result_ids {
            if dfg.type_of_value(*result_id).contains_slice_element() {
                return Err(RuntimeError::UnconstrainedSliceReturnToConstrained {
                    call_stack: self.acir_context.get_call_stack(),
                });
            }
        }

        let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
        let output_count: usize = result_ids
            .iter()
            .map(|result_id| dfg.type_of_value(*result_id).flattened_size() as usize)
            .sum();

        let Some(acir_function_id) = ssa.get_entry_point_index(func_id) else {
            unreachable!(
                "Expected an associated final index for call to acir function {func_id} with args {arguments:?}"
            );
        };

        let output_vars = self.acir_context.call_acir_function(
            AcirFunctionId(acir_function_id),
            inputs,
            output_count,
            self.current_side_effects_enabled_var,
        )?;

        let output_values = self.convert_vars_to_values(output_vars, dfg, result_ids);
        self.handle_ssa_call_outputs(result_ids, output_values, dfg)
    }

    fn handle_brillig_function_call(
        &mut self,
        func: &Function,
        arguments: &[ValueId],
        result_ids: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        // Convert SSA arguments to Brillig parameters
        let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
        let arguments = self.gen_brillig_parameters(arguments, dfg);
        let outputs: Vec<AcirType> =
            vecmap(result_ids, |result_id| dfg.type_of_value(*result_id).into());

        // Reuse or generate Brillig code
        let output_values = if let Some(generated_pointer) =
            self.shared_context.generated_brillig_pointer(func.id(), arguments.clone())
        {
            let code = self.shared_context.generated_brillig(generated_pointer.as_usize());
            self.acir_context.brillig_call(
                self.current_side_effects_enabled_var,
                code,
                inputs,
                outputs,
                false,
                *generated_pointer,
                None,
            )?
        } else {
            let code =
                gen_brillig_for(func, arguments.clone(), self.brillig, self.brillig_options)?;
            let generated_pointer = self.shared_context.new_generated_pointer();
            let output_values = self.acir_context.brillig_call(
                self.current_side_effects_enabled_var,
                &code,
                inputs,
                outputs,
                false,
                generated_pointer,
                None,
            )?;
            self.shared_context.insert_generated_brillig(
                func.id(),
                arguments,
                generated_pointer,
                code,
            );
            output_values
        };

        assert_eq!(result_ids.len(), output_values.len(), "Brillig output length mismatch");
        self.handle_ssa_call_outputs(result_ids, output_values, dfg)
    }

    pub(super) fn gen_brillig_parameters(
        &self,
        values: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Vec<BrilligParameter> {
        values
            .iter()
            .map(|&value_id| {
                let typ = dfg.type_of_value(value_id);
                if let Type::Slice(item_types) = typ {
                    let len = match self
                        .ssa_values
                        .get(&value_id)
                        .expect("ICE: Unknown slice input to brillig")
                    {
                        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => *len,
                        AcirValue::Array(array) => array.len(),
                        _ => unreachable!("ICE: Slice value is not an array"),
                    };

                    BrilligParameter::Slice(
                        item_types
                            .iter()
                            .map(BrilligFunctionContext::ssa_type_to_parameter)
                            .collect(),
                        len / item_types.len(),
                    )
                } else {
                    BrilligFunctionContext::ssa_type_to_parameter(&typ)
                }
            })
            .collect()
    }

    /// Returns a vector of `AcirVar`s constrained to be result of the function call.
    ///
    /// The function being called is required to be intrinsic.
    fn convert_ssa_intrinsic_call(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        match intrinsic {
            Intrinsic::Hint(Hint::BlackBox) => {
                // Identity function; at the ACIR level this is a no-op, it only affects the SSA.
                assert_eq!(
                    arguments.len(),
                    result_ids.len(),
                    "ICE: BlackBox input and output lengths should match."
                );
                Ok(arguments.iter().map(|v| self.convert_value(*v, dfg)).collect())
            }
            Intrinsic::BlackBox(black_box) => {
                // Slice arguments to blackbox functions would break the following logic (due to being split over two `ValueIds`)
                // No blackbox functions currently take slice arguments so we have an assertion here to catch if this changes in the future.
                assert!(
                    !arguments.iter().any(|arg| matches!(dfg.type_of_value(*arg), Type::Slice(_))),
                    "ICE: Slice arguments passed to blackbox function"
                );

                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));

                let output_count = result_ids.iter().fold(0usize, |sum, result_id| {
                    sum + dfg.type_of_value(*result_id).flattened_size() as usize
                });

                let vars = self.acir_context.black_box_function(
                    black_box,
                    inputs,
                    None,
                    output_count,
                    Some(self.current_side_effects_enabled_var),
                )?;

                Ok(self.convert_vars_to_values(vars, dfg, result_ids))
            }
            Intrinsic::ApplyRangeConstraint => {
                unreachable!(
                    "ICE: `Intrinsic::ApplyRangeConstraint` calls should be transformed into an `Instruction::RangeCheck`"
                );
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let radix = self.convert_value(arguments[1], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToRadix result must be an array");
                };

                self.acir_context
                    .radix_decompose(
                        endian,
                        field,
                        radix,
                        array_length,
                        result_type[0].clone().into(),
                    )
                    .map(|array| vec![array])
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToBits result must be an array");
                };

                self.acir_context
                    .bit_decompose(endian, field, array_length, result_type[0].clone().into())
                    .map(|array| vec![array])
            }
            Intrinsic::ArrayLen => {
                let len = match self.convert_value(arguments[0], dfg) {
                    AcirValue::Var(_, _) => {
                        unreachable!("Non-array passed to array.len() method")
                    }
                    AcirValue::Array(values) => values.len(),
                    AcirValue::DynamicArray(array) => array.len,
                };
                Ok(vec![AcirValue::Var(self.acir_context.add_constant(len), AcirType::field())])
            }
            Intrinsic::AsSlice => {
                let slice_contents = arguments[0];
                let slice_typ = dfg.type_of_value(slice_contents);
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice_length = self.flattened_size(slice_contents, dfg);
                let slice_length = self.acir_context.add_constant(slice_length);

                let acir_value = self.convert_value(slice_contents, dfg);
                let result = self.read_array(acir_value)?;

                Ok(vec![AcirValue::Var(slice_length, AcirType::field()), AcirValue::Array(result)])
            }
            Intrinsic::SlicePushBack => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];
                let elements_to_push = &arguments[2..];

                let slice_typ = dfg.type_of_value(slice_contents);

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

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
                let new_elem_size = arrays::flattened_value_size(&new_slice_val);
                let value_types = new_slice_val.clone().flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    new_elem_size,
                    "ICE: Value types array must match new slice size"
                );

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
            }
            Intrinsic::SlicePushFront => {
                // arguments = [slice_length, slice_contents, ...elements_to_push]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];
                let elements_to_push = &arguments[2..];
                let slice_typ = dfg.type_of_value(slice_contents);
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

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
                let new_slice_size = arrays::flattened_value_size(&new_slice_val);

                let value_types = new_slice_val.clone().flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    new_slice_size,
                    "ICE: Value types array must match new slice size"
                );

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), new_slice_val])
            }
            Intrinsic::SlicePopBack => {
                // arguments = [slice_length, slice_contents]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;
                // For a pop back operation we want to fetch from the `length - 1` as this is the
                // last valid index that can be accessed in a slice. After the pop back operation
                // the elements stored at that index will no longer be able to be accessed.
                let mut var_index = new_slice_length;

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;
                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let mut popped_elements = Vec::new();
                for res in &result_ids[2..] {
                    let elem =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut var_index)?;
                    popped_elements.push(elem);
                }

                let slice = self.convert_value(slice_contents, dfg);
                let new_slice = self.read_array(slice)?;

                let mut results = vec![
                    AcirValue::Var(new_slice_length, AcirType::field()),
                    AcirValue::Array(new_slice),
                ];
                results.append(&mut popped_elements);

                Ok(results)
            }
            Intrinsic::SlicePopFront => {
                // arguments = [slice_length, slice_contents]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

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
                    let elem_size = arrays::flattened_value_size(&element);
                    popped_elements_size += elem_size;
                    popped_elements.push(element);
                }

                // It is expected that the `popped_elements_size` is the flattened size of the elements,
                // as the input slice should be a dynamic array which is represented by flat memory.
                // However in some cases the input slice is an Array with a nested structure,
                // in which case we only need to pop the items that represent a single entry.
                let popped_elements_size =
                    if is_dynamic { popped_elements_size } else { element_size };

                new_slice = new_slice.slice(popped_elements_size..);

                popped_elements.push(AcirValue::Var(new_slice_length, AcirType::field()));
                popped_elements.push(AcirValue::Array(new_slice));

                Ok(popped_elements)
            }
            Intrinsic::SliceInsert => {
                // arguments = [slice_length, slice_contents, insert_index, ...elements_to_insert]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let insert_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.add_var(slice_length, one)?;

                let slice_size = arrays::flattened_value_size(&slice);

                // Fetch the flattened index from the user provided index argument.
                let element_size = slice_typ.element_size();
                let element_size_var = self.acir_context.add_constant(element_size);
                let flat_insert_index =
                    self.acir_context.mul_var(insert_index, element_size_var)?;
                let flat_user_index =
                    self.get_flattened_index(&slice_typ, slice_contents, flat_insert_index, dfg)?;

                let elements_to_insert = &arguments[3..];
                // Determine the elements we need to write into our resulting dynamic array.
                // We need to a fully flat list of AcirVar's as a dynamic array is represented with flat memory.
                let mut inner_elem_size_usize = 0;
                let mut flattened_elements = Vec::new();
                for elem in elements_to_insert {
                    let element = self.convert_value(*elem, dfg);
                    let elem_size = arrays::flattened_value_size(&element);
                    inner_elem_size_usize += elem_size;
                    let mut flat_elem = element.flatten().into_iter().map(|(var, _)| var).collect();
                    flattened_elements.append(&mut flat_elem);
                }
                let inner_elem_size = self.acir_context.add_constant(inner_elem_size_usize);
                // Set the maximum flattened index at which a new element should be inserted.
                let max_flat_user_index =
                    self.acir_context.add_var(flat_user_index, inner_elem_size)?;

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
                    let greater_eq_than_idx =
                        self.acir_context.more_than_eq_var(current_index, flat_user_index, 64)?;
                    // Check that we are below the upper bound of the insertion index
                    let less_than_idx =
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

                        let use_shifted_index_pred = self
                            .acir_context
                            .mul_var(index_minus_elem_size, greater_eq_than_idx)?;

                        let not_pred = self.acir_context.sub_var(one, greater_eq_than_idx)?;
                        let use_current_index_pred =
                            self.acir_context.mul_var(not_pred, current_index)?;

                        self.acir_context.add_var(use_shifted_index_pred, use_current_index_pred)?
                    };

                    let value_shifted_index =
                        self.acir_context.read_from_memory(block_id, &shifted_index)?;

                    // Final predicate to determine whether we are within the insertion bounds
                    let should_insert_value_pred =
                        self.acir_context.mul_var(greater_eq_than_idx, less_than_idx)?;
                    let insert_value_pred = self.acir_context.mul_var(
                        flattened_elements[current_insert_index],
                        should_insert_value_pred,
                    )?;

                    let not_pred = self.acir_context.sub_var(one, should_insert_value_pred)?;
                    let shifted_value_pred =
                        self.acir_context.mul_var(not_pred, value_shifted_index)?;

                    let new_value =
                        self.acir_context.add_var(insert_value_pred, shifted_value_pred)?;

                    self.acir_context.write_to_memory(
                        result_block_id,
                        &current_index,
                        &new_value,
                    )?;

                    current_insert_index += 1;
                    if inner_elem_size_usize == current_insert_index {
                        current_insert_index = 0;
                    }
                }

                let element_type_sizes =
                    if arrays::array_has_constant_element_size(&slice_typ).is_none() {
                        Some(self.init_element_type_sizes_array(
                            &slice_typ,
                            slice_contents,
                            Some(&slice),
                            dfg,
                        )?)
                    } else {
                        None
                    };

                let value_types = slice.flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    slice_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: slice_size,
                    value_types,
                    element_type_sizes,
                });

                Ok(vec![AcirValue::Var(new_slice_length, AcirType::field()), result])
            }
            Intrinsic::SliceRemove => {
                // arguments = [slice_length, slice_contents, remove_index]
                let slice_length = self.convert_value(arguments[0], dfg).into_var()?;
                let slice_contents = arguments[1];

                let slice_typ = dfg.type_of_value(slice_contents);
                let block_id = self.ensure_array_is_initialized(slice_contents, dfg)?;

                assert!(!slice_typ.is_nested_slice(), "ICE: Nested slice used in ACIR generation");

                let slice = self.convert_value(slice_contents, dfg);
                let remove_index = self.convert_value(arguments[2], dfg).into_var()?;

                let one = self.acir_context.add_constant(FieldElement::one());
                let new_slice_length = self.acir_context.sub_var(slice_length, one)?;

                let slice_size = arrays::flattened_value_size(&slice);

                let new_slice = self.read_array(slice)?;

                // Compiler sanity check
                assert_eq!(
                    new_slice.len(),
                    slice_size,
                    "ICE: The read flattened slice should match the computed size"
                );

                // Fetch the flattened index from the user provided index argument.
                let element_size = slice_typ.element_size();
                let element_size_var = self.acir_context.add_constant(element_size);
                let flat_remove_index =
                    self.acir_context.mul_var(remove_index, element_size_var)?;
                let flat_user_index =
                    self.get_flattened_index(&slice_typ, slice_contents, flat_remove_index, dfg)?;

                // Fetch the values we are remove from the slice.
                // As we fetch the values we can determine the size of the removed values
                // which we will later use for writing the correct resulting slice.
                let mut popped_elements = Vec::new();
                let mut popped_elements_size = 0;
                // Set a temp index just for fetching from the original slice as `array_get_value` mutates
                // the index internally.
                let mut temp_index = flat_user_index;
                for res in &result_ids[2..(2 + element_size)] {
                    let element =
                        self.array_get_value(&dfg.type_of_value(*res), block_id, &mut temp_index)?;
                    let elem_size = arrays::flattened_value_size(&element);
                    popped_elements_size += elem_size;
                    popped_elements.push(element);
                }

                // Go through the entire slice argument and determine what value should be written to the new slice.
                // 1. If the current index is greater than the removal index we must write the next value
                //    from the original slice to the current index
                // 2. At the end of the slice reading from the next value of the original slice
                //    can lead to a potential out of bounds error. In this case we just fetch from the original slice
                //    at the current index. As we are decreasing the slice in length, this is a safe operation.
                let result_block_id = self.block_id(result_ids[1]);
                self.initialize_array(
                    result_block_id,
                    slice_size,
                    Some(AcirValue::Array(new_slice.clone())),
                )?;
                for i in 0..slice_size {
                    let current_index = self.acir_context.add_constant(i);

                    let value_current_index = &new_slice[i].borrow_var()?;

                    if slice_size > (i + popped_elements_size) {
                        let shifted_index =
                            self.acir_context.add_constant(i + popped_elements_size);

                        let value_shifted_index =
                            self.acir_context.read_from_memory(block_id, &shifted_index)?;

                        let use_shifted_value = self.acir_context.more_than_eq_var(
                            current_index,
                            flat_user_index,
                            64,
                        )?;

                        let shifted_value_pred =
                            self.acir_context.mul_var(value_shifted_index, use_shifted_value)?;
                        let not_pred = self.acir_context.sub_var(one, use_shifted_value)?;
                        let current_value_pred =
                            self.acir_context.mul_var(not_pred, *value_current_index)?;

                        let new_value =
                            self.acir_context.add_var(shifted_value_pred, current_value_pred)?;

                        self.acir_context.write_to_memory(
                            result_block_id,
                            &current_index,
                            &new_value,
                        )?;
                    };
                }

                let new_slice_val = AcirValue::Array(new_slice);
                let element_type_sizes =
                    if arrays::array_has_constant_element_size(&slice_typ).is_none() {
                        Some(self.init_element_type_sizes_array(
                            &slice_typ,
                            slice_contents,
                            Some(&new_slice_val),
                            dfg,
                        )?)
                    } else {
                        None
                    };

                let value_types = new_slice_val.flat_numeric_types();
                assert_eq!(
                    value_types.len(),
                    slice_size,
                    "ICE: Value types array must match new slice size"
                );

                let result = AcirValue::DynamicArray(AcirDynamicArray {
                    block_id: result_block_id,
                    len: slice_size,
                    value_types,
                    element_type_sizes,
                });

                let mut result = vec![AcirValue::Var(new_slice_length, AcirType::field()), result];
                result.append(&mut popped_elements);

                Ok(result)
            }

            Intrinsic::AsWitness => {
                let arg = arguments[0];
                let input = self.convert_value(arg, dfg).into_var()?;
                Ok(self
                    .acir_context
                    .get_or_create_witness_var(input)
                    .map(|val| self.convert_vars_to_values(vec![val], dfg, result_ids))?)
            }
            Intrinsic::ArrayAsStrUnchecked => Ok(vec![self.convert_value(arguments[0], dfg)]),
            Intrinsic::AssertConstant => {
                unreachable!("Expected assert_constant to be removed by this point")
            }
            Intrinsic::StaticAssert => {
                unreachable!("Expected static_assert to be removed by this point")
            }
            Intrinsic::StrAsBytes => unreachable!("Expected as_bytes to be removed by this point"),
            Intrinsic::IsUnconstrained => {
                unreachable!("Expected is_unconstrained to be removed by this point")
            }
            Intrinsic::DerivePedersenGenerators => Err(RuntimeError::AssertConstantFailed {
                call_stack: self.acir_context.get_call_stack(),
            }),
            Intrinsic::FieldLessThan => {
                unreachable!("FieldLessThan can only be called in unconstrained")
            }
            Intrinsic::ArrayRefCount | Intrinsic::SliceRefCount => {
                let zero = self.acir_context.add_constant(FieldElement::zero());
                Ok(vec![AcirValue::Var(
                    zero,
                    AcirType::NumericType(NumericType::Unsigned { bit_size: 32 }),
                )])
            }
        }
    }

    fn handle_ssa_call_outputs(
        &mut self,
        result_ids: &[ValueId],
        output_values: Vec<AcirValue>,
        dfg: &DataFlowGraph,
    ) -> Result<(), RuntimeError> {
        for (result_id, output) in result_ids.iter().zip(output_values) {
            if let AcirValue::Array(_) = &output {
                let array_id = *result_id;
                let block_id = self.block_id(array_id);
                let array_typ = dfg.type_of_value(array_id);
                let len = if matches!(array_typ, Type::Array(_, _)) {
                    array_typ.flattened_size() as usize
                } else {
                    arrays::flattened_value_size(&output)
                };
                self.initialize_array(block_id, len, Some(output.clone()))?;
            }
            // Do nothing for AcirValue::DynamicArray and AcirValue::Var
            // A dynamic array returned from a function call should already be initialized
            // and a single variable does not require any extra initialization.
            self.ssa_values.insert(*result_id, output);
        }
        Ok(())
    }

    /// Convert a `Vec<[AcirVar]>` into a `Vec<[AcirValue]>` using the given result ids.
    /// If the type of a result id is an array, several acir vars are collected into
    /// a single [AcirValue::Array] of the same length.
    /// If the type of a result id is a slice, the slice length must precede it and we can
    /// convert to an [AcirValue::Array] when the length is known (constant).
    fn convert_vars_to_values(
        &self,
        vars: Vec<AcirVar>,
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Vec<AcirValue> {
        let mut vars = vars.into_iter();
        let mut values: Vec<AcirValue> = Vec::new();
        for result in result_ids {
            let result_type = dfg.type_of_value(*result);
            if let Type::Slice(elements_type) = result_type {
                let error = "ICE - cannot get slice length when converting slice to AcirValue";
                let len = values.last().expect(error).borrow_var().expect(error);
                let len = self.acir_context.constant(len).to_u128();
                let mut element_values = im::Vector::new();
                for _ in 0..len {
                    for element_type in elements_type.iter() {
                        let element = Self::convert_var_type_to_values(element_type, &mut vars);
                        element_values.push_back(element);
                    }
                }
                values.push(AcirValue::Array(element_values));
            } else {
                values.push(Self::convert_var_type_to_values(&result_type, &mut vars));
            }
        }
        values
    }

    /// Recursive helper for [Self::convert_vars_to_values].
    /// If the given result_type is an array of length N, this will create an [AcirValue::Array] with
    /// the first N elements of the given iterator. Otherwise, the result is a single
    /// [AcirValue::Var] wrapping the first element of the iterator.
    fn convert_var_type_to_values(
        result_type: &Type,
        vars: &mut impl Iterator<Item = AcirVar>,
    ) -> AcirValue {
        match result_type {
            Type::Array(elements, size) => {
                let mut element_values = im::Vector::new();
                for _ in 0..*size {
                    for element_type in elements.iter() {
                        let element = Self::convert_var_type_to_values(element_type, vars);
                        element_values.push_back(element);
                    }
                }
                AcirValue::Array(element_values)
            }
            typ => {
                let var = vars.next().unwrap();
                AcirValue::Var(var, typ.into())
            }
        }
    }
}
