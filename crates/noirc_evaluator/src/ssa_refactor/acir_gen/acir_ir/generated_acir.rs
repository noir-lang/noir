//! `GeneratedAcir` is constructed as part of the `acir_gen` pass to accumulate all of the ACIR
//! program as it is being converted from SSA form.
use std::collections::HashMap;

use crate::brillig::brillig_gen::brillig_directive;

use super::errors::AcirGenError;
use acvm::acir::{
    brillig::Opcode as BrilligOpcode,
    circuit::{
        brillig::{Brillig as AcvmBrillig, BrilligInputs, BrilligOutputs},
        directives::{LogInfo, QuotientDirective},
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    },
    native_types::Witness,
    BlackBoxFunc,
};
use acvm::{
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};
use iter_extended::vecmap;
use noirc_errors::Location;
use num_bigint::BigUint;

#[derive(Debug, Default)]
/// The output of the Acir-gen pass
pub(crate) struct GeneratedAcir {
    /// The next witness index that may be declared.
    ///
    /// Equivalent to acvm::acir::circuit::Circuit's field of the same name.
    pub(crate) current_witness_index: u32,

    /// The opcodes of which the compiled ACIR will comprise.
    pub(crate) opcodes: Vec<AcirOpcode>,

    /// All witness indices that comprise the final return value of the program
    ///
    /// Note: This may contain repeated indices, which is necessary for later mapping into the
    /// abi's return type.
    pub(crate) return_witnesses: Vec<Witness>,

    /// Correspondance between an opcode index (in opcodes) and the source code location which generated it
    pub(crate) locations: HashMap<usize, Location>,

    /// Source code location of the current instruction being processed
    /// None if we do not know the location
    pub(crate) current_location: Option<Location>,
}

impl GeneratedAcir {
    /// Returns the current witness index.
    pub(crate) fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    /// Adds a new opcode into ACIR.
    fn push_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode);
        if let Some(location) = self.current_location {
            self.locations.insert(self.opcodes.len() - 1, location);
        }
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
    /// Computes lhs = 2^{rhs_bit_size} * q + r
    ///
    /// For example, if we had a u32:
    ///     - `rhs` would be `32`
    ///     - `max_bits` would be the size of `lhs`
    ///
    /// Take the following code:
    /// ``
    ///   fn main(x : u32) -> u32 {
    ///     let a = x + x; (L1)
    ///     let b = a * a; (L2)
    ///     b + b (L3)
    ///   }
    /// ``
    ///
    ///  Call truncate only on L1:
    ///     - `rhs` would be `32`
    ///     - `max_bits` would be `33` due to the addition of two u32s
    ///  Call truncate only on L2:
    ///     - `rhs` would be `32`
    ///     - `max_bits` would be `66` due to the multiplication of two u33s `a`
    ///  Call truncate only on L3:
    ///     -  `rhs` would be `32`
    ///     - `max_bits` would be `67` due to the addition of two u66s `b`
    ///
    /// Truncation is done via the euclidean division formula:
    ///
    /// a = b * q + r
    ///
    /// where:
    ///     - a = `lhs`
    ///     - b = 2^{max_bits}
    /// The prover will supply the quotient and the remainder, where the remainder
    /// is the truncated value that we will return since it is enforced to be
    /// in the range:  0 <= r < 2^{rhs_bit_size}
    pub(crate) fn truncate(
        &mut self,
        lhs: &Expression,
        rhs_bit_size: u32,
        max_bits: u32,
    ) -> Result<Expression, AcirGenError> {
        assert!(max_bits > rhs_bit_size, "max_bits = {max_bits}, rhs = {rhs_bit_size} -- The caller should ensure that truncation is only called when the value needs to be truncated");
        let exp_big = BigUint::from(2_u32).pow(rhs_bit_size);

        // 0. Check for constant expression.
        if let Some(a_c) = lhs.to_const() {
            let mut a_big = BigUint::from_bytes_be(&a_c.to_be_bytes());
            a_big %= exp_big;
            return Ok(Expression::from(FieldElement::from_be_bytes_reduce(&a_big.to_bytes_be())));
        }
        // Note: This is doing a reduction. However, since the compiler will call
        // `max_bits` before it overflows the modulus, this line should never do a reduction.
        //
        // For example, if the modulus is a 254 bit number.
        // `max_bits` will never be 255 since `exp` will be 2^255, which will cause a reduction in the following line.
        // TODO: We should change this from `from_be_bytes_reduce` to `from_be_bytes`
        // TODO: the latter will return an option that we can unwrap in the compiler
        let exp = FieldElement::from_be_bytes_reduce(&exp_big.to_bytes_be());

        // 1. Generate witnesses a,b,c

        // According to the division theorem, the remainder needs to be 0 <= r < 2^{rhs_bit_size}
        let r_max_bits = rhs_bit_size;
        // According to the formula above, the quotient should be within the range 0 <= q < 2^{max_bits - rhs}
        let q_max_bits = max_bits - rhs_bit_size;

        let (quotient_witness, remainder_witness) =
            self.quotient_directive(lhs.clone(), exp.into(), None, q_max_bits, r_max_bits)?;

        // 2. Add the constraint a == r + (q * 2^{rhs})
        //
        // 2^{rhs}
        let mut two_pow_rhs_bits = FieldElement::from(2_i128);
        two_pow_rhs_bits = two_pow_rhs_bits.pow(&FieldElement::from(rhs_bit_size as i128));

        let remainder_expr = Expression::from(remainder_witness);
        let quotient_expr = Expression::from(quotient_witness);

        let res = &remainder_expr + &(two_pow_rhs_bits * &quotient_expr);
        let euclidean_division = &res - lhs;

        self.push_opcode(AcirOpcode::Arithmetic(euclidean_division));

        Ok(Expression::from(remainder_witness))
    }

    /// Calls a black box function and returns the output
    /// of said blackbox function.
    pub(crate) fn call_black_box(
        &mut self,
        func_name: BlackBoxFunc,
        mut inputs: Vec<FunctionInput>,
        constants: Vec<FieldElement>,
    ) -> Vec<Witness> {
        intrinsics_check_inputs(func_name, &inputs);

        let output_count = black_box_expected_output_size(func_name);
        let outputs = vecmap(0..output_count, |_| self.next_witness_index());

        // clone is needed since outputs is moved when used in blackbox function.
        let outputs_clone = outputs.clone();

        let black_box_func_call = match func_name {
            BlackBoxFunc::AND => {
                BlackBoxFuncCall::AND { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::XOR => {
                BlackBoxFuncCall::XOR { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: inputs[0] },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs, outputs },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s { inputs, outputs },
            BlackBoxFunc::HashToField128Security => {
                BlackBoxFuncCall::HashToField128Security { inputs, output: outputs[0] }
            }
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: inputs[0],
                public_key_y: inputs[1],
                // Schnorr signature is an r & s, 32 bytes each
                signature: inputs[2..66].to_vec(),
                message: inputs[66..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::Pedersen => BlackBoxFuncCall::Pedersen {
                inputs,
                outputs: (outputs[0], outputs[1]),
                domain_separator: constants[0].to_u128() as u32,
            },
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                // 32 bytes for each public key co-ordinate
                public_key_x: inputs[0..32].to_vec(),
                public_key_y: inputs[32..64].to_vec(),
                // (r,s) are both 32 bytes each, so signature
                // takes up 64 bytes
                signature: inputs[64..128].to_vec(),
                hashed_message: inputs[128..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::EcdsaSecp256r1 => BlackBoxFuncCall::EcdsaSecp256r1 {
                // 32 bytes for each public key co-ordinate
                public_key_x: inputs[0..32].to_vec(),
                public_key_y: inputs[32..64].to_vec(),
                // (r,s) are both 32 bytes each, so signature
                // takes up 64 bytes
                signature: inputs[64..128].to_vec(),
                hashed_message: inputs[128..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::FixedBaseScalarMul => BlackBoxFuncCall::FixedBaseScalarMul {
                input: inputs[0],
                outputs: (outputs[0], outputs[1]),
            },
            BlackBoxFunc::Keccak256 => {
                let var_message_size = inputs.pop().expect("ICE: Missing message_size arg");
                BlackBoxFuncCall::Keccak256VariableLength { inputs, var_message_size, outputs }
            }
            // TODO(#1570): Generate ACIR for recursive aggregation
            BlackBoxFunc::RecursiveAggregation => {
                panic!("ICE: Cannot generate ACIR for recursive aggregation")
            }
        };

        self.opcodes.push(AcirOpcode::BlackBoxFuncCall(black_box_func_call));

        outputs_clone
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
    ) -> Result<Vec<Witness>, AcirGenError> {
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
        let inter = &(&Expression::from_field(max_power_of_two) - lhs) * &leading.into();
        lhs.add_mul(FieldElement::from(2_i128), &inter.unwrap())
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
    ) -> Result<(Expression, Expression), AcirGenError> {
        // 2^{max_bit size-1}
        let max_power_of_two =
            FieldElement::from(2_i128).pow(&FieldElement::from(max_bit_size as i128 - 1));

        // Get the sign bit of rhs by computing rhs / max_power_of_two
        let (rhs_leading, _) = self.euclidean_division(
            rhs,
            &max_power_of_two.into(),
            max_bit_size,
            &Expression::one(),
        )?;

        // Get the sign bit of lhs by computing lhs / max_power_of_two
        let (lhs_leading, _) = self.euclidean_division(
            lhs,
            &max_power_of_two.into(),
            max_bit_size,
            &Expression::one(),
        )?;

        // Signed to unsigned:
        let unsigned_lhs = self.two_complement(lhs, lhs_leading, max_bit_size);
        let unsigned_rhs = self.two_complement(rhs, rhs_leading, max_bit_size);
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
        let q_sign = (&Expression::from(lhs_leading) + &Expression::from(rhs_leading)).add_mul(
            -FieldElement::from(2_i128),
            &(&Expression::from(lhs_leading) * &Expression::from(rhs_leading)).unwrap(),
        );
        let q_sign_witness = self.get_or_create_witness(&q_sign);
        let quotient = self.two_complement(&q1.into(), q_sign_witness, max_bit_size);
        let remainder = self.two_complement(&r1.into(), lhs_leading, max_bit_size);
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
    ) -> Result<(Witness, Witness), AcirGenError> {
        // lhs = rhs * q + r
        //
        // If predicate is zero, `q_witness` and `r_witness` will be 0
        let (q_witness, r_witness) = self.quotient_directive(
            lhs.clone(),
            rhs.clone(),
            Some(predicate.clone()),
            max_bit_size,
            max_bit_size,
        )?;

        // Constrain r < rhs
        self.bound_constraint_with_offset(&r_witness.into(), rhs, predicate, max_bit_size)?;

        // a * predicate == (b * q + r) * predicate
        // => predicate * ( a - b * q - r) == 0
        // When the predicate is 0, the equation always passes.
        // When the predicate is 1, the euclidean division needs to be
        // true.
        let mut rhs_constraint = (rhs * &Expression::from(q_witness)).unwrap();
        rhs_constraint = &rhs_constraint + r_witness;
        rhs_constraint = (&rhs_constraint * predicate).unwrap();
        let lhs_constraint = (lhs * predicate).unwrap();
        let div_euclidean = &lhs_constraint - &rhs_constraint;

        self.push_opcode(AcirOpcode::Arithmetic(div_euclidean));

        Ok((q_witness, r_witness))
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
    ) -> Result<(), AcirGenError> {
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
            let witness = self.create_witness_for_expression(&aor);
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

    /// Adds a log directive to print the provided witnesses.
    ///
    /// Logging of strings is currently unsupported.
    pub(crate) fn call_print(&mut self, witnesses: Vec<Witness>) {
        self.push_opcode(AcirOpcode::Directive(Directive::Log(LogInfo::WitnessOutput(witnesses))));
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
    ) -> Result<(), AcirGenError> {
        // We class this as an error because users should instead
        // do `as Field`.
        if num_bits >= FieldElement::max_num_bits() {
            return Err(AcirGenError::InvalidRangeConstraint {
                num_bits: FieldElement::max_num_bits(),
                location: self.current_location,
            });
        };

        let constraint = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness, num_bits },
        });
        self.push_opcode(constraint);

        Ok(())
    }

    /// Adds a directive which injects witnesses with values `q = a / b` and `r = a % b`.
    ///
    /// Suitable range constraints are also applied to `q` and `r`.
    pub(crate) fn quotient_directive(
        &mut self,
        a: Expression,
        b: Expression,
        predicate: Option<Expression>,
        q_max_bits: u32,
        r_max_bits: u32,
    ) -> Result<(Witness, Witness), AcirGenError> {
        let q_witness = self.next_witness_index();
        let r_witness = self.next_witness_index();

        let directive =
            Directive::Quotient(QuotientDirective { a, b, q: q_witness, r: r_witness, predicate });
        self.push_opcode(AcirOpcode::Directive(directive));

        // Apply range constraints to injected witness values.
        // Constrains `q` to be 0 <= q < 2^{q_max_bits}, etc.
        self.range_constraint(q_witness, q_max_bits)?;
        self.range_constraint(r_witness, r_max_bits)?;

        Ok((q_witness, r_witness))
    }

    /// Returns a `Witness` that is constrained to be:
    /// - `1` if lhs >= rhs
    /// - `0` otherwise
    ///
    /// See [R1CS Workshop - Section 10](https://github.com/mir-protocol/r1cs-workshop/blob/master/workshop.pdf)
    /// for an explanation.
    pub(crate) fn more_than_eq_comparison(
        &mut self,
        a: &Expression,
        b: &Expression,
        max_bits: u32,
        predicate: Option<Expression>,
    ) -> Result<Witness, AcirGenError> {
        // Ensure that 2^{max_bits + 1} is less than the field size
        //
        // TODO: perhaps this should be a user error, instead of an assert
        assert!(max_bits + 1 < FieldElement::max_num_bits());

        // Compute : 2^{max_bits} + a - b
        let two = FieldElement::from(2_i128);
        let two_max_bits: FieldElement = two.pow(&FieldElement::from(max_bits as i128));
        let comparison_evaluation = (a - b) + two_max_bits;

        // We want to enforce that `q` is a boolean value.
        // In particular it should be the `n` bit of the `comparison_evaluation`
        // which will indicate whether a >= b.
        //
        // In the document linked above, they mention negating the value of `q`
        // which would tell us whether a < b. Since we do not negate `q`
        // what we get is a boolean indicating whether a >= b.
        let q_max_bits = 1;
        // `r` can take any value up to `two_max_bits`.
        let r_max_bits = max_bits;

        let (q_witness, r_witness) = self.quotient_directive(
            comparison_evaluation.clone(),
            two_max_bits.into(),
            predicate,
            q_max_bits,
            r_max_bits,
        )?;

        // Add constraint : 2^{max_bits} + a - b = q * 2^{max_bits} + r
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
        let mut expr = Expression::default();
        expr.push_addition_term(two_max_bits, q_witness);
        expr.push_addition_term(FieldElement::one(), r_witness);
        self.push_opcode(AcirOpcode::Arithmetic(&comparison_evaluation - &expr));

        Ok(q_witness)
    }

    pub(crate) fn brillig(
        &mut self,
        predicate: Option<Expression>,
        code: Vec<BrilligOpcode>,
        inputs: Vec<BrilligInputs>,
        outputs: Vec<BrilligOutputs>,
    ) {
        let opcode = AcirOpcode::Brillig(AcvmBrillig {
            inputs,
            outputs,
            foreign_call_results: Vec::new(),
            bytecode: code,
            predicate,
        });
        self.push_opcode(opcode);
    }

    /// Generate gates and control bits witnesses which ensure that out_expr is a permutation of in_expr
    /// Add the control bits of the sorting network used to generate the constrains
    /// into the PermutationSort directive for solving in ACVM.
    /// The directive is solving the control bits so that the outputs are sorted in increasing order.
    ///
    /// n.b. A sorting network is a predetermined set of switches,
    /// the control bits indicate the configuration of each switch: false for pass-through and true for cross-over
    pub(crate) fn permutation(&mut self, in_expr: &[Expression], out_expr: &[Expression]) {
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
        let (_, b) = self.permutation_layer(in_expr, &bits, false);

        // Constrain the network output to out_expr
        for (b, o) in b.iter().zip(out_expr) {
            self.push_opcode(AcirOpcode::Arithmetic(b - o));
        }
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
        // TODO(#1570): Generate ACIR for recursive aggregation
        // RecursiveAggregation has variable inputs and we could return `None` here,
        // but as it is not fully implemented we panic for now
        BlackBoxFunc::RecursiveAggregation => {
            panic!("ICE: Cannot generate ACIR for recursive aggregation")
        }
    }
}

/// This function will return the number of outputs that a blackbox function
/// expects. Returning `None` if there is no expectation.
fn black_box_expected_output_size(name: BlackBoxFunc) -> u32 {
    match name {
        // Bitwise opcodes will return 1 parameter which is the output
        // or the operation.
        BlackBoxFunc::AND | BlackBoxFunc::XOR => 1,
        // 32 byte hash algorithms
        BlackBoxFunc::Keccak256 | BlackBoxFunc::SHA256 | BlackBoxFunc::Blake2s => 32,
        // Hash to field returns a field element
        BlackBoxFunc::HashToField128Security => 1,
        // Pedersen returns a point
        BlackBoxFunc::Pedersen => 2,
        // Can only apply a range constraint to one
        // witness at a time.
        BlackBoxFunc::RANGE => 0,
        // Signature verification algorithms will return a boolean
        BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::EcdsaSecp256r1 => 1,
        // Output of fixed based scalar mul over the embedded curve
        // will be 2 field elements representing the point.
        BlackBoxFunc::FixedBaseScalarMul => 2,
        // TODO(#1570): Generate ACIR for recursive aggregation
        BlackBoxFunc::RecursiveAggregation => {
            panic!("ICE: Cannot generate ACIR for recursive aggregation")
        }
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
fn intrinsics_check_inputs(name: BlackBoxFunc, inputs: &[FunctionInput]) {
    let expected_num_inputs = match black_box_func_expected_input_size(name) {
        Some(expected_num_inputs) => expected_num_inputs,
        None => return,
    };
    let got_num_inputs = inputs.len();

    assert_eq!(expected_num_inputs,inputs.len(),"Tried to call black box function {name} with {got_num_inputs} inputs, but this function's definition requires {expected_num_inputs} inputs");
}
