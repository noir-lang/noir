pub(super) mod brillig_black_box;
pub(super) mod brillig_vector_ops;
pub(super) mod code_gen_call;

use acvm::acir::brillig::lengths::{ElementsLength, SemanticLength, SemiFlattenedLength};
use acvm::brillig_vm::offsets;
use iter_extended::vecmap;

use crate::brillig::BrilligBlock;
use crate::brillig::brillig_ir::assert_u32;
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
    /// If there is a vector among the output variables _and_ it's followed by another vector:
    /// when we allocate memory for a vector, we don't know its length, so it just points at the current
    /// free memory pointer without increasing it; a second vector gets allocated at the same memory slot.
    fn allocate_external_call_results(
        &mut self,
        results: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Vec<BrilligVariable> {
        let mut variables = Vec::new();
        let mut vector_allocated = None;

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
                Type::Vector(_) => {
                    let variable = self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        result,
                        dfg,
                    );

                    // Set its pointer to the free memory address, and expect the VM to write the data where the vector points to.
                    // We can only support one vector output this way, otherwise the next vector would overwrite it.
                    // The vector also has to be the last output of the function, there cannot be any arrays following it.
                    assert!(
                        vector_allocated.is_none(),
                        "a previous vector has already been allocated at the free memory pointer"
                    );
                    // Remember the position of single vector we allocated; we will initialize it to the free memory pointer
                    // after we have dealt with any other arrays in the output, otherwise they could overwrite it.
                    vector_allocated = Some(variables.len());

                    variable
                }
                _ => {
                    unreachable!("ICE: unsupported return type for black box call {typ:?}")
                }
            };
            variables.push(variable);
        }

        if let Some(idx) = vector_allocated {
            let variable = &variables[idx];
            let vector = variable.extract_vector();
            self.brillig_context.load_free_memory_pointer_instruction(vector.pointer);
        }

        variables
    }

    /// Recursively allocates memory on the heap for a nested array returned from a foreign function call.
    ///
    /// # Panics
    /// - If the provided `typ` is not an array.
    /// - If any vector types are encountered within the nested structure, since vectors
    ///   require runtime size information and cannot be allocated statically here.
    fn allocate_foreign_call_result_array(&mut self, typ: &Type, array: BrilligArray) {
        let Type::Array(types, size) = typ else {
            unreachable!("ICE: allocate_foreign_call_array() expects an array, got {typ:?}")
        };

        // Reserve free memory on the heap and set the initial ref-count.
        self.brillig_context.codegen_initialize_array(array);

        // Go through each slot in the array: if it's a simple type then we don't need to do anything,
        // but if it's a nested one we have to recursively allocate memory for it, and store the variable in the array.
        // We add one since array.pointer points to [RC, ...items]
        let mut index = offsets::ARRAY_ITEMS;
        for _ in 0..*size {
            for element_type in types.iter() {
                match element_type {
                    Type::Array(items, nested_size) => {
                        // Allocate a pointer for an array on the stack.
                        let size: SemiFlattenedLength =
                            ElementsLength(assert_u32(items.len())) * SemanticLength(*nested_size);
                        let inner_array = self.brillig_context.allocate_brillig_array(size);

                        // Recursively allocate memory for the inner array on the heap.
                        // This sets the pointer on the stack to point at the heap.
                        self.allocate_foreign_call_result_array(element_type, *inner_array);

                        // Set the index in the outer array to be the total offset accounting for complex types.
                        let idx =
                            self.brillig_context.make_usize_constant_instruction(index.into());

                        // Copy the inner array pointer, which points at the heap, into the outer array cell.
                        // After this, it is okay for the `inner_array` to be deallocated from the stack,
                        // and for its address to be reused, ie. we don't need to `.detach()` it.
                        // What matters is that we stored the pointer to the heap.
                        self.brillig_context.codegen_store_with_offset(
                            array.pointer,
                            *idx,
                            inner_array.pointer,
                        );
                    }
                    Type::Vector(_) => unreachable!(
                        "ICE: unsupported vector type in allocate_nested_array(), expects an array or a numeric type"
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

    /// Increase or decrease the vector length by 1.
    ///
    /// Vectors have a tuple structure (vector length, vector contents) to enable logic
    /// that uses dynamic vector lengths (such as with merging vectors in the flattening pass).
    /// This method codegens an update to the vector length.
    ///
    /// The binary operation performed on the vector length is always an addition or subtraction of `1`.
    /// This is because the vector length holds the user length (length as displayed by a `.len()` call),
    /// and not a flattened length used internally to represent arrays of tuples.
    /// The length inside of `RegisterOrMemory::HeapVector` represents the entire flattened number
    /// of fields in the vector.
    ///
    /// Note that when we subtract a value, we expect that there is a constraint in SSA
    /// to check that the length isn't already 0. We could add a constraint opcode here,
    /// but if it's in SSA, there is a chance it can be optimized out.
    fn update_vector_length(
        &mut self,
        target_len: SingleAddrVariable,
        source_len: SingleAddrVariable,
        binary_op: BrilligBinaryOp,
    ) {
        debug_assert!(matches!(binary_op, BrilligBinaryOp::Add | BrilligBinaryOp::Sub));
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

    /// Convert the SSA vector operations to brillig vector operations
    fn convert_ssa_vector_intrinsic_call(
        &mut self,
        dfg: &DataFlowGraph,
        intrinsic: &Value,
        instruction_id: InstructionId,
        arguments: &[ValueId],
    ) {
        // Vector operations always look like `... = call vector_<op> source_len, source_vector, ...`
        let source_len = self.convert_ssa_value(arguments[0], dfg);
        let source_len = source_len.extract_single_addr();

        let vector_id = arguments[1];
        let element_size = dfg.type_of_value(vector_id).element_size();
        let source_vector = self.convert_ssa_value(vector_id, dfg).extract_vector();

        let results = dfg.instruction_results(instruction_id);

        let get_target_len = |this: &mut Self, idx: usize| {
            this.variables
                .define_variable(this.function_context, this.brillig_context, results[idx], dfg)
                .extract_single_addr()
        };

        let get_target_vector = |this: &mut Self, idx: usize| {
            this.variables
                .define_variable(this.function_context, this.brillig_context, results[idx], dfg)
                .extract_vector()
        };

        match intrinsic {
            Value::Intrinsic(Intrinsic::VectorPushBack) => {
                // target_len, target_vector = vector_push_back source_len, source_vector, ...elements
                let target_len = get_target_len(self, 0);
                let target_vector = get_target_vector(self, 1);

                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                // target_len = source_len + 1
                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Add);

                self.vector_push_back_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &item_values,
                );
            }
            Value::Intrinsic(Intrinsic::VectorPushFront) => {
                let target_len = get_target_len(self, 0);
                let target_vector = get_target_vector(self, 1);

                let item_values = vecmap(&arguments[2..element_size + 2], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Add);

                self.vector_push_front_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &item_values,
                );
            }
            Value::Intrinsic(Intrinsic::VectorPopBack) => {
                let target_len = get_target_len(self, 0);
                let target_vector = get_target_vector(self, 1);

                let pop_variables = vecmap(&results[2..element_size + 2], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.vector_pop_back_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &pop_variables,
                );
            }
            Value::Intrinsic(Intrinsic::VectorPopFront) => {
                // ...elements, target_len, target_vector = vector_pop_front len, vector
                let target_len = get_target_len(self, element_size);
                let target_vector = get_target_vector(self, element_size + 1);

                let pop_variables = vecmap(&results[0..element_size], |result| {
                    self.variables.define_variable(
                        self.function_context,
                        self.brillig_context,
                        *result,
                        dfg,
                    )
                });

                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.vector_pop_front_operation(
                    target_vector,
                    source_len,
                    source_vector,
                    &pop_variables,
                );
            }
            Value::Intrinsic(Intrinsic::VectorInsert) => {
                let target_len = get_target_len(self, 0);
                let target_vector = get_target_vector(self, 1);

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

                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Add);

                self.vector_insert_operation(
                    target_vector,
                    source_vector,
                    *converted_index,
                    &items,
                );
            }
            Value::Intrinsic(Intrinsic::VectorRemove) => {
                let target_len = get_target_len(self, 0);
                let target_vector = get_target_vector(self, 1);

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

                self.update_vector_length(target_len, source_len, BrilligBinaryOp::Sub);

                self.vector_remove_operation(
                    target_vector,
                    source_vector,
                    *converted_index,
                    &removed_items,
                );
            }
            _ => unreachable!("ICE: Vector operation not supported"),
        }
    }
}
