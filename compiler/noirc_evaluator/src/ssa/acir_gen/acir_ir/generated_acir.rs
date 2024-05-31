//! `GeneratedAcir` is constructed as part of the `acir_gen` pass to accumulate all of the ACIR
//! program as it is being converted from SSA form.
use std::collections::BTreeMap;

use crate::{
    brillig::{brillig_gen::brillig_directive, brillig_ir::artifact::GeneratedBrillig},
    errors::{InternalError, RuntimeError, SsaReport},
    ssa::ir::dfg::CallStack,
};
use acvm::acir::{
    circuit::{
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
        AssertionPayload, OpcodeLocation,
    },
    native_types::Witness,
    BlackBoxFunc,
};
use acvm::{
    acir::AcirField,
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};
use iter_extended::vecmap;
use num_bigint::BigUint;

/// Brillig calls such as for the Brillig std lib are resolved only after code generation is finished.
/// This index should be used when adding a Brillig call during code generation.
/// Code generation should then keep track of that unresolved call opcode which will be resolved with the
/// correct function index after code generation.
pub(crate) const PLACEHOLDER_BRILLIG_INDEX: u32 = 0;

#[derive(Debug, Default)]
/// The output of the Acir-gen pass, which should only be produced for entry point Acir functions
pub(crate) struct GeneratedAcir {
    /// The next witness index that may be declared.
    /// If witness index is `None` then we have not yet created a witness
    /// and thus next witness index that be declared is zero.
    /// This field is private should only ever be accessed through its getter and setter.
    ///
    /// Equivalent to acvm::acir::circuit::Circuit's field of the same name.
    current_witness_index: Option<u32>,

    /// The opcodes of which the compiled ACIR will comprise.
    opcodes: Vec<AcirOpcode<FieldElement>>,

    /// All witness indices that comprise the final return value of the program
    pub(crate) return_witnesses: Vec<Witness>,

    /// All witness indices which are inputs to the main function
    pub(crate) input_witnesses: Vec<Witness>,

    /// Correspondence between an opcode index (in opcodes) and the source code call stack which generated it
    pub(crate) locations: BTreeMap<OpcodeLocation, CallStack>,

    /// Source code location of the current instruction being processed
    /// None if we do not know the location
    pub(crate) call_stack: CallStack,

    /// Correspondence between an opcode index and the error message associated with it.
    pub(crate) assertion_payloads: BTreeMap<OpcodeLocation, AssertionPayload<FieldElement>>,

    pub(crate) warnings: Vec<SsaReport>,

    /// Name for the corresponding entry point represented by this Acir-gen output.
    /// Only used for debugging and benchmarking purposes
    pub(crate) name: String,

    /// Maps the opcode index to a Brillig std library function call.
    /// As to avoid passing the ACIR gen shared context into each individual ACIR
    /// we can instead keep this map and resolve the Brillig calls at the end of code generation.
    pub(crate) brillig_stdlib_func_locations: BTreeMap<OpcodeLocation, BrilligStdlibFunc>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum BrilligStdlibFunc {
    Inverse,
    // The Brillig quotient code is different depending upon the bit size.
    Quotient(u32),
}

impl BrilligStdlibFunc {
    pub(crate) fn get_generated_brillig(&self) -> GeneratedBrillig {
        match self {
            BrilligStdlibFunc::Inverse => brillig_directive::directive_invert(),
            BrilligStdlibFunc::Quotient(bit_size) => {
                brillig_directive::directive_quotient(*bit_size)
            }
        }
    }
}

impl GeneratedAcir {
    /// Returns the current witness index.
    pub(crate) fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index.unwrap_or(0))
    }

    /// Adds a new opcode into ACIR.
    pub(crate) fn push_opcode(&mut self, opcode: AcirOpcode<FieldElement>) {
        self.opcodes.push(opcode);
        if !self.call_stack.is_empty() {
            self.locations.insert(self.last_acir_opcode_location(), self.call_stack.clone());
        }
    }

    pub(crate) fn opcodes(&self) -> &[AcirOpcode<FieldElement>] {
        &self.opcodes
    }

    pub(crate) fn take_opcodes(&mut self) -> Vec<AcirOpcode<FieldElement>> {
        std::mem::take(&mut self.opcodes)
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    pub(crate) fn next_witness_index(&mut self) -> Witness {
        if let Some(current_index) = self.current_witness_index {
            self.current_witness_index.replace(current_index + 1);
        } else {
            self.current_witness_index = Some(0);
        }
        Witness(self.current_witness_index.expect("ICE: current_witness_index should exist"))
    }

    /// Converts [`Expression`] `expr` into a [`Witness`].
    ///
    /// If `expr` can be represented as a `Witness` then this function will return it,
    /// else a new opcode will be added to create a `Witness` that is equal to `expr`.
    pub(crate) fn get_or_create_witness(&mut self, expr: &Expression<FieldElement>) -> Witness {
        match expr.to_witness() {
            Some(witness) => witness,
            None => self.create_witness_for_expression(expr),
        }
    }

    /// Creates a new [`Witness`] which is constrained to be equal to the passed [`Expression`].
    ///
    /// The reason we do this is because _constraints_ in ACIR have a degree limit
    /// This means you cannot multiply an infinite amount of `Expression`s together.
    /// Once the `Expression` goes over degree-2, then it needs to be reduced to a `Witness`
    /// which has degree-1 in order to be able to continue the multiplication chain.
    pub(crate) fn create_witness_for_expression(
        &mut self,
        expression: &Expression<FieldElement>,
    ) -> Witness {
        let fresh_witness = self.next_witness_index();

        // Create a constraint that sets them to be equal to each other
        // Then return the witness as this can now be used in places
        // where we would have used the `Expression`.
        let constraint = expression - fresh_witness;
        // This assertion means that verification of this
        // program will fail if expression != witness.
        //
        // This is because we have:
        //  => constraint == 0
        //  => expression - fresh_witness == 0
        //  => expression == fresh_witness
        self.assert_is_zero(constraint);

        fresh_witness
    }
}

impl GeneratedAcir {
    /// Calls a black box function and returns the output
    /// of said blackbox function.
    pub(crate) fn call_black_box(
        &mut self,
        func_name: BlackBoxFunc,
        inputs: &[Vec<FunctionInput>],
        constant_inputs: Vec<FieldElement>,
        constant_outputs: Vec<FieldElement>,
        output_count: usize,
    ) -> Result<Vec<Witness>, InternalError> {
        let input_count = inputs.iter().fold(0usize, |sum, val| sum + val.len());
        intrinsics_check_inputs(func_name, input_count);
        intrinsics_check_outputs(func_name, output_count);

        let outputs = vecmap(0..output_count, |_| self.next_witness_index());

        // clone is needed since outputs is moved when used in blackbox function.
        let outputs_clone = outputs.clone();

        let black_box_func_call = match func_name {
            BlackBoxFunc::AES128Encrypt => BlackBoxFuncCall::AES128Encrypt {
                inputs: inputs[0].clone(),
                iv: inputs[1]
                    .clone()
                    .try_into()
                    .expect("Compiler should generate correct size inputs"),
                key: inputs[2]
                    .clone()
                    .try_into()
                    .expect("Compiler should generate correct size inputs"),
                outputs,
            },
            BlackBoxFunc::AND => {
                BlackBoxFuncCall::AND { lhs: inputs[0][0], rhs: inputs[1][0], output: outputs[0] }
            }
            BlackBoxFunc::XOR => {
                BlackBoxFuncCall::XOR { lhs: inputs[0][0], rhs: inputs[1][0], output: outputs[0] }
            }
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: inputs[0][0] },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 {
                inputs: inputs[0].clone(),
                outputs: outputs.try_into().expect("Compiler should generate correct size outputs"),
            },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s {
                inputs: inputs[0].clone(),
                outputs: outputs.try_into().expect("Compiler should generate correct size outputs"),
            },
            BlackBoxFunc::Blake3 => BlackBoxFuncCall::Blake3 {
                inputs: inputs[0].clone(),
                outputs: outputs.try_into().expect("Compiler should generate correct size outputs"),
            },
            BlackBoxFunc::SchnorrVerify => {
                BlackBoxFuncCall::SchnorrVerify {
                    public_key_x: inputs[0][0],
                    public_key_y: inputs[1][0],
                    // Schnorr signature is an r & s, 32 bytes each
                    signature: inputs[2]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    message: inputs[3].clone(),
                    output: outputs[0],
                }
            }
            BlackBoxFunc::PedersenCommitment => BlackBoxFuncCall::PedersenCommitment {
                inputs: inputs[0].clone(),
                outputs: (outputs[0], outputs[1]),
                domain_separator: constant_inputs[0].to_u128() as u32,
            },
            BlackBoxFunc::PedersenHash => BlackBoxFuncCall::PedersenHash {
                inputs: inputs[0].clone(),
                output: outputs[0],
                domain_separator: constant_inputs[0].to_u128() as u32,
            },
            BlackBoxFunc::EcdsaSecp256k1 => {
                BlackBoxFuncCall::EcdsaSecp256k1 {
                    // 32 bytes for each public key co-ordinate
                    public_key_x: inputs[0]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    public_key_y: inputs[1]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    // (r,s) are both 32 bytes each, so signature
                    // takes up 64 bytes
                    signature: inputs[2]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    hashed_message: inputs[3]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    output: outputs[0],
                }
            }
            BlackBoxFunc::EcdsaSecp256r1 => {
                BlackBoxFuncCall::EcdsaSecp256r1 {
                    // 32 bytes for each public key co-ordinate
                    public_key_x: inputs[0]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    public_key_y: inputs[1]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    // (r,s) are both 32 bytes each, so signature
                    // takes up 64 bytes
                    signature: inputs[2]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    hashed_message: inputs[3]
                        .clone()
                        .try_into()
                        .expect("Compiler should generate correct size inputs"),
                    output: outputs[0],
                }
            }
            BlackBoxFunc::MultiScalarMul => BlackBoxFuncCall::MultiScalarMul {
                points: inputs[0].clone(),
                scalars: inputs[1].clone(),
                outputs: (outputs[0], outputs[1], outputs[2]),
            },

            BlackBoxFunc::EmbeddedCurveAdd => BlackBoxFuncCall::EmbeddedCurveAdd {
                input1: Box::new([inputs[0][0], inputs[1][0], inputs[2][0]]),
                input2: Box::new([inputs[3][0], inputs[4][0], inputs[5][0]]),
                outputs: (outputs[0], outputs[1], outputs[2]),
            },
            BlackBoxFunc::Keccak256 => {
                let var_message_size = match inputs.to_vec().pop() {
                    Some(var_message_size) => var_message_size[0],
                    None => {
                        return Err(InternalError::MissingArg {
                            name: "".to_string(),
                            arg: "message_size".to_string(),
                            call_stack: self.call_stack.clone(),
                        });
                    }
                };

                BlackBoxFuncCall::Keccak256 {
                    inputs: inputs[0].clone(),
                    var_message_size,
                    outputs: outputs
                        .try_into()
                        .expect("Compiler should generate correct size outputs"),
                }
            }
            BlackBoxFunc::Keccakf1600 => BlackBoxFuncCall::Keccakf1600 {
                inputs: inputs[0]
                    .clone()
                    .try_into()
                    .expect("Compiler should generate correct size inputs"),
                outputs: outputs.try_into().expect("Compiler should generate correct size outputs"),
            },
            BlackBoxFunc::RecursiveAggregation => BlackBoxFuncCall::RecursiveAggregation {
                verification_key: inputs[0].clone(),
                proof: inputs[1].clone(),
                public_inputs: inputs[2].clone(),
                key_hash: inputs[3][0],
            },
            BlackBoxFunc::BigIntAdd => BlackBoxFuncCall::BigIntAdd {
                lhs: constant_inputs[0].to_u128() as u32,
                rhs: constant_inputs[1].to_u128() as u32,
                output: constant_outputs[0].to_u128() as u32,
            },
            BlackBoxFunc::BigIntSub => BlackBoxFuncCall::BigIntSub {
                lhs: constant_inputs[0].to_u128() as u32,
                rhs: constant_inputs[1].to_u128() as u32,
                output: constant_outputs[0].to_u128() as u32,
            },
            BlackBoxFunc::BigIntMul => BlackBoxFuncCall::BigIntMul {
                lhs: constant_inputs[0].to_u128() as u32,
                rhs: constant_inputs[1].to_u128() as u32,
                output: constant_outputs[0].to_u128() as u32,
            },
            BlackBoxFunc::BigIntDiv => BlackBoxFuncCall::BigIntDiv {
                lhs: constant_inputs[0].to_u128() as u32,
                rhs: constant_inputs[1].to_u128() as u32,
                output: constant_outputs[0].to_u128() as u32,
            },
            BlackBoxFunc::BigIntFromLeBytes => BlackBoxFuncCall::BigIntFromLeBytes {
                inputs: inputs[0].clone(),
                modulus: vecmap(constant_inputs, |c| c.to_u128() as u8),
                output: constant_outputs[0].to_u128() as u32,
            },
            BlackBoxFunc::BigIntToLeBytes => BlackBoxFuncCall::BigIntToLeBytes {
                input: constant_inputs[0].to_u128() as u32,
                outputs,
            },
            BlackBoxFunc::Poseidon2Permutation => BlackBoxFuncCall::Poseidon2Permutation {
                inputs: inputs[0].clone(),
                outputs,
                len: constant_inputs[0].to_u128() as u32,
            },
            BlackBoxFunc::Sha256Compression => BlackBoxFuncCall::Sha256Compression {
                inputs: inputs[0]
                    .clone()
                    .try_into()
                    .expect("Compiler should generate correct size inputs"),
                hash_values: inputs[1]
                    .clone()
                    .try_into()
                    .expect("Compiler should generate correct size inputs"),
                outputs: outputs.try_into().expect("Compiler should generate correct size outputs"),
            },
        };

        self.push_opcode(AcirOpcode::BlackBoxFuncCall(black_box_func_call));

        Ok(outputs_clone)
    }

    /// Takes an input expression and returns witnesses that are constrained to be limbs
    /// decomposed from the input for the given radix and limb count.
    ///
    /// Only radix that are a power of two are supported
    pub(crate) fn radix_le_decompose(
        &mut self,
        input_expr: &Expression<FieldElement>,
        radix: u32,
        limb_count: u32,
        bit_size: u32,
    ) -> Result<Vec<Witness>, RuntimeError> {
        let radix_big = BigUint::from(radix);
        assert_eq!(
            BigUint::from(2u128).pow(bit_size),
            radix_big,
            "ICE: Radix must be a power of 2"
        );

        let limb_witnesses = vecmap(0..limb_count, |_| self.next_witness_index());
        self.push_opcode(AcirOpcode::Directive(Directive::ToLeRadix {
            a: input_expr.clone(),
            b: limb_witnesses.clone(),
            radix,
        }));

        let mut composed_limbs = Expression::default();

        let mut radix_pow = BigUint::from(1u128);
        for limb_witness in &limb_witnesses {
            self.range_constraint(*limb_witness, bit_size)?;

            composed_limbs = composed_limbs.add_mul(
                FieldElement::from_be_bytes_reduce(&radix_pow.to_bytes_be()),
                &Expression::from(*limb_witness),
            );

            radix_pow *= &radix_big;
        }

        self.assert_is_zero(input_expr - &composed_limbs);

        Ok(limb_witnesses)
    }

    /// Adds an inversion brillig opcode.
    ///
    /// This code will invert `expr` without applying constraints
    /// and return a `Witness` which may or may not be the result of
    /// inverting `expr`.
    ///
    /// Safety: It is the callers responsibility to ensure that the
    /// resulting `Witness` is constrained to be the inverse.
    pub(crate) fn brillig_inverse(&mut self, expr: Expression<FieldElement>) -> Witness {
        // Create the witness for the result
        let inverted_witness = self.next_witness_index();

        // Compute the inverse with brillig code
        let inverse_code = brillig_directive::directive_invert();
        let inputs = vec![BrilligInputs::Single(expr)];
        let outputs = vec![BrilligOutputs::Simple(inverted_witness)];
        self.brillig_call(
            Some(Expression::one()),
            &inverse_code,
            inputs,
            outputs,
            PLACEHOLDER_BRILLIG_INDEX,
            Some(BrilligStdlibFunc::Inverse),
        );

        inverted_witness
    }

    /// Asserts `expr` to be zero.
    ///
    /// If `expr` is not zero, then the constraint system will
    /// fail upon verification.
    pub(crate) fn assert_is_zero(&mut self, expr: Expression<FieldElement>) {
        self.push_opcode(AcirOpcode::AssertZero(expr));
    }

    /// Returns a `Witness` that is constrained to be:
    /// - `1` if `lhs == rhs`
    /// - `0` otherwise
    pub(crate) fn is_equal(
        &mut self,
        lhs: &Expression<FieldElement>,
        rhs: &Expression<FieldElement>,
    ) -> Witness {
        let t = lhs - rhs;

        self.is_zero(&t)
    }

    /// Returns a `Witness` that is constrained to be:
    /// - `1` if `t == 0`
    /// - `0` otherwise
    ///
    /// # Proof
    ///
    /// First, let's create a new variable `y` which will be the Witness that we will ultimately
    /// return indicating whether `t == 0`.
    /// Note: During this process we need to apply constraints that ensure that it is a boolean.
    /// But right now with no constraints applied to it, it is essentially a free variable.
    ///
    /// Next we apply the following constraint `y * t == 0`.
    /// This implies that either `y` or `t` or both is `0`.
    /// - If `t == 0`, then by definition `t == 0`.
    /// - If `y == 0`, this does not mean anything at this point in time, due to it having no
    /// constraints.
    ///
    /// Naively, we could apply the following constraint: `y == 1 - t`.
    /// This along with the previous `y * t == 0` constraint means that
    /// `y` or `t` needs to be zero, but they both cannot be zero.
    ///
    /// This equation however falls short when `t != 0` because then `t`
    /// may not be `1`. If `t` is non-zero, then `y` is also non-zero due to
    /// `y == 1 - t` and the equation `y * t == 0` fails.  
    ///
    /// To fix, we introduce another free variable called `z` and apply the following
    /// constraint instead: `y == 1 - t * z`.
    ///
    /// When `t == 0`, `y` is `1`.
    /// When `t != 0`, the prover can set `z = 1/t` which will make `y = 1 - t * 1/t = 0`.
    ///
    /// We now arrive at the conclusion that when `t == 0`, `y` is `1` and when
    /// `t != 0`, then `y` is `0`.
    ///  
    /// Bringing it all together, We introduce two variables `y` and `z`,
    /// With the following equations:
    /// - `y == 1 - tz` (`z` is a value that is chosen to be the inverse of `t` by the prover)
    /// - `y * t == 0`
    ///
    /// Lets convince ourselves that the prover cannot prove an untrue statement.
    ///
    /// ---
    /// Assume that `t == 0`, can the prover return `y == 0`?
    ///
    /// When `t == 0`, there is no way to make `y` be zero since `y = 1 - 0 * z = 1`.
    ///
    /// ---
    /// Assume that `t != 0`, can the prover return `y == 1`?
    ///
    /// By setting `z` to be `0`, we can make `y` equal to `1`.
    /// This is easily observed: `y = 1 - t * 0`
    /// Now since `y` is one, this means that `t` needs to be zero, or else `y * t == 0` will fail.
    fn is_zero(&mut self, t_expr: &Expression<FieldElement>) -> Witness {
        // We're checking for equality with zero so we can negate the expression without changing the result.
        // This is useful as it will sometimes allow us to simplify an expression down to a witness.
        let t_witness = if let Some(witness) = t_expr.to_witness() {
            witness
        } else {
            let negated_expr = t_expr * -FieldElement::one();
            self.get_or_create_witness(&negated_expr)
        };

        // Call the inversion directive, since we do not apply a constraint
        // the prover can choose anything here.
        let z = self.brillig_inverse(t_witness.into());

        let y = self.next_witness_index();

        // Add constraint y == 1 - tz => y + tz - 1 == 0
        let y_is_boolean_constraint = Expression {
            mul_terms: vec![(FieldElement::one(), t_witness, z)],
            linear_combinations: vec![(FieldElement::one(), y)],
            q_c: -FieldElement::one(),
        };
        self.assert_is_zero(y_is_boolean_constraint);

        // Add constraint that y * t == 0;
        let ty_zero_constraint = Expression {
            mul_terms: vec![(FieldElement::one(), t_witness, y)],
            linear_combinations: vec![],
            q_c: FieldElement::zero(),
        };
        self.assert_is_zero(ty_zero_constraint);

        y
    }

    /// Adds a constraint which ensure thats `witness` is an
    /// integer within the range `[0, 2^{num_bits} - 1]`
    pub(crate) fn range_constraint(
        &mut self,
        witness: Witness,
        num_bits: u32,
    ) -> Result<(), RuntimeError> {
        // We class this as an error because users should instead
        // do `as Field`.
        if num_bits >= FieldElement::max_num_bits() {
            return Err(RuntimeError::InvalidRangeConstraint {
                num_bits: FieldElement::max_num_bits(),
                call_stack: self.call_stack.clone(),
            });
        };

        let constraint = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness, num_bits },
        });
        self.push_opcode(constraint);

        Ok(())
    }

    pub(crate) fn brillig_call(
        &mut self,
        predicate: Option<Expression<FieldElement>>,
        generated_brillig: &GeneratedBrillig,
        inputs: Vec<BrilligInputs<FieldElement>>,
        outputs: Vec<BrilligOutputs>,
        brillig_function_index: u32,
        stdlib_func: Option<BrilligStdlibFunc>,
    ) {
        let opcode =
            AcirOpcode::BrilligCall { id: brillig_function_index, inputs, outputs, predicate };
        self.push_opcode(opcode);
        if let Some(stdlib_func) = stdlib_func {
            self.brillig_stdlib_func_locations
                .insert(self.last_acir_opcode_location(), stdlib_func);
        }

        for (brillig_index, call_stack) in generated_brillig.locations.iter() {
            self.locations.insert(
                OpcodeLocation::Brillig {
                    acir_index: self.opcodes.len() - 1,
                    brillig_index: *brillig_index,
                },
                call_stack.clone(),
            );
        }
        for (brillig_index, message) in generated_brillig.assert_messages.iter() {
            self.assertion_payloads.insert(
                OpcodeLocation::Brillig {
                    acir_index: self.opcodes.len() - 1,
                    brillig_index: *brillig_index,
                },
                AssertionPayload::StaticString(message.clone()),
            );
        }
    }

    // We can only resolve the Brillig stdlib after having processed the entire ACIR
    pub(crate) fn resolve_brillig_stdlib_call(
        &mut self,
        opcode_location: OpcodeLocation,
        brillig_function_index: u32,
    ) {
        let acir_index = match opcode_location {
            OpcodeLocation::Acir(index) => index,
            _ => panic!("should not have brillig index"),
        };
        match &mut self.opcodes[acir_index] {
            AcirOpcode::BrilligCall { id, .. } => *id = brillig_function_index,
            _ => panic!("expected brillig call opcode"),
        }
    }

    pub(crate) fn last_acir_opcode_location(&self) -> OpcodeLocation {
        OpcodeLocation::Acir(self.opcodes.len() - 1)
    }
}

/// This function will return the number of inputs that a blackbox function
/// expects. Returning `None` if there is no expectation.
fn black_box_func_expected_input_size(name: BlackBoxFunc) -> Option<usize> {
    match name {
        // Bitwise opcodes will take in 2 parameters
        BlackBoxFunc::AND | BlackBoxFunc::XOR => Some(2),

        // All of the hash/cipher methods will take in a
        // variable number of inputs.
        BlackBoxFunc::AES128Encrypt
        | BlackBoxFunc::Keccak256
        | BlackBoxFunc::SHA256
        | BlackBoxFunc::Blake2s
        | BlackBoxFunc::Blake3
        | BlackBoxFunc::PedersenCommitment
        | BlackBoxFunc::PedersenHash => None,

        BlackBoxFunc::Keccakf1600 => Some(25),
        // The permutation takes a fixed number of inputs, but the inputs length depends on the proving system implementation.
        BlackBoxFunc::Poseidon2Permutation => None,

        // SHA256 compression requires 16 u32s as input message and 8 u32s for the hash state.
        BlackBoxFunc::Sha256Compression => Some(24),
        // Can only apply a range constraint to one
        // witness at a time.
        BlackBoxFunc::RANGE => Some(1),

        // Signature verification algorithms will take in a variable
        // number of inputs, since the message/hashed-message can vary in size.
        BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::EcdsaSecp256r1 => None,

        // Inputs for multi scalar multiplication is an arbitrary number of [point, scalar] pairs.
        BlackBoxFunc::MultiScalarMul => None,

        // Recursive aggregation has a variable number of inputs
        BlackBoxFunc::RecursiveAggregation => None,

        // Addition over the embedded curve: input are coordinates (x1,y1,infinite1) and (x2,y2,infinite2) of the Grumpkin points
        BlackBoxFunc::EmbeddedCurveAdd => Some(6),

        // Big integer operations take in 0 inputs. They use constants for their inputs.
        BlackBoxFunc::BigIntAdd
        | BlackBoxFunc::BigIntSub
        | BlackBoxFunc::BigIntMul
        | BlackBoxFunc::BigIntDiv
        | BlackBoxFunc::BigIntToLeBytes => Some(0),

        // FromLeBytes takes a variable array of bytes as input
        BlackBoxFunc::BigIntFromLeBytes => None,
    }
}

/// This function will return the number of outputs that a blackbox function
/// expects. Returning `None` if there is no expectation.
fn black_box_expected_output_size(name: BlackBoxFunc) -> Option<usize> {
    match name {
        // Bitwise opcodes will return 1 parameter which is the output
        // or the operation.
        BlackBoxFunc::AND | BlackBoxFunc::XOR => Some(1),

        // 32 byte hash algorithms
        BlackBoxFunc::Keccak256
        | BlackBoxFunc::SHA256
        | BlackBoxFunc::Blake2s
        | BlackBoxFunc::Blake3 => Some(32),

        BlackBoxFunc::Keccakf1600 => Some(25),
        // The permutation returns a fixed number of outputs, equals to the inputs length which depends on the proving system implementation.
        BlackBoxFunc::Poseidon2Permutation => None,

        BlackBoxFunc::Sha256Compression => Some(8),
        // Pedersen commitment returns a point
        BlackBoxFunc::PedersenCommitment => Some(2),

        // Pedersen hash returns a field
        BlackBoxFunc::PedersenHash => Some(1),

        // Can only apply a range constraint to one
        // witness at a time.
        BlackBoxFunc::RANGE => Some(0),

        // Signature verification algorithms will return a boolean
        BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::EcdsaSecp256r1 => Some(1),

        // Output of operations over the embedded curve
        // will be 2 field elements representing the point.
        BlackBoxFunc::MultiScalarMul | BlackBoxFunc::EmbeddedCurveAdd => Some(3),

        // Big integer operations return a big integer
        BlackBoxFunc::BigIntAdd
        | BlackBoxFunc::BigIntSub
        | BlackBoxFunc::BigIntMul
        | BlackBoxFunc::BigIntDiv
        | BlackBoxFunc::BigIntFromLeBytes => Some(0),

        // ToLeBytes returns a variable array of bytes
        BlackBoxFunc::BigIntToLeBytes => None,

        // Recursive aggregation has a variable number of outputs
        BlackBoxFunc::RecursiveAggregation => None,

        // AES encryption returns a variable number of outputs
        BlackBoxFunc::AES128Encrypt => None,
    }
}

/// Checks that the number of inputs being used to call the blackbox function
/// is correct according to the function definition.
///
/// Some functions expect a variable number of inputs and in such a case,
/// this method will do nothing.  An example of this is sha256.
/// In that case, this function will not check anything.
///
/// Since we expect black box functions to be called behind a Noir shim function,
/// we trigger a compiler error if the inputs do not match.
///
/// An example of Noir shim function is the following:
/// ``
/// #[foreign(sha256)]
/// fn sha256<N>(_input : [u8; N]) -> [u8; 32] {}
/// ``
fn intrinsics_check_inputs(name: BlackBoxFunc, input_count: usize) {
    let expected_num_inputs = match black_box_func_expected_input_size(name) {
        Some(expected_num_inputs) => expected_num_inputs,
        None => return,
    };

    assert_eq!(expected_num_inputs,input_count,"Tried to call black box function {name} with {input_count} inputs, but this function's definition requires {expected_num_inputs} inputs");
}

/// Checks that the number of outputs being used to call the blackbox function
/// is correct according to the function definition.
///
/// Some functions expect a variable number of outputs and in such a case,
/// this method will do nothing.  An example of this is recursive aggregation.
/// In that case, this function will not check anything.
///
/// Since we expect black box functions to be called behind a Noir shim function,
/// we trigger a compiler error if the inputs do not match.
///
/// An example of Noir shim function is the following:
/// ``
/// #[foreign(sha256)]
/// fn verify_proof<N>(
///     _verification_key : [Field],
///     _proof : [Field],
///     _public_inputs : [Field],
///     _key_hash : Field,
///     _input_aggregation_object : [Field; N]
/// ) -> [Field; N] {}
/// ``
fn intrinsics_check_outputs(name: BlackBoxFunc, output_count: usize) {
    let expected_num_outputs = match black_box_expected_output_size(name) {
        Some(expected_num_inputs) => expected_num_inputs,
        None => return,
    };

    assert_eq!(expected_num_outputs,output_count,"Tried to call black box function {name} with {output_count} outputs, but this function's definition requires {expected_num_outputs} outputs");
}
