#[macro_export]
macro_rules! impl_uint {
    (
        $name:ident,
        $type:ty,
        $size:expr
    ) => {
        use acir::{
            brillig::{self, RegisterIndex},
            circuit::{
                brillig::{Brillig, BrilligInputs, BrilligOutputs},
                directives::QuotientDirective,
                opcodes::{BlackBoxFuncCall, FunctionInput},
                Opcode,
            },
            native_types::{Expression, Witness},
            FieldElement,
        };
        use $crate::helpers::VariableStore;

        /// UInt contains a witness that points to a field element that represents a u32 integer
        /// It has a inner field of type [Witness] that points to the field element and width = 32
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            pub(crate) inner: Witness,
            width: u32,
        }

        impl $name {
            #[cfg(any(test, feature = "testing"))]
            pub fn get_inner(&self) -> Witness {
                self.inner
            }
        }

        impl $name {
            /// Initialize A new [UInt] type with a [Witness]
            pub fn new(witness: Witness) -> Self {
                $name { inner: witness, width: $size }
            }

            /// Get u(n) + 1
            pub(crate) fn get_max_plus_one(
                &self,
                mut num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![],
                        q_c: FieldElement::from(2_u128.pow(self.width)),
                    })],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],

                    bytecode: vec![brillig::Opcode::Stop],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Load a constant into the circuit
            pub(crate) fn load_constant(
                constant: $type,
                mut num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![],
                        q_c: FieldElement::from(constant as u128),
                    })],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],
                    bytecode: vec![brillig::Opcode::Stop],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Returns the quotient and remainder such that lhs = rhs * quotient + remainder
            // This should be the same as its equivalent in the Noir repo
            pub fn euclidean_division(
                lhs: &$name,
                rhs: &$name,
                mut num_witness: u32,
            ) -> ($name, $name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let q_witness = variables.new_variable();
                let r_witness = variables.new_variable();

                // compute quotient using directive function
                let quotient_opcode = Opcode::Directive(
                    acir::circuit::directives::Directive::Quotient(QuotientDirective {
                        a: lhs.inner.into(),
                        b: rhs.inner.into(),
                        q: q_witness,
                        r: r_witness,
                        predicate: None,
                    }),
                );
                new_opcodes.push(quotient_opcode);

                // make sure r and q are in 32 bit range
                let r_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput { witness: r_witness, num_bits: lhs.width },
                });
                let q_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput { witness: q_witness, num_bits: lhs.width },
                });
                new_opcodes.push(r_range_opcode);
                new_opcodes.push(q_range_opcode);
                let num_witness = variables.finalize();

                // constrain r < rhs
                let (rhs_sub_r, extra_opcodes, num_witness) =
                    rhs.sub_no_overflow(&$name::new(r_witness), num_witness);
                new_opcodes.extend(extra_opcodes);
                let rhs_sub_r_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput { witness: rhs_sub_r.inner, num_bits: lhs.width },
                });
                new_opcodes.push(rhs_sub_r_range_opcode);

                // constrain lhs = rhs * quotient + remainder
                let rhs_expr = Expression::from(rhs.inner);
                let lhs_constraint = Expression::from(lhs.inner);
                let rhs_constraint = &rhs_expr * &Expression::from(q_witness);
                let rhs_constraint = &rhs_constraint.unwrap() + &Expression::from(r_witness);
                let div_euclidean = &lhs_constraint - &rhs_constraint;
                new_opcodes.push(Opcode::Arithmetic(div_euclidean));

                ($name::new(q_witness), $name::new(r_witness), new_opcodes, num_witness)
            }

            /// Rotate left `rotation` bits. `(x << rotation) | (x >> (width - rotation))`
            // This should be the same as `u32.rotate_left(rotation)` in rust stdlib
            pub fn rol(&self, rotation: u32, num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let rotation = rotation % self.width;
                let mut new_opcodes = Vec::new();
                let (right_shift, extra_opcodes, num_witness) =
                    self.rightshift(self.width - rotation, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (left_shift, extra_opcodes, num_witness) =
                    self.leftshift(rotation, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (result, extra_opcodes, num_witness) = left_shift.or(&right_shift, num_witness);
                new_opcodes.extend(extra_opcodes);

                (result, new_opcodes, num_witness)
            }

            /// Rotate right `rotation` bits. `(x >> rotation) | (x << (width - rotation))`
            // This should be the same as `u32.rotate_right(rotation)` in rust stdlib
            pub fn ror(&self, rotation: u32, num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let rotation = rotation % self.width;
                let mut new_opcodes = Vec::new();
                let (left_shift, extra_opcodes, num_witness) =
                    self.leftshift(self.width - rotation, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (right_shift, extra_opcodes, num_witness) =
                    self.rightshift(rotation, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (result, extra_opcodes, num_witness) = left_shift.or(&right_shift, num_witness);
                new_opcodes.extend(extra_opcodes);

                (result, new_opcodes, num_witness)
            }

            /// left shift by `bits`
            pub fn leftshift(&self, bits: u32, num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let bits = bits % self.width;
                let mut new_opcodes = Vec::new();
                let two: $type = 2;
                let (two_pow_rhs, extra_opcodes, num_witness) =
                    $name::load_constant(two.pow(bits), num_witness);
                new_opcodes.extend(extra_opcodes);
                let (left_shift, extra_opcodes, num_witness) = self.mul(&two_pow_rhs, num_witness);
                new_opcodes.extend(extra_opcodes);

                (left_shift, new_opcodes, num_witness)
            }

            /// right shift by `bits`
            pub fn rightshift(&self, bits: u32, num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let bits = bits % self.width;
                let mut new_opcodes = Vec::new();
                let two: $type = 2;
                let (two_pow_rhs, extra_opcodes, num_witness) =
                    $name::load_constant(two.pow(bits), num_witness);
                new_opcodes.extend(extra_opcodes);
                let (right_shift, _, extra_opcodes, num_witness) =
                    $name::euclidean_division(self, &two_pow_rhs, num_witness);
                new_opcodes.extend(extra_opcodes);

                (right_shift, new_opcodes, num_witness)
            }

            /// Caculate and constrain `self` + `rhs`
            pub fn add(&self, rhs: &$name, mut num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                // calculate `self` + `rhs` with overflow
                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                            q_c: FieldElement::zero(),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],

                    bytecode: vec![brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 127,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(1),
                        destination: RegisterIndex::from(0),
                    }],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                // constrain addition
                let mut add_expr = Expression::from(new_witness);
                add_expr.push_addition_term(-FieldElement::one(), self.inner);
                add_expr.push_addition_term(-FieldElement::one(), rhs.inner);
                new_opcodes.push(Opcode::Arithmetic(add_expr));

                // mod 2^width to get final result as the remainder
                let (two_pow_width, extra_opcodes, num_witness) =
                    self.get_max_plus_one(num_witness);
                new_opcodes.extend(extra_opcodes);
                let (_, add_mod, extra_opcodes, num_witness) = $name::euclidean_division(
                    &$name::new(new_witness),
                    &two_pow_width,
                    num_witness,
                );
                new_opcodes.extend(extra_opcodes);

                (add_mod, new_opcodes, num_witness)
            }

            /// Caculate and constrain `self` - `rhs`
            pub fn sub(&self, rhs: &$name, mut num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                // calculate 2^32 + self - rhs to avoid overflow
                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![],
                            q_c: FieldElement::from(1_u128 << self.width),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],

                    bytecode: vec![
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Add,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(2),
                            destination: RegisterIndex::from(0),
                        },
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Sub,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(1),
                            destination: RegisterIndex::from(0),
                        },
                    ],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                // constrain subtraction
                let mut sub_constraint = Expression::from(self.inner);
                sub_constraint.push_addition_term(-FieldElement::one(), new_witness);
                sub_constraint.push_addition_term(-FieldElement::one(), rhs.inner);
                sub_constraint.q_c = FieldElement::from(1_u128 << self.width);
                new_opcodes.push(Opcode::Arithmetic(sub_constraint));

                // mod 2^width to get final result as the remainder
                let (two_pow_width, extra_opcodes, num_witness) =
                    self.get_max_plus_one(num_witness);
                new_opcodes.extend(extra_opcodes);
                let (_, sub_mod, extra_opcodes, num_witness) = $name::euclidean_division(
                    &$name::new(new_witness),
                    &two_pow_width,
                    num_witness,
                );
                new_opcodes.extend(extra_opcodes);

                (sub_mod, new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` - `rhs` - 1 without allowing overflow
            /// This is a helper function to `euclidean_division`
            //  There is a `-1` because theres a case where rhs = 2^32 and remainder = 0
            pub(crate) fn sub_no_overflow(
                &self,
                rhs: &$name,
                mut num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                // calculate self - rhs - 1
                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![],
                            q_c: FieldElement::one(),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],
                    bytecode: vec![
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Sub,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(1),
                            destination: RegisterIndex::from(0),
                        },
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Sub,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(2),
                            destination: RegisterIndex::from(0),
                        },
                    ],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                // constrain subtraction
                let mut sub_constraint = Expression::from(self.inner);
                sub_constraint.push_addition_term(-FieldElement::one(), new_witness);
                sub_constraint.push_addition_term(-FieldElement::one(), rhs.inner);
                sub_constraint.q_c = -FieldElement::one();
                new_opcodes.push(Opcode::Arithmetic(sub_constraint));

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` * `rhs`
            pub(crate) fn mul(
                &self,
                rhs: &$name,
                mut num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                // calulate `self` * `rhs` with overflow
                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                            q_c: FieldElement::zero(),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],
                    bytecode: vec![brillig::Opcode::BinaryFieldOp {
                        op: brillig::BinaryFieldOp::Mul,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(1),
                        destination: RegisterIndex::from(0),
                    }],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                // constrain mul
                let mut mul_constraint = Expression::from(new_witness);
                mul_constraint.push_multiplication_term(
                    -FieldElement::one(),
                    self.inner,
                    rhs.inner,
                );
                new_opcodes.push(Opcode::Arithmetic(mul_constraint));

                // mod 2^width to get final result as the remainder
                let (two_pow_rhs, extra_opcodes, num_witness) = self.get_max_plus_one(num_witness);
                new_opcodes.extend(extra_opcodes);
                let (_, mul_mod, extra_opcodes, num_witness) =
                    $name::euclidean_division(&$name::new(new_witness), &two_pow_rhs, num_witness);
                new_opcodes.extend(extra_opcodes);

                (mul_mod, new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` and `rhs`
            pub fn and(&self, rhs: &$name, mut num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();
                let num_witness = variables.finalize();
                let and_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
                    lhs: FunctionInput { witness: self.inner, num_bits: self.width },
                    rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
                    output: new_witness,
                });
                new_opcodes.push(and_opcode);

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` xor `rhs`
            pub fn xor(&self, rhs: &$name, mut num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();
                let num_witness = variables.finalize();
                let xor_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
                    lhs: FunctionInput { witness: self.inner, num_bits: self.width },
                    rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
                    output: new_witness,
                });
                new_opcodes.push(xor_opcode);

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` or `rhs`
            pub fn or(&self, rhs: &$name, num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();

                // a | b = (a & b) + (a ^ b)
                let (a_and_b, extra_opcodes, num_witness) = self.and(rhs, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (a_xor_b, extra_opcodes, num_witness) = self.xor(rhs, num_witness);
                new_opcodes.extend(extra_opcodes);
                let (or, extra_opcodes, num_witness) = a_and_b.add(&a_xor_b, num_witness);
                new_opcodes.extend(extra_opcodes);

                (or, new_opcodes, num_witness)
            }

            /// Calculate and constrain not `self`
            pub(crate) fn not(&self, mut num_witness: u32) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();

                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![],
                            q_c: FieldElement::from((1_u128 << self.width) - 1),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],
                    bytecode: vec![brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Sub,
                        bit_size: self.width,
                        lhs: RegisterIndex::from(1),
                        rhs: RegisterIndex::from(0),
                        destination: RegisterIndex::from(0),
                    }],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                let mut not_constraint = Expression::from(new_witness);
                not_constraint.push_addition_term(FieldElement::one(), self.inner);
                not_constraint.q_c = -FieldElement::from((1_u128 << self.width) - 1);
                new_opcodes.push(Opcode::Arithmetic(not_constraint));

                ($name::new(new_witness), new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` >= `rhs`
            //  This should be similar to its equivalent in the Noir repo
            pub(crate) fn more_than_eq_comparison(
                &self,
                rhs: &$name,
                mut num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let mut variables = VariableStore::new(&mut num_witness);
                let new_witness = variables.new_variable();
                let q_witness = variables.new_variable();
                let r_witness = variables.new_variable();

                // calculate 2^32 + self - rhs
                let brillig_opcode = Opcode::Brillig(Brillig {
                    inputs: vec![
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), self.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                            q_c: FieldElement::zero(),
                        }),
                        BrilligInputs::Single(Expression {
                            mul_terms: vec![],
                            linear_combinations: vec![],
                            q_c: FieldElement::from(1_u128 << self.width),
                        }),
                    ],
                    outputs: vec![BrilligOutputs::Simple(new_witness)],
                    bytecode: vec![
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Add,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(2),
                            destination: RegisterIndex::from(0),
                        },
                        brillig::Opcode::BinaryIntOp {
                            op: brillig::BinaryIntOp::Sub,
                            bit_size: 127,
                            lhs: RegisterIndex::from(0),
                            rhs: RegisterIndex::from(1),
                            destination: RegisterIndex::from(0),
                        },
                    ],
                    predicate: None,
                });
                new_opcodes.push(brillig_opcode);
                let num_witness = variables.finalize();

                // constrain subtraction
                let mut sub_constraint = Expression::from(self.inner);
                sub_constraint.push_addition_term(-FieldElement::one(), new_witness);
                sub_constraint.push_addition_term(-FieldElement::one(), rhs.inner);
                sub_constraint.q_c = FieldElement::from(1_u128 << self.width);
                new_opcodes.push(Opcode::Arithmetic(sub_constraint));

                let (two_pow_rhs, extra_opcodes, num_witness) = self.get_max_plus_one(num_witness);
                new_opcodes.extend(extra_opcodes);

                // constraint 2^{max_bits} + a - b = q * 2^{max_bits} + r
                // q = 1 if a == b
                // q = 1 if a > b
                // q = 0 if a < b
                let quotient_opcode = Opcode::Directive(
                    acir::circuit::directives::Directive::Quotient(QuotientDirective {
                        a: new_witness.into(),
                        b: two_pow_rhs.inner.into(),
                        q: q_witness,
                        r: r_witness,
                        predicate: None,
                    }),
                );
                new_opcodes.push(quotient_opcode);

                // make sure r in 32 bit range and q is 1 bit
                let r_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput { witness: r_witness, num_bits: self.width },
                });
                let q_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput { witness: q_witness, num_bits: 1 },
                });
                new_opcodes.push(r_range_opcode);
                new_opcodes.push(q_range_opcode);

                ($name::new(q_witness), new_opcodes, num_witness)
            }

            /// Calculate and constrain `self` < `rhs`
            pub fn less_than_comparison(
                &self,
                rhs: &$name,
                num_witness: u32,
            ) -> ($name, Vec<Opcode>, u32) {
                let mut new_opcodes = Vec::new();
                let (mut comparison, extra_opcodes, num_witness) =
                    self.more_than_eq_comparison(rhs, num_witness);
                new_opcodes.extend(extra_opcodes);
                comparison.width = 1;

                // `self` < `rhs` == not `self` >= `rhs`
                let (less_than, extra_opcodes, num_witness) = comparison.not(num_witness);
                new_opcodes.extend(extra_opcodes);

                (less_than, new_opcodes, num_witness)
            }
        }
    };
}
