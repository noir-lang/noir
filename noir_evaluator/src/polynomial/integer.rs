use crate::binary_op;
use crate::circuit::Witness;
use crate::polynomial::{arithmetic::Arithmetic, Linear, Polynomial};
use crate::Gate;

use crate::{AndGate, Environment, Evaluator, FieldElement, Signedness, Type, XorGate};

#[derive(Clone, Debug)]
pub struct Integer {
    pub(crate) witness: Witness,
    pub(crate) num_bits: u32,
}

// XXX: Most of the needed functionality seems to be to monitor the final num_bits and then constrain it.
// We can put this inside of the Analyser instead

impl Integer {
    pub fn from_witness(witness: Witness, num_bits: u32) -> Integer {
        Integer { witness, num_bits }
    }

    pub fn constrain(&self, evaluator: &mut Evaluator) {
        if self.num_bits == 1 {
            // Add a bool gate
            let x = Linear::from_witness(self.witness.clone());
            let mut x_minus_one = Linear::from_witness(self.witness.clone());
            x_minus_one.add_scale = -noir_field::FieldElement::one();
            let bool_constraint = &x_minus_one * &x;

            evaluator.gates.push(Gate::Arithmetic(bool_constraint));
        } else if self.num_bits == FieldElement::max_num_bits() {
            // Don't apply any constraints if the range is for the maximum number of bits
            panic!(
                "All Witnesses are by default u{}. Apply this type does not apply any constraints.",
                FieldElement::max_num_bits()
            );
        } else {
            // Note if the number of bits is odd, then barretenberg will panic
            evaluator
                .gates
                .push(Gate::Range(self.witness.clone(), self.num_bits));
        }
    }

    pub fn from_arithmetic(
        arith: Arithmetic,
        num_bits: u32,
        env: &mut Environment,
        evaluator: &mut Evaluator,
    ) -> Integer {
        // We can only range constrain witness variables, so create an intermediate variable, constraint it to the arithmetic gate
        // then cast it as an integer
        let (intermediate, witness) = evaluator.create_intermediate_variable(
            env,
            arith.clone(),
            Type::Integer(Signedness::Unsigned, num_bits),
        );

        let rhs_arith = Arithmetic::from(intermediate.linear().unwrap());
        evaluator.gates.push(Gate::Arithmetic(&arith - &rhs_arith));

        Integer::from_witness(witness, num_bits)
    }
    pub fn from_polynomial(
        poly: Polynomial,
        num_bits: u32,
        env: &mut Environment,
        evaluator: &mut Evaluator,
    ) -> Integer {
        match poly {
            Polynomial::Arithmetic(arith) => {
                Integer::from_arithmetic(arith, num_bits, env, evaluator)
            }
            Polynomial::Linear(linear) => {
                Integer::from_arithmetic(linear.into(), num_bits, env, evaluator)
            }
            k => panic!(
                "Error: Tried to convert a {:?} into an integer. This is not possible.",
                k
            ),
        }
    }

    pub fn add(
        &self,
        poly: Polynomial,
        env: &mut Environment,
        evaluator: &mut Evaluator,
    ) -> Integer {
        // You can only sub an integer from an integer and they must have the same number of bits
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly);

        assert_eq!(
            self.num_bits, num_bits,
            "Both integers must have the same integer type. expected u{}, got u{}",
            self.num_bits, num_bits
        );

        let res = binary_op::handle_add_op(
            Polynomial::from_witness(self.witness.clone()),
            witness_rhs,
            env,
            evaluator,
        );

        Integer::from_polynomial(res, self.num_bits, env, evaluator)
    }
    pub fn sub(
        &self,
        poly: Polynomial,
        env: &mut Environment,
        evaluator: &mut Evaluator,
    ) -> Integer {
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly);

        assert_eq!(
            self.num_bits, num_bits,
            "Both integers must have the same integer type. Expected u{}, got u{}",
            self.num_bits, num_bits
        );

        // Add a gate which subtracts both integers
        let res = binary_op::handle_sub_op(
            Polynomial::from_witness(self.witness.clone()),
            witness_rhs,
            env,
            evaluator,
        );

        // Constrain the result to be equal to an integer in range of 2^num_bits
        Integer::from_polynomial(res, self.num_bits, env, evaluator)
    }

    pub fn logic(
        &self,
        rhs: Integer,
        env: &mut Environment,
        is_xor_gate: bool,
        evaluator: &mut Evaluator,
    ) -> Integer {
        if self.num_bits != rhs.num_bits {
            panic!("Expected a u{} got u{}", self.num_bits, rhs.num_bits);
        }

        let op_str = if is_xor_gate { "xor" } else { "and" };

        // XXX: We need to create a better function for fresh variables
        let result_str = format!("{}_{}", op_str, evaluator.get_unique_value(),);
        let result = evaluator.store_witness(result_str.clone(), Type::Witness);

        if is_xor_gate {
            evaluator.gates.push(Gate::Xor(XorGate {
                a: self.witness.clone(),
                b: rhs.witness,
                result: result.clone(),
                num_bits: self.num_bits,
            }));
        } else {
            evaluator.gates.push(Gate::And(AndGate {
                a: self.witness.clone(),
                b: rhs.witness,
                result: result.clone(),
                num_bits: self.num_bits,
            }));
        }

        // Note: The result is not constrained to be `self.num_bits` because the underlying proof system will
        // make force the result ot be equal to the correct result of a & b
        Integer {
            witness: result,
            num_bits: self.num_bits,
        }
    }
    pub fn xor(&self, rhs: Integer, env: &mut Environment, evaluator: &mut Evaluator) -> Integer {
        self.logic(rhs, env, true, evaluator)
    }
    pub fn and(&self, rhs: Integer, env: &mut Environment, evaluator: &mut Evaluator) -> Integer {
        self.logic(rhs, env, false, evaluator)
    }

    pub fn mul(
        &self,
        poly: Polynomial,
        env: &mut Environment,
        evaluator: &mut Evaluator,
    ) -> Integer {
        // You can only mul an integer with another integer and they must have the same number of bits
        let (witness_rhs, num_bits) = extract_witness_and_num_bits(self.num_bits, poly);

        assert_eq!(
            self.num_bits, num_bits,
            "Both integers must have the same integer type. expected u{}, got u{}",
            self.num_bits, num_bits
        );

        let res = binary_op::handle_mul_op(
            Polynomial::from_witness(self.witness.clone()),
            witness_rhs,
            env,
            evaluator,
        );

        Integer::from_polynomial(res, self.num_bits + num_bits, env, evaluator)
    }
}

fn extract_witness_and_num_bits(num_bits: u32, poly: Polynomial) -> (Polynomial, u32) {
    match &poly {
        Polynomial::Integer(integer_rhs) => (
            Polynomial::from_witness(integer_rhs.witness.clone()),
            integer_rhs.num_bits,
        ),
        Polynomial::Linear(_) => (poly, num_bits),
        Polynomial::Constants(c) => (Polynomial::Constants(*c), num_bits), // XXX: Here since we know the value of constant, we could get how many bits it is and do static checks
        k => panic!("Woops expected an integer, but got {:?}", k),
    }
}
