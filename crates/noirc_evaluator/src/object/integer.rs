use crate::binary_op;
use crate::{AndGate, Evaluator, FieldElement, XorGate};
use crate::{Gate, Object};
use acvm::acir::native_types::{Arithmetic, Linear, Witness};

use super::RuntimeErrorKind;

#[derive(Clone, Copy, Debug)]
pub struct Integer {
    pub(crate) witness: Witness,
    pub(crate) num_bits: u32,
}

// XXX: Most of the needed functionality seems to be to monitor the final num_bits and then constrain it.
// We can put this inside of the Analyser instead

impl Integer {
    pub fn from_witness_unconstrained(witness: Witness, num_bits: u32) -> Integer {
        Integer { witness, num_bits }
    }

    pub fn constrain(&self, evaluator: &mut Evaluator) -> Result<(), RuntimeErrorKind> {
        if self.num_bits == 1 {
            // Add a bool gate
            let x = Linear::from_witness(self.witness);
            let mut x_minus_one = Linear::from_witness(self.witness);
            x_minus_one.add_scale = -FieldElement::one();
            let bool_constraint = x_minus_one * x;

            evaluator.gates.push(Gate::Arithmetic(bool_constraint));
        } else if self.num_bits == FieldElement::max_num_bits() {
            // Don't apply any constraints if the range is for the maximum number of bits
            let message = format!("All Witnesses are by default u{}. Applying this type does not apply any constraints.",FieldElement::max_num_bits());
            return Err(RuntimeErrorKind::UnstructuredError { message });
        } else {
            // Note if the number of bits is odd, then Barretenberg will panic
            evaluator
                .gates
                .push(Gate::Range(self.witness, self.num_bits));
        }
        Ok(())
    }

    pub fn from_arithmetic(arith: Arithmetic, num_bits: u32, evaluator: &mut Evaluator) -> Integer {
        // We can only range constrain witness variables, so create an intermediate variable, constraint it to the arithmetic gate
        // then cast it as an integer
        let (intermediate, witness) = evaluator.create_intermediate_variable(arith.clone());

        let rhs_arith = Arithmetic::from(intermediate.linear().unwrap());
        evaluator.gates.push(Gate::Arithmetic(&arith - &rhs_arith));

        Integer::from_witness_unconstrained(witness, num_bits)
    }

    /// Constrains the integer to be equal to zero
    pub fn constrain_zero(&self, evaluator: &mut Evaluator) {
        let witness_linear = Linear::from_witness(self.witness);

        evaluator
            .gates
            .push(Gate::Arithmetic(witness_linear.into()))
    }

    pub fn from_object(
        poly: Object,
        num_bits: u32,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        match poly {
            Object::Arithmetic(arith) => Ok(Integer::from_arithmetic(arith, num_bits, evaluator)),
            Object::Linear(linear) => {
                Ok(Integer::from_arithmetic(linear.into(), num_bits, evaluator))
            }
            k => {
                let message = format!(
                    "tried to convert a {} into an integer. This is not possible.",
                    k.r#type()
                );
                Err(RuntimeErrorKind::UnstructuredError { message })
            }
        }
    }

    pub fn add(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        // You can only sub an integer from an integer and they must have the same number of bits
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let err = RuntimeErrorKind::Spanless(format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            ));
            return Err(err);
        }

        let res =
            binary_op::handle_add_op(Object::from_witness(self.witness), witness_rhs, evaluator)?;

        Integer::from_object(res, self.num_bits, evaluator)
    }
    pub fn sub(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let err = RuntimeErrorKind::Spanless(format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            ));
            return Err(err);
        }

        // Add a gate which subtracts both integers
        let res =
            binary_op::handle_sub_op(Object::from_witness(self.witness), witness_rhs, evaluator)?;

        // Constrain the result to be equal to an integer in range of 2^num_bits
        Integer::from_object(res, self.num_bits, evaluator)
    }

    pub fn logic(
        &self,
        rhs: Integer,
        is_xor_gate: bool,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        if self.num_bits != rhs.num_bits {
            let message = format!("Expected a u{} got u{}", self.num_bits, rhs.num_bits);
            return Err(RuntimeErrorKind::Spanless(message));
        }

        let result = evaluator.add_witness_to_cs();

        if is_xor_gate {
            evaluator.gates.push(Gate::Xor(XorGate {
                a: self.witness,
                b: rhs.witness,
                result,
                num_bits: self.num_bits,
            }));
        } else {
            evaluator.gates.push(Gate::And(AndGate {
                a: self.witness,
                b: rhs.witness,
                result,
                num_bits: self.num_bits,
            }));
        }

        // Note: The result is not constrained to be `self.num_bits` because the underlying proof system will
        // force the result to be equal to the correct result of a & b
        Ok(Integer {
            witness: result,
            num_bits: self.num_bits,
        })
    }
    pub fn xor(
        &self,
        rhs: Integer,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        self.logic(rhs, true, evaluator)
    }
    pub fn and(
        &self,
        rhs: Integer,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        self.logic(rhs, false, evaluator)
    }

    pub fn mul(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        // You can only mul an integer with another integer and they must have the same number of bits
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let message = format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }

        let res =
            binary_op::handle_mul_op(Object::from_witness(self.witness), witness_rhs, evaluator)?;

        Integer::from_object(res, self.num_bits, evaluator)
    }
}

fn extract_witness_and_num_bits(
    num_bits: u32,
    poly: Object,
) -> Result<(Object, u32), RuntimeErrorKind> {
    let (object, bits) = match &poly {
        Object::Integer(integer_rhs) => (
            Object::from_witness(integer_rhs.witness),
            integer_rhs.num_bits,
        ),
        Object::Linear(_) => (poly, num_bits),
        Object::Constants(c) => (Object::Constants(*c), num_bits), // XXX: Here since we know the value of constant, we could get how many bits it is and do static checks
        k => {
            let message = format!(
                "Woops expected an integer or a field element with known bit size, but got {:?}",
                k
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }
    };
    Ok((object, bits))
}
