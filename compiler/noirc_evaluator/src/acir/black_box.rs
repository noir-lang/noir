use acvm::{
    acir::{
        circuit::opcodes::{ConstantOrWitnessEnum, FunctionInput},
        AcirField, BlackBoxFunc,
    },
    BlackBoxFunctionSolver,
};
use iter_extended::vecmap;
use num_bigint::BigUint;

use crate::errors::{InternalError, RuntimeError};

use super::{
    acir_variable::{AcirContext, AcirVar},
    AcirValue,
};

impl<F: AcirField, B: BlackBoxFunctionSolver<F>> AcirContext<F, B> {
    /// Calls a Blackbox function on the given inputs and returns a given set of outputs
    /// to represent the result of the blackbox function.
    pub(crate) fn black_box_function(
        &mut self,
        name: BlackBoxFunc,
        mut inputs: Vec<AcirValue>,
        mut output_count: usize,
    ) -> Result<Vec<AcirVar>, RuntimeError> {
        // Separate out any arguments that should be constants
        let (constant_inputs, constant_outputs) = match name {
            BlackBoxFunc::Poseidon2Permutation => {
                // The last argument is the state length, which must be a constant
                let state_len = match inputs.pop() {
                    Some(state_len) => state_len.into_var()?,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::MissingArg {
                            name: "poseidon_2_permutation call".to_string(),
                            arg: "length".to_string(),
                            call_stack: self.get_call_stack(),
                        }))
                    }
                };

                let state_len = match self.var_to_expression(state_len)?.to_const() {
                    Some(state_len) => *state_len,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::NotAConstant {
                            name: "length".to_string(),
                            call_stack: self.get_call_stack(),
                        }))
                    }
                };

                (vec![state_len], Vec::new())
            }
            BlackBoxFunc::BigIntAdd
            | BlackBoxFunc::BigIntSub
            | BlackBoxFunc::BigIntMul
            | BlackBoxFunc::BigIntDiv => {
                assert_eq!(inputs.len(), 4, "ICE - bigint operation requires 4 inputs");
                let const_inputs = vecmap(inputs, |i| {
                    let var = i.into_var()?;
                    match self.var_to_expression(var)?.to_const() {
                        Some(const_var) => Ok(*const_var),
                        None => Err(RuntimeError::InternalError(InternalError::NotAConstant {
                            name: "big integer".to_string(),
                            call_stack: self.get_call_stack(),
                        })),
                    }
                });
                inputs = Vec::new();
                output_count = 0;
                let mut field_inputs = Vec::new();
                for i in const_inputs {
                    field_inputs.push(i?);
                }
                if field_inputs[1] != field_inputs[3] {
                    return Err(RuntimeError::BigIntModulus { call_stack: self.get_call_stack() });
                }

                let result_id = self.big_int_ctx.new_big_int(field_inputs[1]);
                (
                    vec![field_inputs[0], field_inputs[2]],
                    vec![result_id.bigint_id::<F>(), result_id.modulus_id::<F>()],
                )
            }
            BlackBoxFunc::BigIntToLeBytes => {
                let const_inputs = vecmap(inputs, |i| {
                    let var = i.into_var()?;
                    match self.var_to_expression(var)?.to_const() {
                        Some(const_var) => Ok(*const_var),
                        None => Err(RuntimeError::InternalError(InternalError::NotAConstant {
                            name: "big integer".to_string(),
                            call_stack: self.get_call_stack(),
                        })),
                    }
                });
                inputs = Vec::new();
                let mut field_inputs = Vec::new();
                for i in const_inputs {
                    field_inputs.push(i?);
                }
                let bigint = self.big_int_ctx.get(field_inputs[0]);
                let modulus = self.big_int_ctx.modulus(bigint.modulus_id::<F>());
                let bytes_len = ((modulus - BigUint::from(1_u32)).bits() - 1) / 8 + 1;
                output_count = bytes_len as usize;
                assert!(bytes_len == 32);
                (field_inputs, vec![])
            }
            BlackBoxFunc::BigIntFromLeBytes => {
                let invalid_input = "ICE - bigint operation requires 2 inputs";
                assert_eq!(inputs.len(), 2, "{invalid_input}");
                let mut modulus = Vec::new();
                match inputs.pop().expect(invalid_input) {
                    AcirValue::Array(values) => {
                        for value in values {
                            modulus.push(
                                *self.var_to_expression(value.into_var()?)?.to_const().ok_or(
                                    RuntimeError::InternalError(InternalError::NotAConstant {
                                        name: "big integer".to_string(),
                                        call_stack: self.get_call_stack(),
                                    }),
                                )?,
                            );
                        }
                    }
                    _ => {
                        return Err(RuntimeError::InternalError(InternalError::MissingArg {
                            name: "big_int_from_le_bytes".to_owned(),
                            arg: "modulus".to_owned(),
                            call_stack: self.get_call_stack(),
                        }));
                    }
                }
                let big_modulus = BigUint::from_bytes_le(&vecmap(&modulus, |b| b.to_u128() as u8));
                output_count = 0;

                let modulus_id = self.big_int_ctx.get_or_insert_modulus(big_modulus);
                let result_id = self.big_int_ctx.new_big_int(F::from(modulus_id as u128));
                (modulus, vec![result_id.bigint_id::<F>(), result_id.modulus_id::<F>()])
            }
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
                output_count = input_size + (16 - input_size % 16);
                (vec![], vec![])
            }
            BlackBoxFunc::RecursiveAggregation => {
                let proof_type_var = match inputs.pop() {
                    Some(domain_var) => domain_var.into_var()?,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::MissingArg {
                            name: "verify proof".to_string(),
                            arg: "proof type".to_string(),
                            call_stack: self.get_call_stack(),
                        }))
                    }
                };

                let proof_type_constant = match self.var_to_expression(proof_type_var)?.to_const() {
                    Some(proof_type_constant) => *proof_type_constant,
                    None => {
                        return Err(RuntimeError::InternalError(InternalError::NotAConstant {
                            name: "proof type".to_string(),
                            call_stack: self.get_call_stack(),
                        }))
                    }
                };

                (vec![proof_type_constant], Vec::new())
            }
            _ => (vec![], vec![]),
        };
        let inputs = self.prepare_inputs_for_black_box_func(inputs, name)?;
        // Call Black box with `FunctionInput`
        let mut results = vecmap(&constant_outputs, |c| self.add_constant(*c));

        let output_vars = vecmap(0..output_count, |_| self.add_variable());
        let output_witnesses = vecmap(&output_vars, |var| {
            self.var_to_witness(*var).expect("variable was just created as witness")
        });

        self.acir_ir.call_black_box(
            name,
            &inputs,
            constant_inputs,
            constant_outputs,
            output_witnesses,
        )?;

        // Convert `Witness` values which are now constrained to be the output of the
        // black box function call into `AcirVar`s.
        //
        // We do not apply range information on the output of the black box function.
        // See issue #1439
        results.extend(output_vars);
        Ok(results)
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
            inputs = self.all_or_nothing_for_ec_add(inputs)?;
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
                        single_val_witnesses.push(
                            FunctionInput::constant(*constant, num_bits).map_err(
                                |invalid_input_bit_size| {
                                    RuntimeError::InvalidBlackBoxInputBitSize {
                                        value: invalid_input_bit_size.value,
                                        num_bits: invalid_input_bit_size.value_num_bits,
                                        max_num_bits: invalid_input_bit_size.max_bits,
                                        call_stack: self.get_call_stack(),
                                    }
                                },
                            )?,
                        );
                    }
                    _ => {
                        let witness_var = self.get_or_create_witness_var(input)?;
                        let witness = self.var_to_witness(witness_var)?;
                        single_val_witnesses.push(FunctionInput::witness(witness, num_bits));
                    }
                }
            }
            witnesses.push(single_val_witnesses);
        }
        Ok(witnesses)
    }

    /// EcAdd has 6 inputs representing the two points to add
    /// Each point must be either all constant, or all witnesses
    fn all_or_nothing_for_ec_add(
        &mut self,
        inputs: Vec<Vec<FunctionInput<F>>>,
    ) -> Result<Vec<Vec<FunctionInput<F>>>, RuntimeError> {
        let mut has_constant = false;
        let mut has_witness = false;
        let mut result = inputs.clone();
        for (i, input) in inputs.iter().enumerate() {
            if input[0].is_constant() {
                has_constant = true;
            } else {
                has_witness = true;
            }
            if i % 3 == 2 {
                if has_constant && has_witness {
                    // Convert the constants to witness if mixed constant and witness,
                    for j in i - 2..i + 1 {
                        if let ConstantOrWitnessEnum::Constant(constant) = inputs[j][0].input() {
                            let constant = self.add_constant(constant);
                            let witness_var = self.get_or_create_witness_var(constant)?;
                            let witness = self.var_to_witness(witness_var)?;
                            result[j] =
                                vec![FunctionInput::witness(witness, inputs[j][0].num_bits())];
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
