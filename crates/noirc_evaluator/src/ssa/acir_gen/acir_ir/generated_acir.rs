//! `GeneratedAcir` is constructed as part of the `acir_gen` pass to accumulate all of the ACIR
//! program as it is being converted from SSA form.
use std::collections::BTreeMap;

use crate::{
    brillig::{brillig_gen::brillig_directive, brillig_ir::artifact::GeneratedBrillig},
    errors::{InternalError, RuntimeError},
    ssa::ir::dfg::CallStack,
};

use acvm::acir::{
    circuit::{
        brillig::{Brillig as AcvmBrillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
        OpcodeLocation,
    },
    native_types::Witness,
    BlackBoxFunc,
};
use acvm::{
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};
use iter_extended::vecmap;
use num_bigint::BigUint;

#[derive(Debug, Default)]
/// The output of the Acir-gen pass
pub(crate) struct GeneratedAcir {
    /// The next witness index that may be declared.
    ///
    /// Equivalent to acvm::acir::circuit::Circuit's field of the same name.
    pub(crate) current_witness_index: u32,

    /// The opcodes of which the compiled ACIR will comprise.
    opcodes: Vec<AcirOpcode>,

    /// All witness indices that comprise the final return value of the program
    ///
    /// Note: This may contain repeated indices, which is necessary for later mapping into the
    /// abi's return type.
    pub(crate) return_witnesses: Vec<Witness>,

    /// All witness indices which are inputs to the main function
    pub(crate) input_witnesses: Vec<Witness>,

    /// Correspondance between an opcode index (in opcodes) and the source code call stack which generated it
    pub(crate) locations: BTreeMap<OpcodeLocation, CallStack>,

    /// Source code location of the current instruction being processed
    /// None if we do not know the location
    pub(crate) call_stack: CallStack,
}

impl GeneratedAcir {
    /// Returns the current witness index.
    pub(crate) fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    /// Adds a new opcode into ACIR.
    pub(crate) fn push_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode);
        if !self.call_stack.is_empty() {
            self.locations
                .insert(OpcodeLocation::Acir(self.opcodes.len() - 1), self.call_stack.clone());
        }
    }

    pub(crate) fn take_opcodes(&mut self) -> Vec<AcirOpcode> {
        std::mem::take(&mut self.opcodes)
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    pub(crate) fn next_witness_index(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    /// Converts [`Expression`] `expr` into a [`Witness`].
    ///
    /// If `expr` can be represented as a `Witness` then this function will return it,
    /// else a new opcode will be added to create a `Witness` that is equal to `expr`.
    pub(crate) fn get_or_create_witness(&mut self, expr: &Expression) -> Witness {
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
    pub(crate) fn create_witness_for_expression(&mut self, expression: &Expression) -> Witness {
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

    /// Adds a witness index to the program's return witnesses.
    pub(crate) fn push_return_witness(&mut self, witness: Witness) {
        self.return_witnesses.push(witness);
    }
}

impl GeneratedAcir {
    /// Calls a black box function and returns the output
    /// of said blackbox function.
    pub(crate) fn call_black_box(
        &mut self,
        func_name: BlackBoxFunc,
        inputs: &[Vec<FunctionInput>],
        constants: Vec<FieldElement>,
        output_count: usize,
    ) -> Result<Vec<Witness>, InternalError> {
        let input_count = inputs.iter().fold(0usize, |sum, val| sum + val.len());
        intrinsics_check_inputs(func_name, input_count);
        intrinsics_check_outputs(func_name, output_count);

        let outputs = vecmap(0..output_count, |_| self.next_witness_index());

        // clone is needed since outputs is moved when used in blackbox function.
        let outputs_clone = outputs.clone();

        let black_box_func_call = match func_name {
            BlackBoxFunc::AND => {
                BlackBoxFuncCall::AND { lhs: inputs[0][0], rhs: inputs[1][0], output: outputs[0] }
            }
            BlackBoxFunc::XOR => {
                BlackBoxFuncCall::XOR { lhs: inputs[0][0], rhs: inputs[1][0], output: outputs[0] }
            }
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: inputs[0][0] },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs: inputs[0].clone(), outputs },
            BlackBoxFunc::Blake2s => {
                BlackBoxFuncCall::Blake2s { inputs: inputs[0].clone(), outputs }
            }
            BlackBoxFunc::HashToField128Security => BlackBoxFuncCall::HashToField128Security {
                inputs: inputs[0].clone(),
                output: outputs[0],
            },
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: inputs[0][0],
                public_key_y: inputs[1][0],
                // Schnorr signature is an r & s, 32 bytes each
                signature: inputs[2].clone(),
                message: inputs[3].clone(),
                output: outputs[0],
            },
            BlackBoxFunc::Pedersen => BlackBoxFuncCall::Pedersen {
                inputs: inputs[0].clone(),
                outputs: (outputs[0], outputs[1]),
                domain_separator: constants[0].to_u128() as u32,
            },
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                // 32 bytes for each public key co-ordinate
                public_key_x: inputs[0].clone(),
                public_key_y: inputs[1].clone(),
                // (r,s) are both 32 bytes each, so signature
                // takes up 64 bytes
                signature: inputs[2].clone(),
                hashed_message: inputs[3].clone(),
                output: outputs[0],
            },
            BlackBoxFunc::EcdsaSecp256r1 => BlackBoxFuncCall::EcdsaSecp256r1 {
                // 32 bytes for each public key co-ordinate
                public_key_x: inputs[0].clone(),
                public_key_y: inputs[1].clone(),
                // (r,s) are both 32 bytes each, so signature
                // takes up 64 bytes
                signature: inputs[2].clone(),
                hashed_message: inputs[3].clone(),
                output: outputs[0],
            },
            BlackBoxFunc::FixedBaseScalarMul => BlackBoxFuncCall::FixedBaseScalarMul {
                input: inputs[0][0],
                outputs: (outputs[0], outputs[1]),
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
                BlackBoxFuncCall::Keccak256VariableLength {
                    inputs: inputs[0].clone(),
                    var_message_size,
                    outputs,
                }
            }
            BlackBoxFunc::RecursiveAggregation => {
                let has_previous_aggregation = self.opcodes.iter().any(|op| {
                    matches!(
                        op,
                        AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::RecursiveAggregation { .. })
                    )
                });

                let input_aggregation_object =
                    if !has_previous_aggregation { None } else { Some(inputs[4].clone()) };

                BlackBoxFuncCall::RecursiveAggregation {
                    verification_key: inputs[0].clone(),
                    proof: inputs[1].clone(),
                    public_inputs: inputs[2].clone(),
                    key_hash: inputs[3][0],
                    input_aggregation_object,
                    output_aggregation_object: outputs,
                }
            }
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
        input_expr: &Expression,
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

    // Returns the 2-complement of lhs, using the provided sign bit in 'leading'
    // if leading is zero, it returns lhs
    // if leading is one, it returns 2^bit_size-lhs
    fn two_complement(
        &mut self,
        lhs: &Expression,
        leading: Witness,
        max_bit_size: u32,
    ) -> Expression {
        let max_power_of_two =
            FieldElement::from(2_i128).pow(&FieldElement::from(max_bit_size as i128 - 1));

        let intermediate =
            self.mul_with_witness(&(&Expression::from(max_power_of_two) - lhs), &leading.into());

        lhs.add_mul(FieldElement::from(2_i128), &intermediate)
    }

    /// Returns an expression which represents `lhs * rhs`
    ///
    /// If one has multiplicative term and the other is of degree one or more,
    /// the function creates [intermediate variables][`Witness`] accordingly.
    /// There are two cases where we can optimize the multiplication between two expressions:
    /// 1. If the sum of the degrees of both expressions is at most 2, then we can just multiply them
    /// as each term in the result will be degree-2.
    /// 2. If one expression is a constant, then we can just multiply the constant with the other expression
    ///
    /// (1) is because an [`Expression`] can hold at most a degree-2 univariate polynomial
    /// which is what you get when you multiply two degree-1 univariate polynomials.
    pub(crate) fn mul_with_witness(&mut self, lhs: &Expression, rhs: &Expression) -> Expression {
        use std::borrow::Cow;
        let lhs_is_linear = lhs.is_linear();
        let rhs_is_linear = rhs.is_linear();

        // Case 1: The sum of the degrees of both expressions is at most 2.
        //
        // If one of the expressions is constant then it does not increase the degree when multiplying by another expression.
        // If both of the expressions are linear (degree <=1) then the product will be at most degree 2.
        let both_are_linear = lhs_is_linear && rhs_is_linear;
        let either_is_const = lhs.is_const() || rhs.is_const();
        if both_are_linear || either_is_const {
            return (lhs * rhs).expect("Both expressions are degree <= 1");
        }

        // Case 2: One or both of the sides needs to be reduced to a degree-1 univariate polynomial
        let lhs_reduced = if lhs_is_linear {
            Cow::Borrowed(lhs)
        } else {
            Cow::Owned(self.get_or_create_witness(lhs).into())
        };

        // If the lhs and rhs are the same, then we do not need to reduce
        // rhs, we only need to square the lhs.
        if lhs == rhs {
            return (&*lhs_reduced * &*lhs_reduced)
                .expect("Both expressions are reduced to be degree <= 1");
        };

        let rhs_reduced = if rhs_is_linear {
            Cow::Borrowed(rhs)
        } else {
            Cow::Owned(self.get_or_create_witness(rhs).into())
        };

        (&*lhs_reduced * &*rhs_reduced).expect("Both expressions are reduced to be degree <= 1")
    }

    /// Signed division lhs /  rhs
    /// We derive the signed division from the unsigned euclidian division.
    /// note that this is not euclidian division!
    // if x is a signed integer, then sign(x)x >= 0
    // so if a and b are signed integers, we can do the unsigned division:
    // sign(a)a = q1*sign(b)b + r1
    // => a = sign(a)sign(b)q1*b + sign(a)r1
    // => a = qb+r, with |r|<|b| and a and r have the same sign.
    pub(crate) fn signed_division(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        max_bit_size: u32,
    ) -> Result<(Expression, Expression), RuntimeError> {
        // 2^{max_bit size-1}
        let max_power_of_two =
            FieldElement::from(2_i128).pow(&FieldElement::from(max_bit_size as i128 - 1));

        // Get the sign bit of rhs by computing rhs / max_power_of_two
        let (rhs_leading_witness, _) = self.euclidean_division(
            rhs,
            &max_power_of_two.into(),
            max_bit_size,
            &Expression::one(),
        )?;

        // Get the sign bit of lhs by computing lhs / max_power_of_two
        let (lhs_leading_witness, _) = self.euclidean_division(
            lhs,
            &max_power_of_two.into(),
            max_bit_size,
            &Expression::one(),
        )?;

        // Signed to unsigned:
        let unsigned_lhs = self.two_complement(lhs, lhs_leading_witness, max_bit_size);
        let unsigned_rhs = self.two_complement(rhs, rhs_leading_witness, max_bit_size);
        let unsigned_l_witness = self.get_or_create_witness(&unsigned_lhs);
        let unsigned_r_witness = self.get_or_create_witness(&unsigned_rhs);

        // Performs the division using the unsigned values of lhs and rhs
        let (q1, r1) = self.euclidean_division(
            &unsigned_l_witness.into(),
            &unsigned_r_witness.into(),
            max_bit_size - 1,
            &Expression::one(),
        )?;

        // Unsigned to signed: derive q and r from q1,r1 and the signs of lhs and rhs
        // Quotient sign is lhs sign * rhs sign, whose resulting sign bit is the XOR of the sign bits
        let sign_sum =
            &Expression::from(lhs_leading_witness) + &Expression::from(rhs_leading_witness);
        let sign_prod = (&Expression::from(lhs_leading_witness)
            * &Expression::from(rhs_leading_witness))
            .expect("Product of two witnesses so result is degree 2");
        let q_sign = sign_sum.add_mul(-FieldElement::from(2_i128), &sign_prod);

        let q_sign_witness = self.get_or_create_witness(&q_sign);
        let quotient = self.two_complement(&q1.into(), q_sign_witness, max_bit_size);
        let remainder = self.two_complement(&r1.into(), lhs_leading_witness, max_bit_size);
        Ok((quotient, remainder))
    }

    /// Computes lhs/rhs by using euclidean division.
    ///
    /// Returns `q` for quotient and `r` for remainder such
    /// that lhs = rhs * q + r
    pub(crate) fn euclidean_division(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        max_bit_size: u32,
        predicate: &Expression,
    ) -> Result<(Witness, Witness), RuntimeError> {
        // lhs = rhs * q + r
        //
        // If predicate is zero, `q_witness` and `r_witness` will be 0

        // maximum bit size for q and for [r and rhs]
        let mut max_q_bits = max_bit_size;
        let mut max_rhs_bits = max_bit_size;
        // when rhs is constant, we can better estimate the maximum bit sizes
        if let Some(rhs_const) = rhs.to_const() {
            max_rhs_bits = rhs_const.num_bits();
            if max_rhs_bits != 0 {
                max_q_bits = max_bit_size - max_rhs_bits + 1;
            }
        }

        let (q_witness, r_witness) =
            self.brillig_quotient(lhs.clone(), rhs.clone(), predicate.clone(), max_bit_size + 1);

        // Apply range constraints to injected witness values.
        // Constrains `q` to be 0 <= q < 2^{q_max_bits}, etc.
        self.range_constraint(q_witness, max_q_bits)?;
        self.range_constraint(r_witness, max_rhs_bits)?;

        // Constrain r < rhs
        self.bound_constraint_with_offset(&r_witness.into(), rhs, predicate, max_rhs_bits)?;

        // a * predicate == (b * q + r) * predicate
        // => predicate * (a - b * q - r) == 0
        // When the predicate is 0, the equation always passes.
        // When the predicate is 1, the euclidean division needs to be
        // true.
        let rhs_constraint = &self.mul_with_witness(rhs, &q_witness.into()) + r_witness;
        let div_euclidean = &self.mul_with_witness(lhs, predicate)
            - &self.mul_with_witness(&rhs_constraint, predicate);

        self.push_opcode(AcirOpcode::Arithmetic(div_euclidean));

        Ok((q_witness, r_witness))
    }

    /// Adds a brillig opcode which injects witnesses with values `q = a / b` and `r = a % b`.
    ///
    /// Suitable range constraints for `q` and `r` must be applied externally.
    pub(crate) fn brillig_quotient(
        &mut self,
        lhs: Expression,
        rhs: Expression,
        predicate: Expression,
        max_bit_size: u32,
    ) -> (Witness, Witness) {
        // Create the witness for the result
        let q_witness = self.next_witness_index();
        let r_witness = self.next_witness_index();

        let quotient_code = brillig_directive::directive_quotient(max_bit_size);
        let inputs = vec![
            BrilligInputs::Single(lhs),
            BrilligInputs::Single(rhs),
            BrilligInputs::Single(predicate.clone()),
        ];
        let outputs = vec![BrilligOutputs::Simple(q_witness), BrilligOutputs::Simple(r_witness)];
        self.brillig(Some(predicate), quotient_code, inputs, outputs);

        (q_witness, r_witness)
    }

    /// Generate constraints that are satisfied iff
    /// lhs < rhs , when offset is 1, or
    /// lhs <= rhs, when offset is 0
    /// bits is the bit size of a and b (or an upper bound of the bit size)
    ///
    /// lhs<=rhs is done by constraining b-a to a bit size of 'bits':
    /// if lhs<=rhs, 0 <= rhs-lhs <= b < 2^bits
    /// if lhs>rhs, rhs-lhs = p+rhs-lhs > p-2^bits >= 2^bits  (if log(p) >= bits + 1)
    /// n.b: we do NOT check here that lhs and rhs are indeed 'bits' size
    /// lhs < rhs <=> a+1<=b
    /// TODO: Consolidate this with bounds_check function.
    fn bound_constraint_with_offset(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        offset: &Expression,
        bits: u32,
    ) -> Result<(), RuntimeError> {
        const fn num_bits<T>() -> usize {
            std::mem::size_of::<T>() * 8
        }

        fn bit_size_u128(a: u128) -> u32 where {
            num_bits::<u128>() as u32 - a.leading_zeros()
        }

        fn bit_size_u32(a: u32) -> u32 where {
            num_bits::<u32>() as u32 - a.leading_zeros()
        }

        assert!(
            bits < FieldElement::max_num_bits(),
            "range check with bit size of the prime field is not implemented yet"
        );

        let mut lhs_offset = lhs + offset;

        // Optimization when rhs is const and fits within a u128
        if rhs.is_const() && rhs.q_c.fits_in_u128() {
            // We try to move the offset to rhs
            let rhs_offset = if *offset == Expression::one() && rhs.q_c.to_u128() >= 1 {
                lhs_offset = lhs.clone();
                rhs.q_c.to_u128() - 1
            } else {
                rhs.q_c.to_u128()
            };
            // we now have lhs+offset <= rhs <=> lhs_offset <= rhs_offset

            let bit_size = bit_size_u128(rhs_offset);
            // r = 2^bit_size - rhs_offset
            let r = (1_u128 << bit_size) - rhs_offset - 1;
            // witness = lhs_offset + r
            assert!(bits + bit_size < FieldElement::max_num_bits()); //we need to ensure lhs_offset + r does not overflow
            let mut aor = lhs_offset;
            aor.q_c += FieldElement::from(r);
            let witness = self.get_or_create_witness(&aor);
            // lhs_offset<=rhs_offset <=> lhs_offset + r < rhs_offset + r = 2^bit_size <=> witness < 2^bit_size
            self.range_constraint(witness, bit_size)?;
            return Ok(());
        }

        // General case:  lhs_offset<=rhs <=> rhs-lhs_offset>=0 <=> rhs-lhs_offset is a 'bits' bit integer
        let sub_expression = rhs - &lhs_offset; //rhs-lhs_offset
        let w = self.create_witness_for_expression(&sub_expression);
        self.range_constraint(w, bits)?;

        Ok(())
    }

    /// Computes the expression x(x-1)
    ///
    /// If the above is constrained to zero, then it can only be
    /// true, iff x equals zero or one.
    fn boolean_expr(&mut self, expr: &Expression) -> Expression {
        let expr_as_witness = self.create_witness_for_expression(expr);
        let mut expr_squared = Expression::default();
        expr_squared.push_multiplication_term(
            FieldElement::one(),
            expr_as_witness,
            expr_as_witness,
        );
        &expr_squared - expr
    }

    /// Adds an inversion brillig opcode.
    ///
    /// This code will invert `expr` without applying constraints
    /// and return a `Witness` which may or may not be the result of
    /// inverting `expr`.
    ///
    /// Safety: It is the callers responsibility to ensure that the
    /// resulting `Witness` is constrained to be the inverse.
    pub(crate) fn brillig_inverse(&mut self, expr: Expression) -> Witness {
        // Create the witness for the result
        let inverted_witness = self.next_witness_index();

        // Compute the inverse with brillig code
        let inverse_code = brillig_directive::directive_invert();
        let inputs = vec![BrilligInputs::Single(expr)];
        let outputs = vec![BrilligOutputs::Simple(inverted_witness)];
        self.brillig(Some(Expression::one()), inverse_code, inputs, outputs);

        inverted_witness
    }

    /// Asserts `expr` to be zero.
    ///
    /// If `expr` is not zero, then the constraint system will
    /// fail upon verification.
    pub(crate) fn assert_is_zero(&mut self, expr: Expression) {
        self.push_opcode(AcirOpcode::Arithmetic(expr));
    }

    /// Returns a `Witness` that is constrained to be:
    /// - `1` if `lhs == rhs`
    /// - `0` otherwise
    ///
    /// Intuition: the equality of two Expressions is linked to whether
    /// their difference has an inverse; `a == b` implies that `a - b == 0`
    /// which implies that a - b has no inverse. So if two variables are equal,
    /// their difference will have no inverse.
    ///
    /// First, lets create a new variable that is equal to the difference
    /// of the two expressions: `t = lhs - rhs` (constraint has been applied)
    ///
    /// Next lets create a new variable `y` which will be the Witness that we will ultimately
    /// return indicating whether `lhs == rhs`.
    /// Note: During this process we need to apply constraints that ensure that it is a boolean.
    /// But right now with no constraints applied to it, it is essentially a free variable.
    ///
    /// Next we apply the following constraint `y * t == 0`.
    /// This implies that either `y` or `t` or both is `0`.
    /// - If `t == 0`, then this means that `lhs == rhs`.
    /// - If `y == 0`, this does not mean anything at this point in time, due to it having no
    /// constraints.
    ///
    /// Naively, we could apply the following constraint: `y == 1 - t`.
    /// This along with the previous `y * t == 0` constraint means that
    /// `y` or `t` needs to be zero, but they both cannot be zero.
    ///
    /// This equation however falls short when lhs != rhs because then `t`
    /// may not be `1`. If `t` is non-zero, then `y` is also non-zero due to
    /// `y == 1 - t` and the equation `y * t == 0` fails.  
    ///
    /// To fix, we introduce another free variable called `z` and apply the following
    /// constraint instead: `y == 1 - t * z`.
    ///
    /// When `lhs == rhs`, `t` is `0` and so `y` is `1`.
    /// When `lhs != rhs`, `t` is non-zero, however the prover can set `z = 1/t`
    /// which will make `y = 1 - t * 1/t = 0`.
    ///
    /// We now arrive at the conclusion that when `lhs == rhs`, `y` is `1` and when
    /// `lhs != rhs`, then `y` is `0`.
    ///  
    /// Bringing it all together, We introduce three variables `y`, `t` and `z`,
    /// With the following equations:
    /// - `t == lhs - rhs`
    /// - `y == 1 - tz` (`z` is a value that is chosen to be the inverse of `t` by the prover)
    /// - `y * t == 0`
    ///
    /// Lets convince ourselves that the prover cannot prove an untrue statement.
    ///
    /// Assume that `lhs == rhs`, can the prover return `y == 0`?
    ///
    /// When `lhs == rhs`, `t` is 0. There is no way to make `y` be zero
    /// since `y = 1 - 0 * z = 1`.
    ///
    /// Assume that `lhs != rhs`, can the prover return `y == 1`?
    ///
    /// When `lhs != rhs`, then `t` is non-zero.
    /// By setting `z` to be `0`, we can make `y` equal to `1`.
    /// This is easily observed: `y = 1 - t * 0`
    /// Now since `y` is one, this means that `t` needs to be zero, or else `y * t == 0` will fail.
    pub(crate) fn is_equal(&mut self, lhs: &Expression, rhs: &Expression) -> Witness {
        let t = lhs - rhs;
        // We avoid passing the expression to `self.brillig_inverse` directly because we need
        // the `Witness` representation for constructing `y_is_boolean_constraint`.
        let t_witness = self.get_or_create_witness(&t);

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

    /// Returns a `Witness` that is constrained to be:
    /// - `1` if lhs >= rhs
    /// - `0` otherwise
    ///
    /// We essentially computes the sign bit of `b-a`
    /// For this we sign-extend `b-a` with `c = 2^{max_bits} - (b - a)`, since both `a` and `b` are less than `2^{max_bits}`
    /// Then we get the bit sign of `c`, the 2-complement representation of `(b-a)`, which is a `max_bits+1` integer,
    /// by doing the euclidean division `c / 2^{max_bits}`
    ///
    /// To see why it really works;
    /// We first note that `c` is an integer of `(max_bits+1)` bits. Therefore,
    /// if `b-a>0`, then `c < 2^{max_bits}`, so the division by `2^{max_bits}` will give `0`
    /// If `b-a<=0`, then `c >= 2^{max_bits}`, so the division by `2^{max_bits}` will give `1`.
    ///
    /// In other words, `1` means `a >= b` and `0` means `b > a`.
    /// The important thing here is that `c` does not overflow nor underflow the field;
    /// - By construction we have `c >= 0`, so there is no underflow
    /// - We assert at the beginning that `2^{max_bits+1}` does not overflow the field, so neither c.
    pub(crate) fn more_than_eq_comparison(
        &mut self,
        a: &Expression,
        b: &Expression,
        max_bits: u32,
        predicate: Expression,
    ) -> Result<Witness, RuntimeError> {
        // Ensure that 2^{max_bits + 1} is less than the field size
        //
        // TODO: perhaps this should be a user error, instead of an assert
        assert!(max_bits + 1 < FieldElement::max_num_bits());

        // Compute : 2^{max_bits} + a - b
        let two = FieldElement::from(2_i128);
        let two_max_bits: FieldElement = two.pow(&FieldElement::from(max_bits as i128));
        let comparison_evaluation = (a - b) + two_max_bits;

        // Euclidian division by 2^{max_bits}  : 2^{max_bits} + a - b = q * 2^{max_bits} + r
        //
        // 2^{max_bits} is of max_bits+1 bit size
        // If a>b, then a-b is less than 2^{max_bits} - 1, so 2^{max_bits} + a - b is less than 2^{max_bits} + 2^{max_bits} - 1 = 2^{max_bits+1} - 1
        // If a <= b, then 2^{max_bits} + a - b is less than 2^{max_bits} <= 2^{max_bits+1} - 1
        // This means that both operands of the division have at most max_bits+1 bit size.
        //
        // case: a == b
        //
        //   let k = 0;
        // - 2^{max_bits} == q *  2^{max_bits} + r
        // - This is only the case when q == 1 and r == 0 (assuming r is bounded to be less than 2^{max_bits})
        //
        // case: a > b
        //
        //   let k = a - b;
        // - k + 2^{max_bits} == q * 2^{max_bits} + r
        // - This is the case when q == 1 and r = k
        //
        // case: a < b
        //
        //   let k = b - a
        // - 2^{max_bits} - k == q * 2^{max_bits} + r
        // - This is only the case when q == 0 and r == 2^{max_bits} - k
        //
        let (q, _) = self.euclidean_division(
            &comparison_evaluation,
            &Expression::from(two_max_bits),
            max_bits + 1,
            &predicate,
        )?;
        Ok(q)
    }

    pub(crate) fn brillig(
        &mut self,
        predicate: Option<Expression>,
        code: GeneratedBrillig,
        inputs: Vec<BrilligInputs>,
        outputs: Vec<BrilligOutputs>,
    ) {
        let opcode = AcirOpcode::Brillig(AcvmBrillig {
            inputs,
            outputs,
            foreign_call_results: Vec::new(),
            bytecode: code.byte_code,
            predicate,
        });
        self.push_opcode(opcode);
        for (brillig_index, call_stack) in code.locations {
            self.locations.insert(
                OpcodeLocation::Brillig { acir_index: self.opcodes.len() - 1, brillig_index },
                call_stack,
            );
        }
    }

    /// Generate gates and control bits witnesses which ensure that out_expr is a permutation of in_expr
    /// Add the control bits of the sorting network used to generate the constrains
    /// into the PermutationSort directive for solving in ACVM.
    /// The directive is solving the control bits so that the outputs are sorted in increasing order.
    ///
    /// n.b. A sorting network is a predetermined set of switches,
    /// the control bits indicate the configuration of each switch: false for pass-through and true for cross-over
    pub(crate) fn permutation(
        &mut self,
        in_expr: &[Expression],
        out_expr: &[Expression],
    ) -> Result<(), RuntimeError> {
        let mut bits_len = 0;
        for i in 0..in_expr.len() {
            bits_len += ((i + 1) as f32).log2().ceil() as u32;
        }

        let bits = vecmap(0..bits_len, |_| self.next_witness_index());
        let inputs = in_expr.iter().map(|a| vec![a.clone()]).collect();
        self.push_opcode(AcirOpcode::Directive(Directive::PermutationSort {
            inputs,
            tuple: 1,
            bits: bits.clone(),
            sort_by: vec![0],
        }));
        let (_, b) = self.permutation_layer(in_expr, &bits, false)?;

        // Constrain the network output to out_expr
        for (b, o) in b.iter().zip(out_expr) {
            self.push_opcode(AcirOpcode::Arithmetic(b - o));
        }
        Ok(())
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
        BlackBoxFunc::Keccak256
        | BlackBoxFunc::SHA256
        | BlackBoxFunc::Blake2s
        | BlackBoxFunc::Pedersen
        | BlackBoxFunc::HashToField128Security => None,

        // Can only apply a range constraint to one
        // witness at a time.
        BlackBoxFunc::RANGE => Some(1),

        // Signature verification algorithms will take in a variable
        // number of inputs, since the message/hashed-message can vary in size.
        BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::EcdsaSecp256r1 => None,
        // Inputs for fixed based scalar multiplication
        // is just a scalar
        BlackBoxFunc::FixedBaseScalarMul => Some(1),
        // Recursive aggregation has a variable number of inputs
        BlackBoxFunc::RecursiveAggregation => None,
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
        BlackBoxFunc::Keccak256 | BlackBoxFunc::SHA256 | BlackBoxFunc::Blake2s => Some(32),
        // Hash to field returns a field element
        BlackBoxFunc::HashToField128Security => Some(1),
        // Pedersen returns a point
        BlackBoxFunc::Pedersen => Some(2),
        // Can only apply a range constraint to one
        // witness at a time.
        BlackBoxFunc::RANGE => Some(0),
        // Signature verification algorithms will return a boolean
        BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::EcdsaSecp256r1 => Some(1),
        // Output of fixed based scalar mul over the embedded curve
        // will be 2 field elements representing the point.
        BlackBoxFunc::FixedBaseScalarMul => Some(2),
        // Recursive aggregation has a variable number of outputs
        BlackBoxFunc::RecursiveAggregation => None,
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

    assert_eq!(expected_num_outputs,output_count,"Tried to call black box function {name} with {output_count} inputs, but this function's definition requires {expected_num_outputs} inputs");
}
