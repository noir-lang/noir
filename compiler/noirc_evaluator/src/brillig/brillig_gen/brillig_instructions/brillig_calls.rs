use acvm::acir::BlackBoxFunc;
use acvm::acir::brillig::{MemoryAddress, ValueOrArray};
use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;

use crate::brillig::BrilligBlock;
use crate::brillig::brillig_ir::BrilligBinaryOp;
use crate::brillig::brillig_ir::registers::RegisterAllocator;
use crate::ssa::ir::function::FunctionId;
use crate::ssa::ir::types::NumericType;
use crate::ssa::ir::types::Type;
use crate::ssa::ir::value::Value;
use crate::ssa::ir::{dfg::DataFlowGraph, value::ValueId};

use super::super::brillig_black_box::convert_black_box_call;
use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, SingleAddrVariable, type_to_heap_value_type,
};
use crate::ssa::ir::instruction::{Endian, Hint, InstructionId, Intrinsic};

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Allocates a variable to hold the result of an external function call (e.g., foreign or black box).
    /// For more information about foreign function calls in Brillig take a look at the [foreign call opcode][acvm::acir::brillig::Opcode::ForeignCall].
    ///
    /// This is typically used during Brillig codegen for calls to [Value::ForeignFunction], where
    /// external host functions return values back into the program.
    ///
    /// Numeric types and fixed-sized array results are directly allocated.
    /// As vector's are determined at runtime they are allocated differently.
    /// - Allocates memory for a [BrilligVariable::BrilligVector], which holds a pointer and dynamic size.
    /// - Initializes the pointer using the free memory pointer.
    /// - The actual size will be updated after the foreign function call returns.
    ///
    /// # Returns
    /// A [BrilligVariable] representing the allocated memory structure to store the foreign call's result.
    fn allocate_external_call_result(
        &mut self,
        result: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let typ = dfg[result].get_type();
        match typ.as_ref() {
            Type::Numeric(_) => self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                result,
                dfg,
            ),

            Type::Array(..) => {
                let variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result,
                    dfg,
                );
                let array = variable.extract_array();
                self.allocate_foreign_call_result_array(typ.as_ref(), array);

                variable
            }
            Type::Slice(_) => {
                let variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    result,
                    dfg,
                );
                let vector = variable.extract_vector();

                // Set the pointer to the current stack frame
                // The stack pointer will then be updated by the caller of this method
                // once the external call is resolved and the array size is known
                self.brillig_context.load_free_memory_pointer_instruction(vector.pointer);

                variable
            }
            _ => {
                unreachable!("ICE: unsupported return type for black box call {typ:?}")
            }
        }
    }

    /// Recursively allocates memory for a nested array returned from a foreign function call.
    ///
    /// # Panics
    /// - If the provided `typ` is not an array.
    /// - If any slice types are encountered within the nested structure, since slices
    ///   require runtime size information and cannot be allocated statically here.
    fn allocate_foreign_call_result_array(&mut self, typ: &Type, array: BrilligArray) {
        let Type::Array(types, size) = typ else {
            unreachable!("ICE: allocate_foreign_call_array() expects an array, got {typ:?}")
        };

        self.brillig_context.codegen_initialize_array(array);

        let mut index = 0_usize;
        for _ in 0..*size {
            for element_type in types.iter() {
                match element_type {
                    Type::Array(_, nested_size) => {
                        let inner_array = BrilligArray {
                            pointer: self.brillig_context.allocate_register(),
                            size: *nested_size as usize,
                        };
                        self.allocate_foreign_call_result_array(element_type, inner_array);

                        // We add one since array.pointer points to [RC, ...items]
                        let idx = self
                            .brillig_context
                            .make_usize_constant_instruction((index + 1).into());
                        self.brillig_context.codegen_store_with_offset(
                            array.pointer,
                            idx,
                            inner_array.pointer,
                        );

                        self.brillig_context.deallocate_single_addr(idx);
                        self.brillig_context.deallocate_register(inner_array.pointer);
                    }
                    Type::Slice(_) => unreachable!(
                        "ICE: unsupported slice type in allocate_nested_array(), expects an array or a numeric type"
                    ),
                    _ => (),
                }
                index += 1;
            }
        }
    }

    /// Internal method to codegen an [Instruction::Call] to a [Value::Function]
    fn convert_ssa_function_call(
        &mut self,
        func_id: FunctionId,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) {
        let argument_variables =
            vecmap(arguments, |argument_id| self.convert_ssa_value(*argument_id, dfg));
        let return_variables = vecmap(result_ids, |result_id| {
            self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                *result_id,
                dfg,
            )
        });
        self.brillig_context.codegen_call(func_id, &argument_variables, &return_variables);
    }

    /// Gets the "user-facing" length of an array.
    /// An array of structs with two fields would be stored as an 2 * array.len() array/vector.
    /// So we divide the length by the number of subitems in an item to get the user-facing length.
    fn convert_ssa_array_len(
        &mut self,
        array_id: ValueId,
        result_register: MemoryAddress,
        dfg: &DataFlowGraph,
    ) {
        let array_variable = self.convert_ssa_value(array_id, dfg);
        let element_size = dfg.type_of_value(array_id).element_size();

        match array_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, .. }) => {
                self.brillig_context
                    .usize_const_instruction(result_register, (size / element_size).into());
            }
            BrilligVariable::BrilligVector(vector) => {
                let size = self.brillig_context.codegen_make_vector_length(vector);

                self.brillig_context.codegen_usize_op(
                    size.address,
                    result_register,
                    BrilligBinaryOp::UnsignedDiv,
                    element_size,
                );

                self.brillig_context.deallocate_single_addr(size);
            }
            _ => {
                unreachable!("ICE: Cannot get length of {array_variable:?}")
            }
        }
    }

    /// Increase or decrease the slice length by 1.
    ///
    /// Slices have a tuple structure (slice length, slice contents) to enable logic
    /// that uses dynamic slice lengths (such as with merging slices in the flattening pass).
    /// This method codegens an update to the slice length.
    ///
    /// The binary operation performed on the slice length is always an addition or subtraction of `1`.
    /// This is because the slice length holds the user length (length as displayed by a `.len()` call),
    /// and not a flattened length used internally to represent arrays of tuples.
    /// The length inside of `RegisterOrMemory::HeapVector` represents the entire flattened number
    /// of fields in the vector.
    ///
    /// Note that when we subtract a value, we expect that there is a constraint in SSA
    /// to check that the length isn't already 0. We could add a constraint opcode here,
    /// but if it's in SSA, there is a chance it can be optimized out.
    fn update_slice_length(
        &mut self,
        target_len: SingleAddrVariable,
        source_len: SingleAddrVariable,
        binary_op: BrilligBinaryOp,
    ) {
        self.brillig_context.codegen_usize_op(source_len.address, target_len.address, binary_op, 1);
    }

    /// Copy the input arguments to the results.
    fn convert_ssa_identity_call(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) {
        let argument_variables =
            vecmap(arguments, |argument_id| self.convert_ssa_value(*argument_id, dfg));

        let return_variables = vecmap(result_ids, |result_id| {
            self.variables.define_variable(
                self.function_context,
                self.brillig_context,
                *result_id,
                dfg,
            )
        });

        for (src, dst) in argument_variables.into_iter().zip(return_variables) {
            self.brillig_context.mov_instruction(dst.extract_register(), src.extract_register());
        }
    }

    /// Convert the SSA slice operations to brillig slice operations
    fn convert_ssa_slice_intrinsic_call(
        &mut self,
        dfg: &DataFlowGraph,
        intrinsic: &Value,
        instruction_id: InstructionId,
        arguments: &[ValueId],
    ) {
        // Slice operations always look like `... = call slice_<op> source_len, source_vector, ...`
        let source_len = self.convert_ssa_value(arguments[0], dfg);
        let source_len = source_len.extract_single_addr();

        let slice_id = arguments[1];
        let element_size = dfg.type_of_value(slice_id).element_size();
        let source_vector = self.convert_ssa_value(slice_id, dfg).extract_vector();

        let results = dfg.instruction_results(instruction_id);
        match intrinsic {
            Value::Intrinsic(Intrinsic::SlicePushBack) => {
                // target_len, target_slice = slice_push_back source_len, source_slice
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                let target_vector = target_variable.extract_vector();
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                // target_len = source_len + 1
                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Add);

                self.slice_push_back_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &item_values,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePushFront) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );
                let target_vector = target_variable.extract_vector();
                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Add);

                self.slice_push_front_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &item_values,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePopBack) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[1],
                    dfg,
                );

                let target_vector = target_variable.extract_vector();

                let pop_variables = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.slice_pop_back_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &pop_variables,
                );
            }
            Value::Intrinsic(Intrinsic::SlicePopFront) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[element_size],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let pop_variables = vecmap(&results[0..element_size], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[element_size + 1],
                    dfg,
                );
                let target_vector = target_variable.extract_vector();

                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.slice_pop_front_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &pop_variables,
                );
            }
            Value::Intrinsic(Intrinsic::SliceInsert) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];
                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    target_id,
                    dfg,
                );

                let target_vector = target_variable.extract_vector();

                // Remove if indexing in insert is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_single_addr_value(arguments[2], dfg);

                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(element_size.into());

                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

                let items = vecmap(&arguments[3..element_size + 3], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Add);

                self.slice_insert_operation(target_vector, source_vector, converted_index, &items);
                self.brillig_context.deallocate_single_addr(converted_index);
            }
            Value::Intrinsic(Intrinsic::SliceRemove) => {
                let target_len = match self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    results[0],
                    dfg,
                ) {
                    BrilligVariable::SingleAddr(register_index) => register_index,
                    _ => unreachable!("ICE: first value of a slice must be a register index"),
                };

                let target_id = results[1];

                let target_variable = self.variables.define_variable(
                    self.function_context,
                    self.brillig_context,
                    target_id,
                    dfg,
                );
                let target_vector = target_variable.extract_vector();

                // Remove if indexing in remove is changed to flattened indexing
                // https://github.com/noir-lang/noir/issues/1889#issuecomment-1668048587
                let user_index = self.convert_ssa_single_addr_value(arguments[2], dfg);

                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(element_size.into());
                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

                let removed_items = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_slice_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.slice_remove_operation(
                    target_vector,
                    source_vector,
                    converted_index,
                    &removed_items,
                );

                self.brillig_context.deallocate_single_addr(converted_index);
            }
            _ => unreachable!("ICE: Slice operation not supported"),
        }
    }

    pub(crate) fn call_gen(
        &mut self,
        instruction_id: InstructionId,
        func: ValueId,
        arguments: &Vec<ValueId>,
        dfg: &DataFlowGraph,
    ) {
        match &dfg[func] {
            Value::ForeignFunction(func_name) => {
                let result_ids = dfg.instruction_results(instruction_id);

                let input_values = vecmap(arguments, |value_id| {
                    let variable = self.convert_ssa_value(*value_id, dfg);
                    self.brillig_context.variable_to_value_or_array(variable)
                });
                let input_value_types = vecmap(arguments, |value_id| {
                    let value_type = dfg.type_of_value(*value_id);
                    type_to_heap_value_type(&value_type)
                });
                let output_variables = vecmap(result_ids, |value_id| {
                    self.allocate_external_call_result(*value_id, dfg)
                });
                let output_values = vecmap(&output_variables, |variable| {
                    self.brillig_context.variable_to_value_or_array(*variable)
                });
                let output_value_types = vecmap(result_ids, |value_id| {
                    let value_type = dfg.type_of_value(*value_id);
                    type_to_heap_value_type(&value_type)
                });
                self.brillig_context.foreign_call_instruction(
                    func_name.to_owned(),
                    &input_values,
                    &input_value_types,
                    &output_values,
                    &output_value_types,
                );

                // Deallocate the temporary heap arrays and vectors of the inputs
                for input_value in input_values {
                    match input_value {
                        ValueOrArray::HeapArray(array) => {
                            self.brillig_context.deallocate_heap_array(array);
                        }
                        ValueOrArray::HeapVector(vector) => {
                            self.brillig_context.deallocate_heap_vector(vector);
                        }
                        _ => {}
                    }
                }

                // Deallocate the temporary heap arrays and vectors of the outputs
                for (i, (output_register, output_variable)) in
                    output_values.iter().zip(output_variables).enumerate()
                {
                    match output_register {
                        // Returned vectors need to emit some bytecode to format the result as a BrilligVector
                        ValueOrArray::HeapVector(heap_vector) => {
                            self.brillig_context.initialize_externally_returned_vector(
                                output_variable.extract_vector(),
                                *heap_vector,
                            );
                            // Update the dynamic slice length maintained in SSA
                            if let ValueOrArray::MemoryAddress(len_index) = output_values[i - 1] {
                                let element_size = dfg[result_ids[i]].get_type().element_size();
                                self.brillig_context.mov_instruction(len_index, heap_vector.size);
                                self.brillig_context.codegen_usize_op_in_place(
                                    len_index,
                                    BrilligBinaryOp::UnsignedDiv,
                                    element_size,
                                );
                            } else {
                                unreachable!(
                                    "ICE: a vector must be preceded by a register containing its length"
                                );
                            }
                            self.brillig_context.deallocate_heap_vector(*heap_vector);
                        }
                        ValueOrArray::HeapArray(array) => {
                            self.brillig_context.deallocate_heap_array(*array);
                        }
                        ValueOrArray::MemoryAddress(_) => {}
                    }
                }
            }
            Value::Function(func_id) => {
                let result_ids = dfg.instruction_results(instruction_id);
                self.convert_ssa_function_call(*func_id, arguments, dfg, result_ids);
            }
            Value::Intrinsic(intrinsic) => {
                // This match could be combined with the above but without it rust analyzer
                // can't automatically insert any missing cases
                match intrinsic {
                    Intrinsic::ArrayLen => {
                        let [result_value] = dfg.instruction_result(instruction_id);
                        let result_variable = self.variables.define_single_addr_variable(
                            self.function_context,
                            self.brillig_context,
                            result_value,
                            dfg,
                        );
                        let param_id = arguments[0];
                        // Slices are represented as a tuple in the form: (length, slice contents).
                        // Thus, we can expect the first argument to a field in the case of a slice
                        // or an array in the case of an array.
                        if let Type::Numeric(_) = dfg.type_of_value(param_id) {
                            let len_variable = self.convert_ssa_value(arguments[0], dfg);
                            let length = len_variable.extract_single_addr();
                            self.brillig_context
                                .mov_instruction(result_variable.address, length.address);
                        } else {
                            self.convert_ssa_array_len(arguments[0], result_variable.address, dfg);
                        }
                    }
                    Intrinsic::AsSlice => {
                        let source_variable = self.convert_ssa_value(arguments[0], dfg);
                        let result_ids = dfg.instruction_results(instruction_id);
                        let destination_len_variable = self.variables.define_single_addr_variable(
                            self.function_context,
                            self.brillig_context,
                            result_ids[0],
                            dfg,
                        );
                        let destination_variable = self.variables.define_variable(
                            self.function_context,
                            self.brillig_context,
                            result_ids[1],
                            dfg,
                        );
                        let destination_vector = destination_variable.extract_vector();
                        let source_array = source_variable.extract_array();
                        let element_size = dfg.type_of_value(arguments[0]).element_size();

                        let source_size_register = self
                            .brillig_context
                            .make_usize_constant_instruction(source_array.size.into());

                        // we need to explicitly set the destination_len_variable
                        self.brillig_context.codegen_usize_op(
                            source_size_register.address,
                            destination_len_variable.address,
                            BrilligBinaryOp::UnsignedDiv,
                            element_size,
                        );

                        self.brillig_context.codegen_initialize_vector(
                            destination_vector,
                            source_size_register,
                            None,
                        );

                        // Items
                        let vector_items_pointer = self
                            .brillig_context
                            .codegen_make_vector_items_pointer(destination_vector);
                        let array_items_pointer =
                            self.brillig_context.codegen_make_array_items_pointer(source_array);

                        self.brillig_context.codegen_mem_copy(
                            array_items_pointer,
                            vector_items_pointer,
                            source_size_register,
                        );

                        self.brillig_context.deallocate_single_addr(source_size_register);
                        self.brillig_context.deallocate_register(vector_items_pointer);
                        self.brillig_context.deallocate_register(array_items_pointer);
                    }
                    Intrinsic::SlicePushBack
                    | Intrinsic::SlicePopBack
                    | Intrinsic::SlicePushFront
                    | Intrinsic::SlicePopFront
                    | Intrinsic::SliceInsert
                    | Intrinsic::SliceRemove => {
                        self.convert_ssa_slice_intrinsic_call(
                            dfg,
                            &dfg[func],
                            instruction_id,
                            arguments,
                        );
                    }
                    Intrinsic::ToBits(endianness) => {
                        let [result] = dfg.instruction_result(instruction_id);

                        let source = self.convert_ssa_single_addr_value(arguments[0], dfg);

                        let target_array = self
                            .variables
                            .define_variable(
                                self.function_context,
                                self.brillig_context,
                                result,
                                dfg,
                            )
                            .extract_array();

                        let two =
                            self.brillig_context.make_usize_constant_instruction(2_usize.into());

                        self.brillig_context.codegen_to_radix(
                            source,
                            target_array,
                            two,
                            matches!(endianness, Endian::Little),
                            true,
                        );

                        self.brillig_context.deallocate_single_addr(two);
                    }

                    Intrinsic::ToRadix(endianness) => {
                        let [result] = dfg.instruction_result(instruction_id);

                        let source = self.convert_ssa_single_addr_value(arguments[0], dfg);
                        let radix = self.convert_ssa_single_addr_value(arguments[1], dfg);

                        let target_array = self
                            .variables
                            .define_variable(
                                self.function_context,
                                self.brillig_context,
                                result,
                                dfg,
                            )
                            .extract_array();

                        self.brillig_context.codegen_to_radix(
                            source,
                            target_array,
                            radix,
                            matches!(endianness, Endian::Little),
                            false,
                        );
                    }
                    Intrinsic::Hint(Hint::BlackBox) => {
                        let result_ids = dfg.instruction_results(instruction_id);
                        self.convert_ssa_identity_call(arguments, dfg, result_ids);
                    }
                    Intrinsic::BlackBox(bb_func) => {
                        // Slices are represented as a tuple of (length, slice contents).
                        // We must check the inputs to determine if there are slices
                        // and make sure that we pass the correct inputs to the black box function call.
                        // The loop below only keeps the slice contents, so that
                        // setting up a black box function with slice inputs matches the expected
                        // number of arguments specified in the function signature.
                        let mut arguments_no_slice_len = Vec::new();
                        for (i, arg) in arguments.iter().enumerate() {
                            if matches!(dfg.type_of_value(*arg), Type::Numeric(_)) {
                                if i < arguments.len() - 1 {
                                    if !matches!(
                                        dfg.type_of_value(arguments[i + 1]),
                                        Type::Slice(_)
                                    ) {
                                        arguments_no_slice_len.push(*arg);
                                    }
                                } else {
                                    arguments_no_slice_len.push(*arg);
                                }
                            } else {
                                arguments_no_slice_len.push(*arg);
                            }
                        }

                        if matches!(
                            bb_func,
                            BlackBoxFunc::EcdsaSecp256k1
                                | BlackBoxFunc::EcdsaSecp256r1
                                | BlackBoxFunc::MultiScalarMul
                                | BlackBoxFunc::EmbeddedCurveAdd
                        ) {
                            // Some black box functions have a predicate argument in SSA which we don't want to
                            // use in the brillig VM. This is as we do not need to flatten the CFG in brillig
                            // so we expect the predicate to always be true.
                            let predicate = &arguments_no_slice_len.pop().expect(
                                "ICE: ECDSA black box function must have a predicate argument",
                            );
                            assert_eq!(
                                dfg.get_numeric_constant_with_type(*predicate),
                                Some((FieldElement::one(), NumericType::bool())),
                                "ICE: ECDSA black box function must have a predicate argument with value 1"
                            );
                        }

                        let function_arguments = vecmap(&arguments_no_slice_len, |arg| {
                            self.convert_ssa_value(*arg, dfg)
                        });
                        let function_results = dfg.instruction_results(instruction_id);
                        let function_results = vecmap(function_results, |result| {
                            self.allocate_external_call_result(*result, dfg)
                        });
                        convert_black_box_call(
                            self.brillig_context,
                            bb_func,
                            &function_arguments,
                            &function_results,
                        );
                    }
                    // `Intrinsic::AsWitness` is used to provide hints to acir-gen on optimal expression splitting.
                    // It is then useless in the brillig runtime and so we can ignore it
                    Intrinsic::AsWitness => (),
                    Intrinsic::FieldLessThan => {
                        let lhs = self.convert_ssa_single_addr_value(arguments[0], dfg);
                        assert!(lhs.bit_size == FieldElement::max_num_bits());
                        let rhs = self.convert_ssa_single_addr_value(arguments[1], dfg);
                        assert!(rhs.bit_size == FieldElement::max_num_bits());

                        let [result] = dfg.instruction_result(instruction_id);
                        let destination = self
                            .variables
                            .define_variable(
                                self.function_context,
                                self.brillig_context,
                                result,
                                dfg,
                            )
                            .extract_single_addr();
                        assert!(destination.bit_size == 1);

                        self.brillig_context.binary_instruction(
                            lhs,
                            rhs,
                            destination,
                            BrilligBinaryOp::LessThan,
                        );
                    }
                    Intrinsic::ArrayRefCount => {
                        let array = self.convert_ssa_value(arguments[0], dfg);
                        let [result] = dfg.instruction_result(instruction_id);

                        let destination = self.variables.define_variable(
                            self.function_context,
                            self.brillig_context,
                            result,
                            dfg,
                        );
                        let destination = destination.extract_register();
                        let array = array.extract_register();
                        self.brillig_context.load_instruction(destination, array);
                    }
                    Intrinsic::SliceRefCount => {
                        let array = self.convert_ssa_value(arguments[1], dfg);
                        let [result] = dfg.instruction_result(instruction_id);

                        let destination = self.variables.define_variable(
                            self.function_context,
                            self.brillig_context,
                            result,
                            dfg,
                        );
                        let destination = destination.extract_register();
                        let array = array.extract_register();
                        self.brillig_context.load_instruction(destination, array);
                    }
                    Intrinsic::IsUnconstrained
                    | Intrinsic::DerivePedersenGenerators
                    | Intrinsic::ApplyRangeConstraint
                    | Intrinsic::StrAsBytes
                    | Intrinsic::AssertConstant
                    | Intrinsic::StaticAssert
                    | Intrinsic::ArrayAsStrUnchecked => {
                        unreachable!("unsupported function call type {:?}", dfg[func])
                    }
                }
            }
            Value::Instruction { .. }
            | Value::Param { .. }
            | Value::NumericConstant { .. }
            | Value::Global(_) => {
                unreachable!("unsupported function call type {:?}", dfg[func])
            }
        }
    }
}
