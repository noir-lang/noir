use acvm::acir::{AcirField, BlackBoxFunc, circuit::opcodes::FunctionInput};
use iter_extended::vecmap;

use crate::errors::{InternalError, RuntimeError};

use super::{AcirContext, AcirType, AcirValue, AcirVar};

impl<F: AcirField> AcirContext<F> {
    /// Calls a Blackbox function on the given inputs and returns a given set of outputs
    /// to represent the result of the blackbox function.
    pub(crate) fn black_box_function(
        &mut self,
        name: BlackBoxFunc,
        mut inputs: Vec<AcirValue>,
        num_bits: Option<u32>,
        output_count: usize,
        predicate: Option<AcirVar>,
    ) -> Result<Vec<AcirVar>, RuntimeError> {
        // Separate out any arguments that should be constants
        let constant_inputs = match name {
            BlackBoxFunc::AES128Encrypt => {
                let invalid_input = "aes128_encrypt - operation requires a plaintext to encrypt";
                let input_size: usize = match inputs.first().expect(invalid_input) {
                    AcirValue::Array(values) => Ok::<usize, RuntimeError>(values.len()),
                    AcirValue::DynamicArray(dyn_array) => Ok::<usize, RuntimeError>(dyn_array.len),
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
                inputs.push(AcirValue::Var(predicate.unwrap(), AcirType::unsigned(1)));
                vec![proof_type_constant]
            }
            _ => Vec::new(),
        };
        let inputs = self.prepare_inputs_for_black_box_func(inputs, name)?;

        let output_vars = vecmap(0..output_count, |_| self.add_variable());
        let output_witnesses = vecmap(&output_vars, |var| {
            self.var_to_witness(*var).expect("variable was just created as witness")
        });

        self.acir_ir.call_black_box(name, &inputs, constant_inputs, num_bits, output_witnesses)?;

        // Convert `Witness` values which are now constrained to be the output of the
        // black box function call into `AcirVar`s.
        //
        // We do not apply range information on the output of the black box function.
        // See issue #1439
        Ok(output_vars)
    }

    pub(super) fn prepare_inputs_for_black_box_func(
        &mut self,
        inputs: Vec<AcirValue>,
        name: BlackBoxFunc,
    ) -> Result<Vec<Vec<FunctionInput<F>>>, RuntimeError> {
        // Allow constant inputs for most blackbox, but:
        // - EmbeddedCurveAdd requires all-or-nothing constant inputs
        // - Poseidon2Permutation requires witness input
        let allow_constant_inputs = matches!(
            name,
            BlackBoxFunc::MultiScalarMul
                | BlackBoxFunc::Keccakf1600
                | BlackBoxFunc::Blake2s
                | BlackBoxFunc::Blake3
                | BlackBoxFunc::AND
                | BlackBoxFunc::XOR
                | BlackBoxFunc::AES128Encrypt
                | BlackBoxFunc::EmbeddedCurveAdd
        );
        // Convert `AcirVar` to `FunctionInput`
        let mut inputs =
            self.prepare_inputs_for_black_box_func_call(inputs, allow_constant_inputs)?;
        if name == BlackBoxFunc::EmbeddedCurveAdd {
            inputs = self.all_variables_or_constants_for_ec_add(inputs)?;
        }
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

    /// [`BlackBoxFunc::EmbeddedCurveAdd`] has 6 inputs representing the two points to add
    /// Each point must be either all constants, or all witnesses,
    /// where the constants are converted to witnesses here if mixed constant and witnesses,
    fn all_variables_or_constants_for_ec_add(
        &mut self,
        inputs: Vec<Vec<FunctionInput<F>>>,
    ) -> Result<Vec<Vec<FunctionInput<F>>>, RuntimeError> {
        let mut has_constant = false;
        let mut has_witness = false;
        let mut result = inputs.clone();
        for (i, input) in inputs.iter().enumerate() {
            assert_eq!(input.len(), 1);
            if input[0].is_constant() {
                has_constant = true;
            } else {
                has_witness = true;
            }

            if i % 3 == 2 {
                if has_constant && has_witness {
                    // Convert the constants to witnesses if mixed constants and witnesses,
                    for j in i - 2..i + 1 {
                        if let FunctionInput::Constant(constant) = inputs[j][0] {
                            let constant = self.add_constant(constant);
                            let witness_var = self.get_or_create_witness_var(constant)?;
                            let witness = self.var_to_witness(witness_var)?;
                            result[j] = vec![FunctionInput::Witness(witness)];
                        }
                    }
                }
                has_constant = false;
                has_witness = false;
            }
        }
        Ok(result)
    }
}
