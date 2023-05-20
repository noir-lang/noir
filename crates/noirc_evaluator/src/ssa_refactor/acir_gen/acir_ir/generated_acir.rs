use super::errors::AcirGenError;
use acvm::acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    native_types::Witness,
};
use acvm::{
    acir::{circuit::directives::Directive, native_types::Expression},
    FieldElement,
};

#[derive(Debug, Default)]
/// The output of the Acir-gen pass
pub struct GeneratedAcir {
    pub(crate) current_witness_index: u32,
    opcodes: Vec<AcirOpcode>,
}

impl GeneratedAcir {
    /// Returns the current witness index.
    // TODO: This can be put as a method on `Circuit` in ACVM
    pub fn current_witness_index(&self) -> Witness {
        Witness(self.current_witness_index)
    }

    fn push_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode)
    }

    /// Updates the witness index counter and returns
    /// the next witness index.
    // TODO: This can be put as a method on `Circuit` in ACVM
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
    pub fn expression_to_witness(&mut self, expression: &Expression) -> Witness {
        let fresh_witness = self.next_witness_index();

        // Create a constraint that sets them to be equal to each other
        // Then return the witness as this can now be used in places
        // where we would have used the Witness.
        let constraint = expression - fresh_witness;
        self.assert_is_zero(constraint);

        fresh_witness
    }
}

impl GeneratedAcir {
    pub(crate) fn directive_inverse(&mut self, expr: &Expression) -> Witness {
        // The inversion directive requires that
        // the inputs be Witness, so we need this potential extra
        // reduction constraint.
        // Note: changing this in ACIR would allow us to remove it
        let witness = match expr.to_witness() {
            Some(witness) => witness,
            None => self.expression_to_witness(expr),
        };

        // Create the witness for the result
        let inverted_witness = self.next_witness_index();

        self.push_opcode(AcirOpcode::Directive(Directive::Invert {
            x: witness,
            result: inverted_witness,
        }));

        inverted_witness
    }

    pub(crate) fn assert_is_zero(&mut self, expr: Expression) {
        self.push_opcode(AcirOpcode::Arithmetic(expr))
    }

    pub(crate) fn range_constraint(
        &mut self,
        witness: Witness,
        num_bits: u32,
    ) -> Result<(), AcirGenError> {
        if num_bits == FieldElement::max_num_bits() {
            return Err(AcirGenError::InvalidRangeConstraint {
                num_bits: FieldElement::max_num_bits(),
            });
        };

        // TODO: Note that for odd number of bits, barretenberg may panic
        // TODO: we should check this.
        //
        // TODO: In our previous SSA code, we dealt with this case here,
        // TODO: this is Barretenberg specific, so it was removed.
        //
        // TODO: We also remove the boolean special case, as backends should deal
        // TODO with this usecase.
        let constraint = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: acvm::acir::BlackBoxFunc::RANGE,
            inputs: vec![FunctionInput { witness, num_bits }],
            outputs: vec![],
        });
        self.push_opcode(constraint);

        Ok(())
    }

    // Returns a witness of a >= b
    pub(crate) fn bound_check(
        &mut self,
        a: &Expression,
        b: &Expression,
        max_bits: u32,
    ) -> Result<Witness, AcirGenError> {
        assert!(max_bits + 1 < FieldElement::max_num_bits()); //n.b what we really need is 2^{max_bits+1}<p
        let mut sub = a - b;
        let two = FieldElement::from(2_i128);
        let two_s = two.pow(&FieldElement::from(max_bits as i128));
        sub.q_c += two_s;
        let q_witness = self.next_witness_index();
        let r_witness = self.next_witness_index();

        //2^s+a-b=q*2^s +r
        let mut expr = Expression::default();
        expr.push_addition_term(two_s, q_witness);
        expr.push_addition_term(FieldElement::one(), r_witness);

        self.push_opcode(AcirOpcode::Arithmetic(&sub - &expr));
        self.push_opcode(AcirOpcode::Directive(Directive::Quotient {
            a: sub,
            b: Expression::from_field(two_s),
            q: q_witness,
            r: r_witness,
            predicate: None,
        }));

        self.range_constraint(r_witness, max_bits)?;
        self.range_constraint(q_witness, 1)?;
        Ok(q_witness)
    }
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn bit_size_u128(a: u128) -> u32 where {
    num_bits::<u128>() as u32 - a.leading_zeros()
}

fn bit_size_u32(a: u32) -> u32 where {
    num_bits::<u32>() as u32 - a.leading_zeros()
}
