use crate::binary_op;
use crate::binary_op::bound_check;
use crate::{AndGate, Evaluator, FieldElement, XorGate};
use crate::{Gate, Object};
use acvm::acir::native_types::{Arithmetic, Linear, Witness};

use super::RuntimeErrorKind;
use acvm::acir::circuit::gate::Directive;
use std::ops::Add;
#[derive(Clone, Copy, Debug)]
pub struct Integer {
    pub(crate) witness: Witness,
    pub(crate) num_bits: u32, //bit size of the integer type, e.g 32 for u32
    max_bits: u32, //maximum bit size of the witness, e.g 64 if you multiply 2 32 bits values without constraining the result to 32 bits.
}

// XXX: Most of the needed functionality seems to be to monitor the final num_bits and then constrain it.
// We can put this inside of the Analyser instead

impl Integer {
    pub fn from_witness_unconstrained(witness: Witness, num_bits: u32) -> Integer {
        Integer {
            witness,
            num_bits,
            max_bits: num_bits,
        }
    }

    pub fn from_witness_unconstrained_with_max(
        witness: Witness,
        num_bits: u32,
        max_bits: u32,
    ) -> Integer {
        Integer {
            witness,
            num_bits,
            max_bits,
        }
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
            if self.num_bits % 2 == 1 {
                // new witnesses; a is constrained to num_bits-1 and b is 1 bit
                let r_witness = evaluator.add_witness_to_cs();
                let b_witness = evaluator.add_witness_to_cs();
                evaluator.gates.push(Gate::Directive(Directive::Oddrange {
                    a: self.witness,
                    b: b_witness,
                    r: r_witness,
                    bit_size: self.num_bits,
                }));
                let r_int = Integer::from_witness_unconstrained(r_witness, self.num_bits - 1);
                r_int.constrain(evaluator)?;
                let b_int = Integer::from_witness_unconstrained(b_witness, 1);
                b_int.constrain(evaluator)?;
                //Add the constraint a = r + 2^N*b
                let mut f = FieldElement::from(2_i128);
                f = f.pow(&FieldElement::from((self.num_bits - 1) as i128));
                let res = Linear {
                    add_scale: FieldElement::zero(),
                    witness: b_witness,
                    mul_scale: f,
                }
                .add(Linear::from_witness(r_witness));
                let my_constraint = &res - &Arithmetic::from(Linear::from_witness(self.witness));
                evaluator.gates.push(Gate::Arithmetic(my_constraint));
            } else {
                evaluator
                    .gates
                    .push(Gate::Range(self.witness, self.num_bits));
            }
        }
        Ok(())
    }

    //This function is reducing 'self' to its bit size:
    //It returns a new integer c such that c = 'self' % 2^{bit size}
    pub fn truncate(&self, evaluator: &mut Evaluator) -> Result<Integer, RuntimeErrorKind> {
        if self.max_bits <= self.num_bits {
            return Ok(*self);
        }
        //1. Generate witnesses b,c
        let b_witness = evaluator.add_witness_to_cs();
        let c_witness = evaluator.add_witness_to_cs();
        evaluator.gates.push(Gate::Directive(Directive::Truncate {
            a: self.witness,
            b: b_witness,
            c: c_witness,
            bit_size: self.num_bits,
        }));
        let b_int = Integer::from_witness_unconstrained(b_witness, self.num_bits);
        let c_int = Integer::from_witness_unconstrained(c_witness, self.max_bits - self.num_bits);
        b_int.constrain(evaluator)?;
        c_int.constrain(evaluator)?;

        //2. Add the constraint a = b+2^Nc
        let mut f = FieldElement::from(2_i128);
        f = f.pow(&FieldElement::from(self.num_bits as i128));
        let res = Linear {
            add_scale: FieldElement::zero(),
            witness: c_witness,
            mul_scale: f,
        }
        .add(Linear::from_witness(b_witness));
        let my_constraint = &res - &Arithmetic::from(Linear::from_witness(self.witness));
        evaluator.gates.push(Gate::Arithmetic(my_constraint));
        Ok(b_int)
    }

    pub fn from_arithmetic(arith: Arithmetic, num_bits: u32, evaluator: &mut Evaluator) -> Integer {
        // We can only range constrain witness variables, so create an intermediate variable, constraint it to the arithmetic gate
        // then cast it as an integer
        let (intermediate, witness) = evaluator.create_intermediate_variable(arith.clone());

        let rhs_arith = Arithmetic::from(intermediate.linear().unwrap());
        evaluator.gates.push(Gate::Arithmetic(&arith - &rhs_arith));

        Integer::from_witness_unconstrained(witness, num_bits)
    }

    pub fn from_arithmetic_with_max_bits(
        arith: Arithmetic,
        num_bits: u32,
        max_bits: u32,
        evaluator: &mut Evaluator,
    ) -> Integer {
        // We can only range constrain witness variables, so create an intermediate variable, constraint it to the arithmetic gate
        // then cast it as an integer
        let (intermediate, witness) = evaluator.create_intermediate_variable(arith.clone());

        let rhs_arith = Arithmetic::from(intermediate.linear().unwrap());
        evaluator.gates.push(Gate::Arithmetic(&arith - &rhs_arith));

        Integer::from_witness_unconstrained_with_max(witness, num_bits, max_bits)
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

    pub fn from_object_with_max_bits(
        poly: Object,
        num_bits: u32,
        max_bits: u32,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        match poly {
            Object::Arithmetic(arith) => Ok(Integer::from_arithmetic_with_max_bits(
                arith, num_bits, max_bits, evaluator,
            )),
            Object::Linear(linear) => Ok(Integer::from_arithmetic_with_max_bits(
                linear.into(),
                num_bits,
                max_bits,
                evaluator,
            )),
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
        let (witness_rhs, num_bits, _max_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let message = format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }

        let b_obj = match witness_rhs.integer() {
            Some(integer) => integer,
            None => Integer::from_object(witness_rhs, self.num_bits, evaluator)?,
        };

        let (new_bits, a_wit, b_wit) = self
            .truncate_arguments(b_obj, evaluator, |x, y| u32::max(x, y) + 1)
            .unwrap();

        let res = binary_op::handle_add_op(
            Object::from_witness(a_wit),
            Object::from_witness(b_wit),
            evaluator,
        )?;

        Integer::from_object_with_max_bits(res, self.num_bits, new_bits, evaluator)
    }

    //Truncate a and/or b in case of overflow
    // In case the operation 'a op b' does overflow the field size, the function reduces a and/or b to their native bit size so that 'a op b' does not overflow
    // The closure 'op_bsize' returns the maximum number of bits of the result 'a op b' from the max bit size of a and b
    // For instance if op is 'x', then 'op_bsize' would be max(max bit size of a, max bit size of b)+1
    pub fn truncate_arguments(
        &self,
        b: Integer,
        evaluator: &mut Evaluator,
        op_bsize: fn(u32, u32) -> u32,
    ) -> Result<(u32, Witness, Witness), RuntimeErrorKind> {
        let mut result_bits = op_bsize(self.max_bits, b.max_bits);
        let mut a_wit = self.witness;
        let mut b_wit = b.witness;
        if result_bits >= FieldElement::max_num_bits() {
            if self.max_bits > b.max_bits {
                a_wit = self.truncate(evaluator).unwrap().witness;
                result_bits = op_bsize(self.num_bits, b.max_bits);
                if result_bits >= FieldElement::max_num_bits() {
                    b_wit = b.truncate(evaluator).unwrap().witness;
                    result_bits = op_bsize(self.num_bits, b.num_bits);
                }
            } else {
                b_wit = b.truncate(evaluator).unwrap().witness;
                result_bits = op_bsize(self.max_bits, b.num_bits);
                if result_bits >= FieldElement::max_num_bits() {
                    a_wit = self.truncate(evaluator).unwrap().witness;
                    result_bits = op_bsize(self.num_bits, b.num_bits);
                }
            }
            if result_bits >= FieldElement::max_num_bits() {
                let message = format!(
                    "Require big int implementation, the bit size too big for the field: {}, {}",
                    self.num_bits, b.num_bits
                );
                return Err(RuntimeErrorKind::Unimplemented(message));
            }
        }
        Ok((result_bits, a_wit, b_wit))
    }

    pub fn sub(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        let (witness_rhs, num_bits, max_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let err = RuntimeErrorKind::Spanless(format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            ));
            return Err(err);
        }
        let obj_rhs = Integer::from_witness_unconstrained_with_max(
            witness_rhs.witness().unwrap(),
            num_bits,
            max_bits,
        );
        let (new_bits, a_wit, b_wit) = self
            .truncate_arguments(obj_rhs, evaluator, |x, y| u32::max(x, y) + 1)
            .unwrap();

        let mut f = FieldElement::from(2_i128);
        f = f.pow(&FieldElement::from((new_bits - 1) as i128));
        let a_plus = Linear {
            mul_scale: FieldElement::one(),
            witness: a_wit,
            add_scale: f,
        };
        // Add a gate which subtracts both integers
        let res = binary_op::handle_sub_op(
            Object::Linear(a_plus),
            Object::from_witness(b_wit),
            evaluator,
        )?;
        Integer::from_object_with_max_bits(res, self.num_bits, new_bits, evaluator)
    }

    pub fn div(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        let b_object = poly
            .integer()
            .expect("expected the rhs to be an integer in division operator");
        //we need to truncate a and b
        let a = self.truncate(evaluator)?;
        let b = b_object.truncate(evaluator)?;

        // Create quotient and remainder witnesses
        let q_witness = evaluator.add_witness_to_cs();
        let r_witness = evaluator.add_witness_to_cs();
        let r_int = Integer::from_witness_unconstrained(r_witness, b.num_bits);
        let q_int = Integer::from_witness_unconstrained(q_witness, a.num_bits);
        evaluator.gates.push(Gate::Directive(Directive::Quotient {
            a: a.witness,
            b: b.witness,
            q: q_witness,
            r: r_witness,
        }));

        // Constraints quotient and remainder
        let b_copy = Object::Integer(Integer {
            witness: b.witness,
            num_bits: b.num_bits,
            max_bits: b.num_bits,
        });

        q_int.constrain(evaluator)?; //we need to bound q so we use the fact that q<=a
        bound_check::handle_less_than_op(Object::Integer(r_int), b_copy, evaluator)?; //r < b
                                                                                      // a-b*q-r = 0
        let res = Arithmetic::from(Linear::from_witness(a.witness));
        let eucl_div_constraint = &res
            - &Arithmetic {
                mul_terms: vec![(FieldElement::one(), b.witness, q_witness)],
                linear_combinations: vec![(FieldElement::one(), r_witness)],
                q_c: FieldElement::zero(),
            };
        evaluator.gates.push(Gate::Arithmetic(eucl_div_constraint));
        Ok(q_int)
    }

    pub fn safe_sub(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        let (witness_rhs, num_bits, _) = extract_witness_and_num_bits(self.num_bits, poly)?;

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
        //XXX: this creates many intermediate variables that are potentially not needed, cf. #128
        let res_object = Integer::from_object(res, self.num_bits, evaluator)?;
        // Constrain the result to be equal to an integer in range of 2^num_bits
        res_object.constrain(evaluator)?;
        Ok(res_object)
    }

    pub fn safe_add(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        // You can only add an integer from an integer and they must have the same number of bits
        let (witness_rhs, num_bits, _) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let err = RuntimeErrorKind::Spanless(format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            ));
            return Err(err);
        }
        let res =
            binary_op::handle_add_op(Object::from_witness(self.witness), witness_rhs, evaluator)?;

        let res_object = Integer::from_object(res, self.num_bits, evaluator)?;

        // Constrain the result to be equal to an integer in range of 2^num_bits
        res_object.constrain(evaluator)?;
        Ok(res_object)
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
            max_bits: self.num_bits,
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

        let (witness_rhs, num_bits, max_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;
        let obj_rhs = Integer::from_witness_unconstrained_with_max(
            witness_rhs.witness().unwrap(),
            num_bits,
            max_bits,
        );
        if self.num_bits != num_bits {
            let message = format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }

        let (new_bits, a_wit, b_wit) = self
            .truncate_arguments(obj_rhs, evaluator, |x, y| x + y)
            .unwrap();
        let res = binary_op::handle_mul_op(
            Object::from_witness(a_wit),
            Object::from_witness(b_wit),
            evaluator,
        )?;

        Integer::from_object_with_max_bits(res, self.num_bits, new_bits, evaluator)
    }

    pub fn safe_mul(
        &self,
        poly: Object,
        evaluator: &mut Evaluator,
    ) -> Result<Integer, RuntimeErrorKind> {
        // You can only mul an integer with another integer and they must have the same number of bits
        let (witness_rhs, num_bits, _max_bits) = extract_witness_and_num_bits(self.num_bits, poly)?;

        if self.num_bits != num_bits {
            let message = format!(
                "Both integers must have the same integer type. Expected u{}, got u{}",
                self.num_bits, num_bits
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }

        let res =
            binary_op::handle_mul_op(Object::from_witness(self.witness), witness_rhs, evaluator)?;

        let res_object = Integer::from_object(res, self.num_bits, evaluator)?;

        // Constrain the result to be equal to an integer in range of 2^num_bits
        res_object.constrain(evaluator)?;
        Ok(res_object)
    }
}

fn extract_witness_and_num_bits(
    num_bits: u32,
    poly: Object,
) -> Result<(Object, u32, u32), RuntimeErrorKind> {
    let (object, bits, max_bits) = match &poly {
        Object::Integer(integer_rhs) => (
            Object::from_witness(integer_rhs.witness),
            integer_rhs.num_bits,
            integer_rhs.max_bits,
        ),
        Object::Linear(_) => (poly, num_bits, num_bits), //TODO: we do not know the bit size
        Object::Constants(c) => (Object::Constants(*c), num_bits, num_bits), // XXX: Here since we know the value of constant, we could get how many bits it is and do static checks
        k => {
            let message = format!(
                "Woops expected an integer or a field element with known bit size, but got {:?}",
                k
            );
            return Err(RuntimeErrorKind::UnstructuredError { message });
        }
    };
    Ok((object, bits, max_bits))
}
