//! `GeneratedAcir` is constructed as part of the `acir_gen` pass to accumulate all of the ACIR
//! program as it is being converted from SSA form.
use super::errors::AcirGenError;
use acvm::acir::{
    circuit::{
        directives::QuotientDirective,
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    },
    native_types::Witness,
};
use acvm::{
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};

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
}

impl GeneratedAcir {
    /// Returns the current witness index.
    pub(crate) fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    /// Adds a new opcode into ACIR.
    fn push_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode);
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    pub(crate) fn next_witness_index(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    /// Converts an expression into a Witness.
    ///
    /// This is done by creating a new Witness and creating an opcode which
    /// sets the Witness to be equal to the expression.
    ///
    /// The reason we do this is because _constraints_ in ACIR have a degree limit
    /// This means you cannot multiply an infinite amount of Expressions together.
    /// Once the expression goes over degree-2, then it needs to be reduced to a Witness
    /// which has degree-1 in order to be able to continue the multiplication chain.
    fn expression_to_witness(&mut self, expression: &Expression) -> Witness {
        let fresh_witness = self.next_witness_index();

        // Create a constraint that sets them to be equal to each other
        // Then return the witness as this can now be used in places
        // where we would have used the Witness.
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
    /// If `expr` can be represented as a `Witness` this function will
    /// return it, else a new opcode will be added to create a Witness
    /// that is equal to `expr`.
    pub(crate) fn get_or_create_witness(&mut self, expr: &Expression) -> Witness {
        match expr.to_witness() {
            Some(witness) => witness,
            None => self.expression_to_witness(expr),
        }
    }
    /// Adds an inversion directive.
    ///
    /// This directive will invert `expr` without applying constraints
    /// and return a `Witness` which may or may not be the result of
    /// inverting `expr`.
    ///
    /// Safety: It is the callers responsibility to ensure that the
    /// resulting `Witness` is constrained to be the inverse.
    pub(crate) fn directive_inverse(&mut self, expr: &Expression) -> Witness {
        // The inversion directive requires that
        // the inputs be Witness, so we need this potential extra
        // reduction constraint.
        // Note: changing this in ACIR would allow us to remove it
        let witness = self.get_or_create_witness(expr);

        // Create the witness for the result
        let inverted_witness = self.next_witness_index();

        self.push_opcode(AcirOpcode::Directive(Directive::Invert {
            x: witness,
            result: inverted_witness,
        }));

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
    /// - `1` if lhs == rhs
    /// - `0` otherwise
    ///
    /// Intuition: the equality of two Expressions is linked to whether
    /// their difference has an inverse; a == b implies that a - b == 0
    /// which implies that a - b has no inverse. So if two variables are equal,
    /// their difference will have no inverse.
    ///
    /// First, lets create a new variable that is equal to the difference
    /// of the two expressions: `t = lhs - rhs` (constraint has been applied)
    ///
    /// Next lets create a new variable `y` which will be the Witness that we will ultimately
    /// return indicating whether lhs == rhs.
    /// Note: We eventually need to apply constraints that ensure that it is a boolean.
    /// But right now with no constraints applied to it, it is essentially a free variable.
    ///
    /// Next we apply the following constraint `y * t == 0`.
    /// This implies that either `y` or `t` or both is `0`.
    /// - If t == 0, then this means that lhs == rhs.
    /// - If y == 0, this does not mean anything at this point in time, due to it having no
    /// constraints.
    ///
    /// Naively, we could apply the following constraint: y == 1 - t.
    /// This along with the previous `y * t == 0` constraint means that
    /// `y` or `t` needs to be zero, but they both cannot be zero.
    ///
    /// This equation however falls short when lhs != rhs because then `t`
    /// may not be `1`. If `t` is non-zero, then `y` is also non-zero due to
    /// `y == 1 - t` and the equation `y * t == 0` fails.  
    ///
    /// To fix, we introduce another free variable called `z` and apply the following
    /// constraint instead: y == 1 - t * z.
    ///
    /// When lhs == rhs, t is zero and so `y` is `1`.
    /// When lhs != rhs, t is non-zero, however the prover can set `z = 1/t`
    /// which will make `y` = 1 - t * 1/t =  `0`.
    ///
    /// We now arrive at the conclusion that when lhs == rhs, `y` is `1` and when
    /// lhs != rhs, then `y` is `0`.
    ///  
    /// Bringing it all together, We introduce three variables `y`, `t` and `z`,
    /// With the following equations:
    /// - t == lhs - rhs
    /// - y == 1 - tz (z is a value that is chosen to be the inverse by the prover)
    /// - y * t == 0
    ///
    /// Lets convince ourselves that the prover cannot prove an untrue statement.
    ///
    /// Assume that lhs == rhs, can the prover return y == 0 ?
    ///
    /// When lhs == rhs, `t` is 0. There is no way to make `y` be zero
    /// since `y = 1 - 0 * z = 1`.
    ///
    /// Assume that lhs != rhs, can the prover return y == 1 ?
    ///
    /// When lhs != rhs, then `t` is non-zero.
    /// By setting `z` to be 0, we can make `y` equal to `1`.
    /// This is easily observed: `y = 1 - t * 0`
    /// Now since `y` is one, this means that `t` needs to be zero, or else `y * t == 0` will fail.
    pub(crate) fn is_equal(&mut self, lhs: &Expression, rhs: &Expression) -> Witness {
        let t = lhs - rhs;

        // This conversion is needed due to us calling Directive::Inverse;
        //
        // We avoid calling directive::inverse(expr) because we need
        // the Witness representation for the Expression.
        let t_witness = self.get_or_create_witness(&t);

        // Call the inversion directive, since we do not apply a constraint
        // the prover can choose anything here.
        let z = self.directive_inverse(&Expression::from(t_witness));

        let y = self.next_witness_index();

        // Add constraint y == 1 - tz => y + tz - 1 == 0
        let y_booleanity_constraint = Expression {
            mul_terms: vec![(FieldElement::one(), t_witness, z)],
            linear_combinations: vec![(FieldElement::one(), y)],
            q_c: -FieldElement::one(),
        };
        self.assert_is_zero(y_booleanity_constraint);

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
    /// integer within the range [0, 2^{num_bits} - 1]
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
    /// See [R1CS Workshop - Section 10](https://github.com/mir-protocol/r1cs-workshop/blob/master/workshop.pdf)
    /// for an explanation.
    pub(crate) fn more_than_eq_comparison(
        &mut self,
        a: &Expression,
        b: &Expression,
        max_bits: u32,
    ) -> Result<Witness, AcirGenError> {
        // Ensure that 2^{max_bits + 1} is less than the field size
        //
        // TODO: perhaps this should be a user error, instead of an assert
        assert!(max_bits + 1 < FieldElement::max_num_bits());

        // Compute : 2^max_bits + a - b
        let mut comparison_evaluation = a - b;
        let two = FieldElement::from(2_i128);
        let two_max_bits = two.pow(&FieldElement::from(max_bits as i128));
        comparison_evaluation.q_c += two_max_bits;

        let q_witness = self.next_witness_index();
        let r_witness = self.next_witness_index();

        // Add constraint : 2^{max_bits} + a - b = q * 2^{max_bits} + r
        let mut expr = Expression::default();
        expr.push_addition_term(two_max_bits, q_witness);
        expr.push_addition_term(FieldElement::one(), r_witness);
        self.push_opcode(AcirOpcode::Arithmetic(&comparison_evaluation - &expr));

        self.push_opcode(AcirOpcode::Directive(Directive::Quotient(QuotientDirective {
            a: comparison_evaluation,
            b: Expression::from_field(two_max_bits),
            q: q_witness,
            r: r_witness,
            predicate: None,
        })));

        // Add constraint to ensure `r` is correctly bounded
        // between [0, 2^{max_bits}-1]
        self.range_constraint(r_witness, max_bits)?;
        // Add constraint to ensure that `q` is a boolean value
        // in particular it should be the `n` bit of the comparison_evaluation
        // which will indicate whether a >= b
        //
        // In the document linked above, they mention negating the value of `q`
        // which would tell us whether a < b. Since we do not negate `q`
        // what we get is a boolean indicating whether a >= b.
        self.range_constraint(q_witness, 1)?;

        Ok(q_witness)
    }
}
