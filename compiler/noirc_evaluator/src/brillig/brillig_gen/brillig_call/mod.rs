pub(super) mod brillig_black_box;
pub(super) mod brillig_slice_ops;
pub(super) mod code_gen_call;

use acvm::brillig_vm::offsets;
use iter_extended::vecmap;

use crate::brillig::BrilligBlock;
use crate::brillig::brillig_ir::{BrilligBinaryOp, registers::RegisterAllocator};
use crate::ssa::ir::function::FunctionId;
use crate::ssa::ir::instruction::{InstructionId, Intrinsic};
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    types::Type,
    value::{Value, ValueId},
};

use crate::brillig::brillig_ir::brillig_variable::{
    BrilligArray, BrilligVariable, SingleAddrVariable,
};

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Allocates a variable to hold the result of an external function call (e.g., foreign or black box).
    /// For more information about foreign function calls in Brillig take a look at the [foreign call opcode][acvm::acir::brillig::Opcode::ForeignCall].
    ///
    /// This is typically used during Brillig codegen for calls to [Value::ForeignFunction], where
    /// external host functions return values back into the program.
    ///
    /// Numeric types and fixed-sized array results are directly allocated.
    ///
    /// As vectors are determined at runtime they are allocated differently:
    /// - Allocates memory for a [BrilligVariable::BrilligVector], which holds a pointer and dynamic size.
    /// - Initializes the pointer using the free memory pointer.
    /// - The actual size will be updated after the foreign function call returns.
    ///
    /// # Returns
    /// A [BrilligVariable] representing the allocated memory structure to store the foreign call's result.
    ///
    /// # Panics
    /// If there are more than one vector types that it needs to allocate, because they would both be
    /// assigned the free memory pointer.
    fn allocate_external_call_results(
        &mut self,
        results: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Vec<BrilligVariable> {
        let mut variables = Vec::new();
        let mut vector_allocated = false;

        for result in results {
            let result = *result;
            let typ = dfg[result].get_type();
            let variable = match typ.as_ref() {
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

                    // Set the pointer to the current free memory pointer.
                    // The free memory pointer will then be updated by the caller of this method,
                    // once the external call is resolved and the vector size is known.
                    assert!(
                        !vector_allocated,
                        "a vector has already been allocated at the free memory pointer"
                    );
                    vector_allocated = true;
                    self.brillig_context.load_free_memory_pointer_instruction(vector.pointer);

                    variable
                }
                _ => {
                    unreachable!("ICE: unsupported return type for black box call {typ:?}")
                }
            };
            variables.push(variable);
        }
        variables
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

        // Reserve free memory and set the initial ref-count.
        self.brillig_context.codegen_initialize_array(array);

        // Go through each slot in the array: if it's a simple type then we don't need to do anything,
        // but if it's a nested one we have to recursively allocate memory for it, and store the variable in the array.
        // We add one since array.pointer points to [RC, ...items]
        let mut index = offsets::ARRAY_ITEMS;
        for _ in 0..*size {
            for element_type in types.iter() {
                match element_type {
                    Type::Array(_, nested_size) => {
                        let inner_array =
                            self.brillig_context.allocate_brillig_array(*nested_size as usize);

                        self.allocate_foreign_call_result_array(element_type, *inner_array);

                        let idx =
                            self.brillig_context.make_usize_constant_instruction(index.into());

                        self.brillig_context.codegen_store_with_offset(
                            array.pointer,
                            *idx,
                            inner_array.pointer,
                        );
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

    /// Internal method to codegen an [crate::ssa::ir::instruction::Instruction::Call] to a [Value::Function]
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

                self.slice_insert_operation(target_vector, source_vector, *converted_index, &items);
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
                    *converted_index,
                    &removed_items,
                );
            }
            _ => unreachable!("ICE: Slice operation not supported"),
        }
    }
}
