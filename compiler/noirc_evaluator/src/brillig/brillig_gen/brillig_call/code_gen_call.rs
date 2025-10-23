use acvm::acir::BlackBoxFunc;
use acvm::acir::brillig::ValueOrArray;
use acvm::{AcirField, FieldElement};
use iter_extended::vecmap;

use crate::brillig::BrilligBlock;
use crate::brillig::brillig_ir::{BrilligBinaryOp, registers::RegisterAllocator};
use crate::ssa::ir::instruction::{Endian, Hint, InstructionId, Intrinsic};
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    types::NumericType,
    value::{Value, ValueId},
};

use super::brillig_black_box::convert_black_box_call;
use crate::brillig::brillig_ir::brillig_variable::type_to_heap_value_type;

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Converts a foreign function call into Brillig bytecode.
    ///
    /// Foreign functions are external host functions that interact with the Brillig VM.
    /// This method handles the conversion of inputs/outputs and manages memory allocation.
    fn convert_ssa_foreign_call(
        &mut self,
        func_name: &str,
        arguments: &[ValueId],
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
    ) {
        let result_ids = dfg.instruction_results(instruction_id);

        let input_values = vecmap(arguments, |value_id| {
            let variable = self.convert_ssa_value(*value_id, dfg);
            self.brillig_context.variable_to_value_or_array(variable)
        });
        let input_value_types = vecmap(arguments, |value_id| {
            let value_type = dfg.type_of_value(*value_id);
            type_to_heap_value_type(&value_type)
        });

        let output_variables =
            vecmap(result_ids, |value_id| self.allocate_external_call_result(*value_id, dfg));
        let output_values = vecmap(&output_variables, |variable| {
            self.brillig_context.variable_to_value_or_array(*variable)
        });
        let output_value_types = vecmap(result_ids, |value_id| {
            let value_type = dfg.type_of_value(*value_id);
            type_to_heap_value_type(&value_type)
        });

        self.brillig_context.foreign_call_instruction(
            func_name.to_owned(),
            &vecmap(input_values, |v| *v),
            &input_value_types,
            &vecmap(&output_values, |v| **v),
            &output_value_types,
        );

        // Massage the return value.
        for (i, (output_register, output_variable)) in
            output_values.iter().zip(output_variables).enumerate()
        {
            match **output_register {
                // Returned vectors need to emit some bytecode to format the result as a BrilligVector
                ValueOrArray::HeapVector(heap_vector) => {
                    self.brillig_context.initialize_externally_returned_vector(
                        output_variable.extract_vector(),
                        heap_vector,
                    );
                    // Update the dynamic slice length maintained in SSA,
                    // which is the parameter preceding the vector.
                    if let ValueOrArray::MemoryAddress(len_index) = *output_values[i - 1] {
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
                }
                ValueOrArray::HeapArray(_) | ValueOrArray::MemoryAddress(_) => {}
            }
        }
    }

    /// Converts a field less than comparison intrinsic to Brillig bytecode.
    ///
    /// Compares two field elements and returns a boolean result.
    fn convert_ssa_field_less_than(
        &mut self,
        arguments: &[ValueId],
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
    ) {
        let lhs = self.convert_ssa_single_addr_value(arguments[0], dfg);
        assert!(lhs.bit_size == FieldElement::max_num_bits());
        let rhs = self.convert_ssa_single_addr_value(arguments[1], dfg);
        assert!(rhs.bit_size == FieldElement::max_num_bits());

        let [result] = dfg.instruction_result(instruction_id);
        let destination = self
            .variables
            .define_variable(self.function_context, self.brillig_context, result, dfg)
            .extract_single_addr();
        assert!(destination.bit_size == 1);

        self.brillig_context.binary_instruction(lhs, rhs, destination, BrilligBinaryOp::LessThan);
    }

    /// Converts a black box function call into Brillig bytecode.
    ///
    /// Black box functions are native cryptographic operations or other optimized primitives.
    /// Some black box functions (ECDSA, MultiScalarMul, EmbeddedCurveAdd) have a predicate
    /// argument in SSA that is removed for Brillig as CFG flattening is not needed.
    fn convert_ssa_black_box_call(
        &mut self,
        bb_func: &BlackBoxFunc,
        arguments: &[ValueId],
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
    ) {
        assert!(
            !arguments.iter().any(|arg| dfg.type_of_value(*arg).contains_slice_element()),
            "Blackbox functions should not be called with arguments of slice type"
        );

        let mut arguments = arguments.to_vec();
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
            let predicate = arguments
                .pop()
                .expect("ICE: ECDSA black box function must have a predicate argument");
            assert_eq!(
                dfg.get_numeric_constant_with_type(predicate),
                Some((FieldElement::one(), NumericType::bool())),
                "ICE: ECDSA black box function must have a predicate argument with value 1"
            );
        }

        let function_arguments = vecmap(arguments, |arg| self.convert_ssa_value(arg, dfg));
        let function_results = dfg.instruction_results(instruction_id);
        let function_results =
            vecmap(function_results, |result| self.allocate_external_call_result(*result, dfg));
        convert_black_box_call(
            self.brillig_context,
            bb_func,
            &function_arguments,
            &function_results,
        );
    }

    /// Converts an array to a slice by copying the array contents into a vector.
    ///
    /// This intrinsic converts a fixed-size array into a dynamically-sized slice (vector).
    fn convert_ssa_as_slice(
        &mut self,
        arguments: &[ValueId],
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
    ) {
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

        let source_size_register =
            self.brillig_context.make_usize_constant_instruction(source_array.size.into());

        // we need to explicitly set the destination_len_variable
        self.brillig_context.codegen_usize_op(
            source_size_register.address,
            destination_len_variable.address,
            BrilligBinaryOp::UnsignedDiv,
            element_size,
        );

        self.brillig_context.codegen_initialize_vector(
            destination_vector,
            *source_size_register,
            None,
        );

        // Items
        let vector_items_pointer =
            self.brillig_context.codegen_make_vector_items_pointer(destination_vector);
        let array_items_pointer =
            self.brillig_context.codegen_make_array_items_pointer(source_array);

        self.brillig_context.codegen_mem_copy(
            *array_items_pointer,
            *vector_items_pointer,
            *source_size_register,
        );
    }

    pub(crate) fn call_gen(
        &mut self,
        instruction_id: InstructionId,
        func: ValueId,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) {
        match &dfg[func] {
            Value::ForeignFunction(func_name) => {
                self.convert_ssa_foreign_call(func_name, arguments, instruction_id, dfg);
            }
            Value::Function(func_id) => {
                let result_ids = dfg.instruction_results(instruction_id);
                self.convert_ssa_function_call(*func_id, arguments, dfg, result_ids);
            }
            Value::Intrinsic(intrinsic) => {
                // This match could be combined with the above but without it rust analyzer
                // can't automatically insert any missing cases
                match intrinsic {
                    Intrinsic::AsSlice => {
                        self.convert_ssa_as_slice(arguments, instruction_id, dfg);
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
                            *two,
                            matches!(endianness, Endian::Little),
                            true,
                        );
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
                        self.convert_ssa_black_box_call(bb_func, arguments, instruction_id, dfg);
                    }
                    // `Intrinsic::AsWitness` is used to provide hints to acir-gen on optimal expression splitting.
                    // It is then useless in the brillig runtime and so we can ignore it
                    Intrinsic::AsWitness => (),
                    Intrinsic::FieldLessThan => {
                        self.convert_ssa_field_less_than(arguments, instruction_id, dfg);
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
                    Intrinsic::ApplyRangeConstraint => {
                        unreachable!(
                            "ICE: `Intrinsic::ApplyRangeConstraint` calls should be transformed into an `Instruction::RangeCheck`"
                        );
                    }
                    Intrinsic::DerivePedersenGenerators => {
                        unreachable!("unsupported function call type {:?}", dfg[func])
                    }
                    Intrinsic::IsUnconstrained
                    | Intrinsic::ArrayLen
                    | Intrinsic::ArrayAsStrUnchecked
                    | Intrinsic::StrAsBytes
                    | Intrinsic::StaticAssert
                    | Intrinsic::AssertConstant => {
                        unreachable!(
                            "Expected {intrinsic} to have been removing during SSA optimizations"
                        )
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
