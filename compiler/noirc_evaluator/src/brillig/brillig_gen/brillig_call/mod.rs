pub(super) mod brillig_black_box;
pub(super) mod brillig_vector_ops;
pub(super) mod code_gen_call;

use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;
use itertools::Itertools;

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
    /// If there is a vector among the output variables _and_ it's followed by another vector:
    /// when we allocate memory for a vector, we don't know its length, so it just points at the current
    /// free memory pointer without increasing it; a second vector gets allocated at the same memory slot.
    ///
    /// If there is a vector among the output variables and it is not preceded by a variable
    /// for the semantic length of the vector.
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
                Type::Numeric(_) => self.define_variable(result, dfg),

                Type::Array(..) => {
                    let variable = self.define_variable(result, dfg);
                    let array = variable.extract_array();

                    self.allocate_foreign_call_result_array(array);

                    variable
                }
                Type::Vector(_) => {
                    let variable = self.define_variable(result, dfg);

                    // Set its pointer to the free memory address, and expect the VM to write the data where the vector points to.
                    // We can only support one vector output this way, otherwise the next vector would overwrite it.
                    // The vector also has to be the last output of the function, there cannot be any arrays following it.
                    assert!(
                        vector_allocated.is_none(),
                        "ICE: a previous vector has already been allocated at the free memory pointer"
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

            // Historically the returned semantic length has been ignored and
            // overwritten based on what we calculate from the returned data.
            // We stopped doing that, so that we can handle the edge case of zero-sized items;
            // now we reject results if the semantic length is set to an unexpected value.
            // The AVM, however, did not set the semantic length at all,
            // which risks leaving some previous memory content in the slot allocated
            // to the SSA variable on the stack. To be on the safe side, set the
            // semantic length to 0 before the call. This can be removed if we
            // are sure that the AVM does, in fact, set the value in all cases.
            let BrilligVariable::SingleAddr(semantic_length) = &variables[idx - 1] else {
                unreachable!("ICE: a vector must be preceded by a register containing its length");
            };
            self.brillig_context.const_instruction(*semantic_length, FieldElement::zero());
        }

        variables
    }

    /// Allocates memory on the heap for a flat array returned from a foreign function call.
    /// With fully-flat arrays, no recursive nesting is needed — just initialize the array.
    fn allocate_foreign_call_result_array(&mut self, array: BrilligArray) {
        // Reserve free memory on the heap and set the initial ref-count.
        self.brillig_context.codegen_initialize_array(array);
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

        let return_variables =
            vecmap(result_ids, |result_id| self.define_variable(*result_id, dfg));
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
        assert!(matches!(binary_op, BrilligBinaryOp::Add | BrilligBinaryOp::Sub));
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

        let return_variables =
            vecmap(result_ids, |result_id| self.define_variable(*result_id, dfg));

        for (src, dst) in argument_variables.into_iter().zip_eq(return_variables) {
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
        let element_size = dfg.type_of_value(vector_id).element_size().to_usize();
        let source_vector = self.convert_ssa_value(vector_id, dfg).extract_vector();

        let results = dfg.instruction_results(instruction_id);

        let get_target_len = |this: &mut Self, idx: usize| {
            this.define_variable(results[idx], dfg).extract_single_addr()
        };

        let get_target_vector =
            |this: &mut Self, idx: usize| this.define_variable(results[idx], dfg).extract_vector();

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
                    self.define_variable(*result, dfg)
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

                let pop_variables =
                    vecmap(&results[0..element_size], |result| self.define_variable(*result, dfg));

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

                let items = vecmap(&arguments[3..element_size + 3], |arg| {
                    self.convert_ssa_value(*arg, dfg)
                });

                let flat_element_size = Self::flat_variable_count(&items);
                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(flat_element_size.into());

                // Safety: This multiplication cannot overflow because:
                // 1. SSA generates bounds checks ensuring `user_index <= length`
                // 2. The vector allocation is protected by FMP's checked addition
                // 3. Therefore `flat_element_size * user_index <= flat_element_size * length <= allocation_size < 2^32`
                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

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

                let removed_items = vecmap(&results[2..element_size + 2], |result| {
                    self.define_variable(*result, dfg)
                });

                let flat_element_size = Self::flat_variable_count(&removed_items);
                let converted_index =
                    self.brillig_context.make_usize_constant_instruction(flat_element_size.into());

                // Safety: This multiplication cannot overflow because:
                // 1. SSA generates bounds checks ensuring `user_index < length`
                // 2. The vector allocation is protected by FMP's checked addition
                // 3. Therefore `flat_element_size * user_index < flat_element_size * length <= allocation_size < 2^32`
                self.brillig_context.memory_op_instruction(
                    converted_index.address,
                    user_index.address,
                    converted_index.address,
                    BrilligBinaryOp::Mul,
                );

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
