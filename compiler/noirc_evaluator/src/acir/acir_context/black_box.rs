use acvm::acir::{
    AcirField, BlackBoxFunc, brillig::lengths::FlattenedLength, circuit::opcodes::FunctionInput,
};
use iter_extended::vecmap;

use crate::{
    brillig::assert_usize,
    errors::{InternalError, RuntimeError},
    ssa::ir::types::NumericType,
};

use super::{AcirContext, AcirValue, AcirVar};

impl<F: AcirField> AcirContext<F> {
    /// Calls a Blackbox function on the given inputs and returns a given set of outputs
    /// to represent the result of the blackbox function.
    pub(crate) fn black_box_function(
        &mut self,
        name: BlackBoxFunc,
        mut inputs: Vec<AcirValue>,
        num_bits: Option<u32>,
        output_count: FlattenedLength,
        predicate: Option<AcirVar>,
    ) -> Result<Vec<AcirVar>, RuntimeError> {
        let output_count = assert_usize(output_count.0);
        // Separate out any arguments that should be constants
        let constant_inputs = match name {
            BlackBoxFunc::AES128Encrypt => {
                let invalid_input = "aes128_encrypt - operation requires a plaintext to encrypt";
                let input_size: usize = match inputs.first().expect(invalid_input) {
                    AcirValue::Array(values) => Ok::<usize, RuntimeError>(values.len()),
                    AcirValue::DynamicArray(dyn_array) => {
                        Ok::<usize, RuntimeError>(assert_usize(dyn_array.len.0))
                    }
                    _ => {
                        return Err(RuntimeError::InternalError(InternalError::General {
                            message: "aes128_encrypt requires an array of inputs".to_string(),
                            call_stack: self.get_call_stack(),
                        }));
                    }
                }?;

                assert_eq!(
                    output_count,
                    input_size + 16 - input_size % 16,
                    "output count mismatch"
                );

                Vec::new()
            }
            BlackBoxFunc::RecursiveAggregation => {
                let proof_type_var = match inputs.pop() {
                    Some(domain_var) => domain_var.into_var()?,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::MissingArg {
                            name: "verify proof".to_string(),
                            arg: "proof type".to_string(),
                            call_stack: self.get_call_stack(),
                        }));
                    }
                };

                let proof_type_constant = match self.var_to_expression(proof_type_var)?.to_const() {
                    Some(proof_type_constant) => *proof_type_constant,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::NotAConstant {
                            name: "proof type".to_string(),
                            call_stack: self.get_call_stack(),
                        }));
                    }
                };
                inputs.push(AcirValue::Var(predicate.unwrap(), NumericType::bool()));
                vec![proof_type_constant]
            }
            _ => Vec::new(),
        };
        let inputs = self.prepare_inputs_for_black_box_func(inputs, name)?;

        let output_vars = vecmap(0..output_count, |_| self.add_variable());
        let output_witnesses = vecmap(&output_vars, |var| {
            self.var_to_witness(*var).expect("variable was just created as witness")
        });

        self.acir_ir.call_black_box(name, inputs, constant_inputs, num_bits, output_witnesses)?;

        // Convert `Witness` values which are now constrained to be the output of the
        // black box function call into `AcirVar`s.
        //
        // We do not apply range information on the output of the black box function.
        // See issue #1439
        Ok(output_vars)
    }

    pub(super) fn prepare_inputs_for_black_box_func(
        &mut self,
        mut inputs: Vec<AcirValue>,
        name: BlackBoxFunc,
    ) -> Result<Vec<Vec<FunctionInput<F>>>, RuntimeError> {
        // Allow constant inputs for most blackbox
        // Allow constant predicate for all blackbox having predicate
        let inputs = match name {
            BlackBoxFunc::MultiScalarMul
            | BlackBoxFunc::Keccakf1600
            | BlackBoxFunc::Blake2s
            | BlackBoxFunc::Blake3
            | BlackBoxFunc::AND
            | BlackBoxFunc::XOR
            | BlackBoxFunc::AES128Encrypt
            | BlackBoxFunc::EmbeddedCurveAdd => {
                self.prepare_inputs_for_black_box_func_call(inputs, true)?
            }
            BlackBoxFunc::EcdsaSecp256k1 | BlackBoxFunc::EcdsaSecp256r1 => {
                // ECDSA blackbox functions have 6 inputs, the last ones are: [.., predicate, output]
                let predicate = inputs.swap_remove(4);
                // convert the inputs into witness, except for the predicate which has been removed
                let mut inputs = self.prepare_inputs_for_black_box_func_call(inputs, false)?;
                // convert the predicate into witness or constant
                let predicate = self.value_to_function_input(predicate)?;
                // Sanity check: proving system does not expect to receive 0 predicates
                assert_ne!(
                    predicate,
                    FunctionInput::Constant(F::zero()),
                    "0 predicate should have been optimized away"
                );

                // add back the predicate into the FunctionInputs
                inputs.insert(4, vec![predicate]);
                inputs
            }
            BlackBoxFunc::RecursiveAggregation => {
                let predicate = inputs.pop().ok_or_else(|| {
                    RuntimeError::InternalError(InternalError::MissingArg {
                        name: "recursive aggregation".to_string(),
                        arg: "predicate".to_string(),
                        call_stack: self.get_call_stack(),
                    })
                })?;
                // convert the inputs into witness, except for the predicate which has been removed
                let mut inputs = self.prepare_inputs_for_black_box_func_call(inputs, false)?;
                // convert the predicate into witness or constant
                let predicate = self.value_to_function_input(predicate)?;
                // Sanity check: proving system does not expect to receive 0 predicates
                assert_ne!(
                    predicate,
                    FunctionInput::Constant(F::zero()),
                    "0 predicate should have been optimized away"
                );
                // add back the predicate into the FunctionInputs
                inputs.push(vec![predicate]);
                inputs
            }
            _ => self.prepare_inputs_for_black_box_func_call(inputs, false)?,
        };
        Ok(inputs)
    }

    /// Black box function calls expect their inputs to be in a specific data structure (FunctionInput).
    ///
    /// This function will convert `AcirVar` into `FunctionInput` for a blackbox function call.
    pub(super) fn prepare_inputs_for_black_box_func_call(
        &mut self,
        inputs: Vec<AcirValue>,
        allow_constant_inputs: bool,
    ) -> Result<Vec<Vec<FunctionInput<F>>>, RuntimeError> {
        let mut witnesses = Vec::new();
        for input in inputs {
            let mut single_val_witnesses = Vec::new();
            for (input, typ) in self.flatten(input)? {
                let num_bits = typ.bit_size::<F>();
                match self.var_to_expression(input)?.to_const() {
                    Some(constant) if allow_constant_inputs => {
                        if num_bits < constant.num_bits() {
                            return Err(RuntimeError::InvalidBlackBoxInputBitSize {
                                value: constant.to_string(),
                                num_bits: constant.num_bits(),
                                max_num_bits: num_bits,
                                call_stack: self.get_call_stack(),
                            });
                        }
                        single_val_witnesses.push(FunctionInput::Constant(*constant));
                    }
                    _ => {
                        let witness_var = self.get_or_create_witness_var(input)?;
                        let witness = self.var_to_witness(witness_var)?;
                        single_val_witnesses.push(FunctionInput::Witness(witness));
                    }
                }
            }
            witnesses.push(single_val_witnesses);
        }
        Ok(witnesses)
    }

    /// Converts an `AcirValue` into a `FunctionInput` for use in black box function calls.
    ///
    /// - If the value can be evaluated to a constant, it returns `FunctionInput::Constant`
    /// - Otherwise, it creates or retrieves a witness variable and returns `FunctionInput::Witness`
    fn value_to_function_input(
        &mut self,
        value: AcirValue,
    ) -> Result<FunctionInput<F>, RuntimeError> {
        if let AcirValue::Var(acir_var, acir_type) = value {
            if let Some(constant) = self.var_to_expression(acir_var)?.to_const() {
                let num_bits = acir_type.bit_size::<F>();
                if num_bits < constant.num_bits() {
                    return Err(RuntimeError::InvalidBlackBoxInputBitSize {
                        value: constant.to_string(),
                        num_bits: constant.num_bits(),
                        max_num_bits: num_bits,
                        call_stack: self.get_call_stack(),
                    });
                }
                Ok(FunctionInput::Constant(*constant))
            } else {
                let witness_var = self.get_or_create_witness_var(acir_var)?;
                let witness = self.var_to_witness(witness_var)?;
                Ok(FunctionInput::Witness(witness))
            }
        } else {
            Err(RuntimeError::InternalError(InternalError::General {
                message: "Expected AcirValue".to_string(),
                call_stack: self.get_call_stack(),
            }))
        }
    }
}
