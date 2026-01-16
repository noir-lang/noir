use acvm::acir::{
    AcirField,
    circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
    native_types::{Expression, Witness},
};
use iter_extended::{try_vecmap, vecmap};

use crate::brillig::{assert_usize, brillig_ir::artifact::GeneratedBrillig};
use crate::errors::{InternalError, RuntimeError};

use super::generated_acir::{BrilligStdlibFunc, PLACEHOLDER_BRILLIG_INDEX};
use super::{AcirContext, AcirDynamicArray, AcirType, AcirValue, AcirVar};

impl<F: AcirField> AcirContext<F> {
    /// Generates a brillig call to a handwritten section of brillig bytecode.
    pub(crate) fn stdlib_brillig_call(
        &mut self,
        predicate: AcirVar,
        brillig_stdlib_func: BrilligStdlibFunc,
        inputs: Vec<AcirValue>,
        outputs: Vec<AcirType>,
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let stdlib_func_bytecode = &self.brillig_stdlib.get_code(brillig_stdlib_func).clone();
        let safe_return_values = false;
        self.brillig_call(
            predicate,
            stdlib_func_bytecode,
            inputs,
            outputs,
            safe_return_values,
            PLACEHOLDER_BRILLIG_INDEX,
            Some(brillig_stdlib_func),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn brillig_call(
        &mut self,
        predicate: AcirVar,
        generated_brillig: &GeneratedBrillig<F>,
        inputs: Vec<AcirValue>,
        outputs: Vec<AcirType>,
        unsafe_return_values: bool,
        brillig_function_index: BrilligFunctionId,
        brillig_stdlib_func: Option<BrilligStdlibFunc>,
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        let predicate = self.var_to_expression(predicate)?;
        if predicate.is_zero() {
            // If the predicate has a constant value of zero, the brillig call will never be executed.
            // We can then immediately zero out all of its outputs as this is the value which would be written
            // if we waited until runtime to resolve this call.
            let outputs_var = vecmap(outputs, |output| match output {
                AcirType::NumericType(numeric_type) => {
                    let var = self.add_constant(F::zero());
                    AcirValue::Var(var, numeric_type)
                }
                AcirType::Array(element_types, size) => {
                    self.zeroed_array_output(&element_types, size)
                }
            });

            return Ok(outputs_var);
        }
        // Remove "always true" predicates.
        let predicate = if predicate == Expression::one() { None } else { Some(predicate) };

        let brillig_inputs: Vec<BrilligInputs<F>> =
            try_vecmap(inputs, |i| -> Result<_, InternalError> {
                match i {
                    AcirValue::Var(var, _) => {
                        Ok(BrilligInputs::Single(self.var_to_expression(var)?))
                    }
                    AcirValue::Array(vars) => {
                        let mut var_expressions: Vec<Expression<F>> = Vec::new();
                        for var in vars {
                            self.brillig_array_input(&mut var_expressions, var)?;
                        }
                        Ok(BrilligInputs::Array(var_expressions))
                    }
                    AcirValue::DynamicArray(AcirDynamicArray { block_id, .. }) => {
                        Ok(BrilligInputs::MemoryArray(block_id))
                    }
                }
            })?;

        let mut brillig_outputs = Vec::new();
        let outputs_var = vecmap(outputs, |output| match output {
            AcirType::NumericType(numeric_type) => {
                let var = self.add_variable();
                let witness_index =
                    self.var_to_witness(var).expect("variable has just been created as witness");
                brillig_outputs.push(BrilligOutputs::Simple(witness_index));
                AcirValue::Var(var, numeric_type)
            }
            AcirType::Array(element_types, size) => {
                let (acir_value, witnesses) = self.brillig_array_output(&element_types, size);
                brillig_outputs.push(BrilligOutputs::Array(witnesses));
                acir_value
            }
        });

        self.acir_ir.brillig_call(
            predicate,
            generated_brillig,
            brillig_inputs,
            brillig_outputs,
            brillig_function_index,
            brillig_stdlib_func,
        );

        fn range_constraint_value<G: AcirField>(
            context: &mut AcirContext<G>,
            value: &AcirValue,
        ) -> Result<(), RuntimeError> {
            let one = context.add_constant(G::one());
            match value {
                AcirValue::Var(var, numeric_type) => {
                    // Predicate is one so that the constrain is always applied, because
                    // values returned from Brillig will be 0 under a false predicate.
                    if !numeric_type.is_field() {
                        context.range_constrain_var(
                            *var,
                            numeric_type.bit_size::<G>(),
                            None,
                            one,
                        )?;
                    }
                }
                AcirValue::Array(values) => {
                    for value in values {
                        range_constraint_value(context, value)?;
                    }
                }
                AcirValue::DynamicArray(_) => {
                    unreachable!("Brillig opcodes cannot return dynamic arrays")
                }
            }
            Ok(())
        }

        // This is a hack to ensure that if we're compiling a brillig entrypoint function then
        // we don't also add a number of range constraints.
        if !unsafe_return_values {
            for output_var in &outputs_var {
                range_constraint_value(self, output_var)?;
            }
        }
        Ok(outputs_var)
    }

    fn brillig_array_input(
        &mut self,
        var_expressions: &mut Vec<Expression<F>>,
        input: AcirValue,
    ) -> Result<(), InternalError> {
        match input {
            AcirValue::Var(var, _) => {
                var_expressions.push(self.var_to_expression(var)?);
            }
            AcirValue::Array(vars) => {
                for var in vars {
                    self.brillig_array_input(var_expressions, var)?;
                }
            }
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, value_types, .. }) => {
                for i in 0..assert_usize(len.0) {
                    // We generate witnesses corresponding to the array values
                    let index_var = self.add_constant(i);

                    let value_read_var = self.read_from_memory(block_id, &index_var)?;
                    let value_typ = value_types[i % value_types.len()];
                    let value_read = AcirValue::Var(value_read_var, value_typ);

                    self.brillig_array_input(var_expressions, value_read)?;
                }
            }
        }
        Ok(())
    }

    /// Recursively create zeroed-out acir values for returned arrays. This is necessary because a brillig returned array can have nested arrays as elements.
    fn zeroed_array_output(&mut self, element_types: &[AcirType], size: usize) -> AcirValue {
        let mut array_values = im::Vector::new();
        for _ in 0..size {
            for element_type in element_types {
                match element_type {
                    AcirType::Array(nested_element_types, nested_size) => {
                        let nested_acir_value =
                            self.zeroed_array_output(nested_element_types, *nested_size);
                        array_values.push_back(nested_acir_value);
                    }
                    AcirType::NumericType(numeric_type) => {
                        let var = self.add_constant(F::zero());
                        array_values.push_back(AcirValue::Var(var, *numeric_type));
                    }
                }
            }
        }
        AcirValue::Array(array_values)
    }

    /// Recursively create acir values for returned arrays. This is necessary because a brillig returned array can have nested arrays as elements.
    /// A singular array of witnesses is collected for a top level array, by deflattening the assigned witnesses at each level.
    fn brillig_array_output(
        &mut self,
        element_types: &[AcirType],
        size: usize,
    ) -> (AcirValue, Vec<Witness>) {
        let mut witnesses = Vec::new();
        let mut array_values = im::Vector::new();
        for _ in 0..size {
            for element_type in element_types {
                match element_type {
                    AcirType::Array(nested_element_types, nested_size) => {
                        let (nested_acir_value, mut nested_witnesses) =
                            self.brillig_array_output(nested_element_types, *nested_size);
                        witnesses.append(&mut nested_witnesses);
                        array_values.push_back(nested_acir_value);
                    }
                    AcirType::NumericType(numeric_type) => {
                        let var = self.add_variable();
                        array_values.push_back(AcirValue::Var(var, *numeric_type));
                        witnesses.push(
                            self.var_to_witness(var)
                                .expect("variable has just been created as witness"),
                        );
                    }
                }
            }
        }
        (AcirValue::Array(array_values), witnesses)
    }
}
