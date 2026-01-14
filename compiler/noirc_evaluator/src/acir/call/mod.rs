use acvm::AcirField;
use acvm::acir::brillig::lengths::{
    ElementsFlattenedLength, ElementTypesLength, FlattenedLength, SemanticLength, SemiFlattenedLength,
};
use acvm::acir::circuit::opcodes::AcirFunctionId;
use iter_extended::vecmap;
use noirc_artifacts::ssa::SsaReport;

use crate::acir::AcirVar;
use crate::brillig::assert_u32;
use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
use crate::brillig::brillig_gen::gen_brillig_for;
use crate::brillig::brillig_ir::artifact::BrilligParameter;
use crate::errors::RuntimeError;
use crate::ssa::ir::value::Value;
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::{Function, FunctionId, RuntimeType},
    instruction::Instruction,
    types::Type,
    value::ValueId,
};
use crate::ssa::ssa_gen::Ssa;

use super::{
    Context, arrays,
    types::{AcirDynamicArray, AcirType, AcirValue},
};

mod intrinsics;

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
        // Check that we are not attempting to return a vector from
        // an unconstrained runtime to a constrained runtime
        for result_id in result_ids {
            if dfg.type_of_value(*result_id).contains_vector_element() {
                return Err(RuntimeError::UnconstrainedVectorReturnToConstrained {
                    call_stack: self.acir_context.get_call_stack(),
                });
            }
        }

        let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));
        let output_count: FlattenedLength =
            result_ids.iter().map(|result_id| dfg.type_of_value(*result_id).flattened_size()).sum();

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
            let safe_return_values = false;
            self.acir_context.brillig_call(
                self.current_side_effects_enabled_var,
                code,
                inputs,
                outputs,
                safe_return_values,
                *generated_pointer,
                None,
            )?
        } else {
            let code =
                gen_brillig_for(func, arguments.clone(), self.brillig, self.brillig_options)?;
            let generated_pointer = self.shared_context.new_generated_pointer();
            let safe_return_values = false;
            let output_values = self.acir_context.brillig_call(
                self.current_side_effects_enabled_var,
                &code,
                inputs,
                outputs,
                safe_return_values,
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
                if let Type::Vector(item_types) = typ {
                    let len = match self
                        .ssa_values
                        .get(&value_id)
                        .expect("ICE: Unknown vector input to brillig")
                    {
                        AcirValue::DynamicArray(AcirDynamicArray { len, .. }) => {
                            // len holds the flattened length of all elements in the vector,
                            // so to get the no-flattened length we need to divide by the flattened
                            // length of a single vector entry
                            let sum: FlattenedLength =
                                item_types.iter().map(|typ| typ.flattened_size()).sum();
                            if sum.0 == 0 {
                                SemanticLength(0)
                            } else {
                                *len / ElementsFlattenedLength::from(sum)
                            }
                        }
                        AcirValue::Array(array) => {
                            if item_types.is_empty() {
                                SemanticLength(0)
                            } else {
                                // len holds the semi-flattened length of all elements in the vector,
                                // so here we need to divide by elements length of the item types
                                let len = SemiFlattenedLength(assert_u32(array.len()));
                                len / ElementTypesLength(assert_u32(item_types.len()))
                            }
                        }
                        _ => unreachable!("ICE: Vector value is not an array"),
                    };

                    BrilligParameter::Vector(
                        item_types.iter().map(FunctionContext::ssa_type_to_parameter).collect(),
                        len,
                    )
                } else {
                    FunctionContext::ssa_type_to_parameter(&typ)
                }
            })
            .collect()
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
                    array_typ.flattened_size()
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
    /// If the type of a result id is a vector, the vector length must precede it and we can
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
            if let Type::Vector(elements_type) = result_type {
                let error = "ICE - cannot get vector length when converting vector to AcirValue";
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
            Type::Numeric(numeric_type) => {
                let var = vars.next().unwrap();
                AcirValue::Var(var, *numeric_type)
            }
            typ => {
                panic!("Unexpected type {typ} in convert_var_type_to_values");
            }
        }
    }
}
