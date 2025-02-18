use acvm::{
    acir::{
        brillig::Opcode as BrilligOpcode,
        circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
        native_types::{Expression, Witness},
        AcirField,
    },
    brillig_vm::{MemoryValue, VMStatus, VM},
    BlackBoxFunctionSolver,
};
use iter_extended::{try_vecmap, vecmap};

use crate::errors::{InternalError, RuntimeError};
use crate::{acir::acir_variable::AcirContext, brillig::brillig_ir::artifact::GeneratedBrillig};

use super::acir_variable::{AcirType, AcirVar};
use super::generated_acir::BrilligStdlibFunc;
use super::{AcirDynamicArray, AcirValue};

impl<F: AcirField, B: BlackBoxFunctionSolver<F>> AcirContext<F, B> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn brillig_call(
        &mut self,
        predicate: AcirVar,
        generated_brillig: &GeneratedBrillig<F>,
        inputs: Vec<AcirValue>,
        outputs: Vec<AcirType>,
        attempt_execution: bool,
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
                AcirType::NumericType(_) => {
                    let var = self.add_constant(F::zero());
                    AcirValue::Var(var, output.clone())
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

        // Optimistically try executing the brillig now, if we can complete execution they just return the results.
        // This is a temporary measure pending SSA optimizations being applied to Brillig which would remove constant-input opcodes (See #2066)
        //
        // We do _not_ want to do this in the situation where the `main` function is unconstrained, as if execution succeeds
        // the entire program will be replaced with witness constraints to its outputs.
        if attempt_execution {
            if let Some(brillig_outputs) =
                self.execute_brillig(&generated_brillig.byte_code, &brillig_inputs, &outputs)
            {
                return Ok(brillig_outputs);
            }
        }

        // Otherwise we must generate ACIR for it and execute at runtime.
        let mut brillig_outputs = Vec::new();
        let outputs_var = vecmap(outputs, |output| match output {
            AcirType::NumericType(_) => {
                let var = self.add_variable();
                let witness_index =
                    self.var_to_witness(var).expect("variable has just been created as witness");
                brillig_outputs.push(BrilligOutputs::Simple(witness_index));
                AcirValue::Var(var, output.clone())
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

        fn range_constraint_value<G: AcirField, C: BlackBoxFunctionSolver<G>>(
            context: &mut AcirContext<G, C>,
            value: &AcirValue,
        ) -> Result<(), RuntimeError> {
            match value {
                AcirValue::Var(var, typ) => {
                    let numeric_type = match typ {
                        AcirType::NumericType(numeric_type) => numeric_type,
                        _ => unreachable!("`AcirValue::Var` may only hold primitive values"),
                    };
                    context.range_constrain_var(*var, numeric_type, None)?;
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
            AcirValue::DynamicArray(AcirDynamicArray { block_id, len, .. }) => {
                for i in 0..len {
                    // We generate witnesses corresponding to the array values
                    let index_var = self.add_constant(i);

                    let value_read_var = self.read_from_memory(block_id, &index_var)?;
                    let value_read = AcirValue::Var(value_read_var, AcirType::field());

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
                    AcirType::NumericType(_) => {
                        let var = self.add_constant(F::zero());
                        array_values.push_back(AcirValue::Var(var, element_type.clone()));
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
                    AcirType::NumericType(_) => {
                        let var = self.add_variable();
                        array_values.push_back(AcirValue::Var(var, element_type.clone()));
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

    fn execute_brillig(
        &mut self,
        code: &[BrilligOpcode<F>],
        inputs: &[BrilligInputs<F>],
        outputs_types: &[AcirType],
    ) -> Option<Vec<AcirValue>> {
        let mut memory = (execute_brillig(code, &self.blackbox_solver, inputs)?).into_iter();

        let outputs_var = vecmap(outputs_types.iter(), |output| match output {
            AcirType::NumericType(_) => {
                let var = self.add_constant(memory.next().expect("Missing return data").to_field());
                AcirValue::Var(var, output.clone())
            }
            AcirType::Array(element_types, size) => {
                self.brillig_constant_array_output(element_types, *size, &mut memory)
            }
        });

        Some(outputs_var)
    }

    /// Recursively create [`AcirValue`]s for returned arrays. This is necessary because a brillig returned array can have nested arrays as elements.
    fn brillig_constant_array_output(
        &mut self,
        element_types: &[AcirType],
        size: usize,
        memory_iter: &mut impl Iterator<Item = MemoryValue<F>>,
    ) -> AcirValue {
        let mut array_values = im::Vector::new();
        for _ in 0..size {
            for element_type in element_types {
                match element_type {
                    AcirType::Array(nested_element_types, nested_size) => {
                        let nested_acir_value = self.brillig_constant_array_output(
                            nested_element_types,
                            *nested_size,
                            memory_iter,
                        );
                        array_values.push_back(nested_acir_value);
                    }
                    AcirType::NumericType(_) => {
                        let memory_value =
                            memory_iter.next().expect("ICE: Unexpected end of memory");
                        let var = self.add_constant(memory_value.to_field());
                        array_values.push_back(AcirValue::Var(var, element_type.clone()));
                    }
                }
            }
        }
        AcirValue::Array(array_values)
    }
}

/// Attempts to execute the provided [`Brillig`][`acvm::acir::brillig`] bytecode
///
/// Returns the finished state of the Brillig VM if execution can complete.
///
/// Returns `None` if complete execution of the Brillig bytecode is not possible.
fn execute_brillig<F: AcirField, B: BlackBoxFunctionSolver<F>>(
    code: &[BrilligOpcode<F>],
    blackbox_solver: &B,
    inputs: &[BrilligInputs<F>],
) -> Option<Vec<MemoryValue<F>>> {
    // Set input values
    let mut calldata: Vec<F> = Vec::new();

    // Each input represents a constant or array of constants.
    // Iterate over each input and push it into registers and/or memory.
    for input in inputs {
        match input {
            BrilligInputs::Single(expr) => {
                calldata.push(*expr.to_const()?);
            }
            BrilligInputs::Array(expr_arr) => {
                // Attempt to fetch all array input values
                for expr in expr_arr.iter() {
                    calldata.push(*expr.to_const()?);
                }
            }
            BrilligInputs::MemoryArray(_) => {
                return None;
            }
        }
    }

    // Instantiate a Brillig VM given the solved input registers and memory, along with the Brillig bytecode.
    let profiling_active = false;
    let mut vm = VM::new(calldata, code, blackbox_solver, profiling_active);

    // Run the Brillig VM on these inputs, bytecode, etc!
    let vm_status = vm.process_opcodes();

    // Check the status of the Brillig VM.
    // It may be finished, in-progress, failed, or may be waiting for results of a foreign call.
    // If it's finished then we can omit the opcode and just write in the return values.
    match vm_status {
        VMStatus::Finished { return_data_offset, return_data_size } => Some(
            vm.get_memory()[return_data_offset..(return_data_offset + return_data_size)].to_vec(),
        ),
        VMStatus::InProgress => unreachable!("Brillig VM has not completed execution"),
        VMStatus::Failure { .. } => {
            // TODO: Return an error stating that the brillig function failed.
            None
        }
        VMStatus::ForeignCallWait { .. } => {
            // If execution can't complete then keep the opcode

            // TODO: We could bake in all the execution up to this point by replacing the inputs
            // such that they initialize the registers/memory to the current values and then discard
            // any opcodes prior to the one which performed this foreign call.
            //
            // Seems overkill for now however.
            None
        }
    }
}
