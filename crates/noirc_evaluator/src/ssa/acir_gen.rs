use super::mem::{MemArray, Memory};
use super::node::{Instruction, Operation};
use acvm::FieldElement;

use super::node::NodeId;

use num_traits::{One, Zero};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Mul, Neg};
//use crate::acir::native_types::{Arithmetic, Witness};
use crate::ssa::context::SsaContext;
use crate::ssa::node::Node;
use crate::ssa::{mem, node};
use crate::Evaluator;
use crate::Gate;
use crate::RuntimeErrorKind;
use acvm::acir::circuit::gate::Directive;
use acvm::acir::native_types::{Arithmetic, Linear, Witness};
use num_bigint::BigUint;
use std::convert::TryInto;

#[derive(Default)]
pub struct Acir {
    pub arith_cache: HashMap<NodeId, InternalVar>,
    pub memory_map: HashMap<u32, InternalVar>, //maps memory adress to expression
    pub memory_witness: HashMap<u32, Vec<Witness>>, //map arrays to their witness...temporary
}

#[derive(Default, Clone, Debug)]
pub struct InternalVar {
    expression: Arithmetic,
    //value: FieldElement,     //not used for now
    witness: Option<Witness>,
    id: Option<NodeId>,
}
impl InternalVar {
    pub fn is_equal(&self, b: &InternalVar) -> bool {
        (self.id.is_some() && self.id == b.id)
            || (self.witness.is_some() && self.witness == b.witness)
            || self.expression == b.expression
    }

    fn new(expression: Arithmetic, witness: Option<Witness>, id: NodeId) -> InternalVar {
        InternalVar {
            expression,
            witness,
            id: Some(id),
        }
    }

    pub fn to_const(&self) -> Option<FieldElement> {
        if self.expression.mul_terms.is_empty() && self.expression.linear_combinations.is_empty() {
            return Some(self.expression.q_c);
        }
        None
    }
}

impl From<Arithmetic> for InternalVar {
    fn from(arith: Arithmetic) -> InternalVar {
        let w = is_unit(&arith);
        InternalVar {
            expression: arith,
            witness: w,
            id: None,
        }
    }
}

impl From<Witness> for InternalVar {
    fn from(w: Witness) -> InternalVar {
        InternalVar {
            expression: from_witness(w),
            witness: Some(w),
            id: None,
        }
    }
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a nodeobj as an arithmetic expression
    fn substitute(
        &mut self,
        id: NodeId,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        if self.arith_cache.contains_key(&id) {
            return self.arith_cache[&id].clone();
        }
        let var = match ctx.try_get_node(id) {
            Some(node::NodeObj::Const(c)) => {
                let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()); //TODO const should be a field
                let expr = Arithmetic {
                    mul_terms: Vec::new(),
                    linear_combinations: Vec::new(),
                    q_c: f_value, //TODO handle other types
                };
                InternalVar::new(expr, None, id)
            }
            Some(node::NodeObj::Obj(v)) => {
                let w = v.witness.unwrap_or_else(|| evaluator.add_witness_to_cs());
                let expr = Arithmetic::from(&w);
                InternalVar::new(expr, Some(w), id)
            }
            _ => {
                let w = evaluator.add_witness_to_cs();
                let expr = Arithmetic::from(&w);
                InternalVar::new(expr, Some(w), id)
            }
        };

        self.arith_cache.insert(id, var.clone());
        var
    }

    pub fn evaluate_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) {
        if ins.operator == Operation::Nop {
            return;
        }
        let l_c = self.substitute(ins.lhs, evaluator, ctx);
        let r_c = self.substitute(ins.rhs, evaluator, ctx);
        let mut output = match ins.operator {
            Operation::Add | Operation::SafeAdd => {
                InternalVar::from(add(&l_c.expression, FieldElement::one(), &r_c.expression))
            }
            Operation::Sub | Operation::SafeSub => {
                //we need the type of rhs and its max value, then:
                //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                let bit_size = ctx[ins.rhs].size_in_bits();
                let r_big = BigUint::one() << bit_size;
                let mut k = &ins.max_value / &r_big;
                if &ins.max_value % &r_big != BigUint::zero() {
                    k = &k + BigUint::one();
                }
                k = &k * r_big;
                let f = FieldElement::from_be_bytes_reduce(&k.to_bytes_be());
                let mut output = add(
                    &l_c.expression,
                    FieldElement::from(-1_i128),
                    &r_c.expression,
                );
                output.q_c += f;
                output.into()
            }
            Operation::Mul | Operation::SafeMul => {
                InternalVar::from(evaluate_mul(&l_c, &r_c, evaluator))
            }
            Operation::Udiv => {
                let (q_wit, _) = evaluate_udiv(&l_c, &r_c, evaluator);
                InternalVar::from(q_wit)
            }
            Operation::Sdiv => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).0),
            Operation::Urem => {
                let (_, r_wit) = evaluate_udiv(&l_c, &r_c, evaluator);
                InternalVar::from(r_wit)
            }
            Operation::Srem => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).1),
            Operation::Div => InternalVar::from(mul(
                &l_c.expression,
                &from_witness(evaluate_inverse(&r_c.expression, evaluator)),
            )),
            Operation::Eq => {
                InternalVar::from(self.evaluate_eq(ins.lhs, ins.rhs, &l_c, &r_c, ctx, evaluator))
            }
            Operation::Ne => {
                InternalVar::from(self.evaluate_neq(ins.lhs, ins.rhs, &l_c, &r_c, ctx, evaluator))
            }
            Operation::Ugt => todo!(),
            Operation::Uge => todo!(),
            Operation::Ult => todo!(),
            Operation::Ule => todo!(),
            Operation::Sgt => todo!(),
            Operation::Sge => todo!(),
            Operation::Slt => todo!(),
            Operation::Sle => todo!(),
            Operation::Lt => todo!(),
            Operation::Gt => todo!(),
            Operation::Lte => evaluate_cmp(&l_c, &r_c, 32, evaluator).into(), //TODO we need the bit_size from the instruction
            Operation::Gte => evaluate_cmp(&r_c, &l_c, 32, evaluator).into(), //TODO we need the bit_size from the instruction
            Operation::And => {
                InternalVar::from(evaluate_and(l_c, r_c, ins.res_type.bits(), evaluator))
            }
            Operation::Not => todo!(),
            Operation::Or => todo!(),
            Operation::Xor => {
                InternalVar::from(evaluate_xor(l_c, r_c, ins.res_type.bits(), evaluator))
            }
            Operation::Cast => l_c.clone(),
            Operation::Assign
            | Operation::Jne
            | Operation::Jeq
            | Operation::Jmp
            | Operation::Phi => {
                todo!("invalid instruction");
            }
            Operation::Truncate => {
                assert!(is_const(&r_c.expression));
                InternalVar::from(evaluate_truncate(
                    l_c,
                    r_c.expression.q_c.to_u128().try_into().unwrap(),
                    ins.bit_size,
                    evaluator,
                ))
            }
            Operation::Intrinsic(_) => todo!("Intrinsic"),
            Operation::Nop => InternalVar::default(),
            Operation::Constrain(op) => match op {
                node::ConstrainOp::Eq => {
                    InternalVar::from(self.equalize(ins.lhs, ins.rhs, &l_c, &r_c, ctx, evaluator))
                }
                node::ConstrainOp::Neq => {
                    InternalVar::from(self.distinct(ins.lhs, ins.rhs, &l_c, &r_c, ctx, evaluator))
                }
            },
            Operation::Load(array_idx) => {
                //retrieves the value from the map if address is known at compile time:
                //address = l_c and should be constant
                if let Some(val) = l_c.to_const() {
                    let address = mem::Memory::as_u32(val);
                    if self.memory_map.contains_key(&address) {
                        InternalVar::from(self.memory_map[&address].expression.clone())
                    } else {
                        //if not found, then it must be a witness (else it is non-initialised memory)
                        let array = &ctx.mem.arrays[array_idx as usize];
                        let index = (address - array.adr) as usize;
                        let w = if array.witness.len() > index {
                            array.witness[index]
                        } else {
                            self.memory_witness[&array_idx][index]
                        };
                        InternalVar::from(w)
                    }
                } else {
                    todo!("dynamic arrays are not implemented yet");
                }
            }

            Operation::Store(_) => {
                //maps the address to the rhs if address is known at compile time
                if let Some(val) = r_c.to_const() {
                    let address = mem::Memory::as_u32(val);
                    self.memory_map.insert(address, l_c);
                    dbg!(&self.memory_map);
                    //we do not generate constraint, so no output.
                    InternalVar::default()
                } else {
                    todo!("dynamic arrays are not implemented yet");
                }
            }
        };
        output.id = Some(ins.id);
        self.arith_cache.insert(ins.id, output);
    }

    pub fn print_circuit(gates: &[Gate]) {
        for gate in gates {
            println!("{:?}", gate);
        }
    }

    //Load array values into InternalVars
    //If create_witness is true, we create witnesses for values that do not have witness
    pub fn load_array(
        &mut self,
        array: &MemArray,
        array_index: u32,
        create_witness: bool,
        evaluator: &mut Evaluator,
    ) -> Vec<InternalVar> {
        (0..array.len)
            .map(|i| {
                let address = array.adr + i;
                if let Some(memory) = self.memory_map.get_mut(&address) {
                    if create_witness && memory.witness.is_none() {
                        let (_, w) =
                            evaluator.create_intermediate_variable(memory.expression.clone());
                        self.memory_map.get_mut(&address).unwrap().witness = Some(w);
                    }
                    self.memory_map[&address].clone()
                } else if let Some(memory) = self.memory_witness.get(&array_index) {
                    let w = memory[i as usize];
                    w.into()
                } else {
                    let w = array.witness[i as usize];
                    w.into()
                }
            })
            .collect()
    }

    pub fn evaluate_neq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Arithmetic {
        if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
            let array_a = &ctx.mem.arrays[a as usize];
            let array_b = &ctx.mem.arrays[b as usize];

            if array_a.len == array_b.len {
                let mut x =
                    InternalVar::from(self.zero_eq_array_sum(array_a, a, array_b, b, evaluator));
                x.witness = Some(generate_witness(&x, evaluator));
                from_witness(evaluate_zero_equality(&x, evaluator))
            } else {
                //If length are different, then the arrays are different
                Arithmetic {
                    mul_terms: Vec::new(),
                    linear_combinations: Vec::new(),
                    q_c: FieldElement::one(),
                }
            }
        } else {
            let mut x = InternalVar::from(subtract(
                &l_c.expression,
                FieldElement::one(),
                &r_c.expression,
            ));
            x.witness = Some(generate_witness(&x, evaluator));
            from_witness(evaluate_zero_equality(&x, evaluator))
        }
    }

    pub fn evaluate_eq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Arithmetic {
        subtract(
            &Arithmetic {
                mul_terms: Vec::new(),
                linear_combinations: Vec::new(),
                q_c: FieldElement::one(),
            },
            FieldElement::one(),
            &self.evaluate_neq(lhs, rhs, l_c, r_c, ctx, evaluator),
        )
    }

    //Constraint lhs to be different than rhs
    pub fn distinct(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Arithmetic {
        if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
            let array_a = &ctx.mem.arrays[a as usize];
            let array_b = &ctx.mem.arrays[b as usize];
            //If length are different, then the arrays are different
            if array_a.len == array_b.len {
                let sum = self.zero_eq_array_sum(array_a, a, array_b, b, evaluator);
                evaluate_inverse(&sum, evaluator);
            }
        } else {
            let diff = subtract(&l_c.expression, FieldElement::one(), &r_c.expression);
            evaluate_inverse(&diff, evaluator);
        }
        Arithmetic::default()
    }

    //Constraint lhs to be equal to rhs
    pub fn equalize(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Arithmetic {
        if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
            let a_values = self.load_array(&ctx.mem.arrays[a as usize], a, false, evaluator);
            let b_values = self.load_array(&ctx.mem.arrays[b as usize], b, false, evaluator);
            assert!(a_values.len() == b_values.len());
            for (a_iter, b_iter) in a_values.into_iter().zip(b_values) {
                let array_diff =
                    subtract(&a_iter.expression, FieldElement::one(), &b_iter.expression);
                evaluator.gates.push(Gate::Arithmetic(array_diff));
            }
            Arithmetic::default()
        } else {
            let output = add(
                &l_c.expression,
                FieldElement::from(-1_i128),
                &r_c.expression,
            );
            evaluator.gates.push(Gate::Arithmetic(output.clone()));
            output
        }
    }

    //Generates gates for the expression: \sum_i(zero_eq(A[i]-B[i]))
    //N.b. We assumes the lenghts of a and b are the same but it is not checked inside the function.
    fn zero_eq_array_sum(
        &mut self,
        a: &MemArray,
        a_idx: u32,
        b: &MemArray,
        b_idx: u32,
        evaluator: &mut Evaluator,
    ) -> Arithmetic {
        let mut sum = Arithmetic::default();

        let a_values = self.load_array(a, a_idx, false, evaluator);
        let b_values = self.load_array(b, b_idx, false, evaluator);

        for (a_iter, b_iter) in a_values.into_iter().zip(b_values) {
            let diff_expr = subtract(&a_iter.expression, FieldElement::one(), &b_iter.expression);

            let diff_witness = evaluator.add_witness_to_cs();
            let diff_var = InternalVar {
                //in cache??
                expression: diff_expr.clone(),
                witness: Some(diff_witness),
                id: None,
            };
            evaluator.gates.push(Gate::Arithmetic(subtract(
                &diff_expr,
                FieldElement::one(),
                &from_witness(diff_witness),
            )));
            //TODO: avoid creating witnesses for diff
            sum = add(
                &sum,
                FieldElement::one(),
                &from_witness(evaluate_zero_equality(&diff_var, evaluator)),
            );
        }
        sum
    }
}

//Returns 1 if lhs <= rhs
pub fn evaluate_cmp(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Witness {
    //TODO use quad_decomposition gate for barretenberg
    let sub_expr = subtract(&lhs.expression, FieldElement::one(), &rhs.expression);
    let bits = split(&sub_expr, bit_size + 1, evaluator);
    bits[bit_size as usize]
}

//Performs bit decomposition
pub fn split(lhs: &Arithmetic, bit_size: u32, evaluator: &mut Evaluator) -> Vec<Witness> {
    let mut bits = Arithmetic::default();
    let mut two_pow = FieldElement::one();
    let two = FieldElement::from(2_i128);
    let mut result = Vec::new();
    for _ in 0..bit_size {
        let bit_witness = evaluator.add_witness_to_cs();
        result.push(bit_witness);
        bits = add(&bits, two_pow, &from_witness(bit_witness));
        two_pow = two_pow.mul(two);
    }
    evaluator
        .gates
        .push(Gate::Arithmetic(subtract(lhs, FieldElement::one(), &bits)));
    //todo witness values for solver
    result
}

pub fn evaluate_and(
    lhs: InternalVar,
    rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Arithmetic {
    let result = evaluator.add_witness_to_cs();
    let a_witness = generate_witness(&lhs, evaluator);
    let b_witness = generate_witness(&rhs, evaluator);
    //TODO checks the cost of the gate vs bit_size (cf. #164)
    evaluator
        .gates
        .push(Gate::And(acvm::acir::circuit::gate::AndGate {
            a: a_witness,
            b: b_witness,
            result,
            num_bits: bit_size,
        }));
    Arithmetic::from(Linear::from_witness(result))
}

pub fn evaluate_xor(
    lhs: InternalVar,
    rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Witness {
    let result = evaluator.add_witness_to_cs();

    let a_witness = generate_witness(&lhs, evaluator);
    let b_witness = generate_witness(&rhs, evaluator);
    //TODO checks the cost of the gate vs bit_size (cf. #164)
    evaluator
        .gates
        .push(Gate::Xor(acvm::acir::circuit::gate::XorGate {
            a: a_witness,
            b: b_witness,
            result,
            num_bits: bit_size,
        }));
    result
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
pub fn evaluate_truncate(
    lhs: InternalVar,
    rhs: u32,
    max_bits: u32,
    evaluator: &mut Evaluator,
) -> Witness {
    assert!(max_bits > rhs);
    //1. Generate witnesses a,b,c
    //TODO: we should truncate the arithmetic expression (and so avoid having to create a witness)
    // if lhs is not a witness, but this requires a new truncate directive...TODO
    let a_witness = generate_witness(&lhs, evaluator);
    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();
    evaluator.gates.push(Gate::Directive(Directive::Truncate {
        a: a_witness,
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));

    range_constraint(b_witness, rhs, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    }); //TODO propagate the error using ?
    range_constraint(c_witness, max_bits - rhs, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    });

    //2. Add the constraint a = b+2^Nc
    let mut f = FieldElement::from(2_i128);
    f = f.pow(&FieldElement::from(rhs as i128));
    let b_arith = from_witness(b_witness);
    let c_arith = from_witness(c_witness);
    let res = add(&b_arith, f, &c_arith); //b+2^Nc
    let a = &Arithmetic::from(Linear::from_witness(a_witness));
    let my_constraint = add(&res, -FieldElement::one(), a);
    evaluator.gates.push(Gate::Arithmetic(my_constraint));
    b_witness
}

pub fn generate_witness(lhs: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    if let Some(witness) = lhs.witness {
        return witness;
    }

    if is_const(&lhs.expression) {
        todo!("Panic");
    }
    if lhs.expression.mul_terms.is_empty() && lhs.expression.linear_combinations.len() == 1 {
        //TODO check if this case can be optimised
    }
    let (_, w) = evaluator.create_intermediate_variable(lhs.expression.clone());
    w //TODO  set lhs.witness = w
}

pub fn evaluate_mul(lhs: &InternalVar, rhs: &InternalVar, evaluator: &mut Evaluator) -> Arithmetic {
    if is_const(&lhs.expression) {
        return &rhs.expression * &lhs.expression.q_c;
    }
    if is_const(&rhs.expression) {
        return &lhs.expression * &rhs.expression.q_c;
    }
    //No multiplicative term
    if lhs.expression.mul_terms.is_empty() && rhs.expression.mul_terms.is_empty() {
        return mul(&lhs.expression, &rhs.expression);
    }
    //Generate intermediate variable
    //create new witness a and a gate: a = lhs
    let a = evaluator.add_witness_to_cs();
    evaluator
        .gates
        .push(Gate::Arithmetic(&lhs.expression - &Arithmetic::from(&a)));
    //create new witness b and gate b = rhs
    let mut b = a;
    if !lhs.is_equal(rhs) {
        b = evaluator.add_witness_to_cs();
        evaluator
            .gates
            .push(Gate::Arithmetic(&rhs.expression - &Arithmetic::from(&b)));
    }

    //return arith(mul=a*b)
    mul(&Arithmetic::from(&a), &Arithmetic::from(&b)) //TODO  &lhs.expression * &rhs.expression
}

pub fn evaluate_udiv(
    lhs: &InternalVar,
    rhs: &InternalVar,
    evaluator: &mut Evaluator,
) -> (Witness, Witness) {
    //a = q*b+r, a= lhs, et b= rhs
    //result = q
    //n.b a et b MUST have proper bit size
    //we need to know a bit size (so that q has the same)
    //generate witnesses

    //TODO: can we handle an arithmetic and not create a witness for a and b?
    let a_witness = generate_witness(lhs, evaluator); //TODO we should set lhs.witness = a.witness and lhs.expression= 1*w

    //TODO: can we handle an arithmetic and not create a witness for a and b?
    let b_witness = generate_witness(rhs, evaluator);

    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();

    //TODO not in master...
    evaluator.gates.push(Gate::Directive(Directive::Quotient {
        a: a_witness,
        b: b_witness,
        q: q_witness,
        r: r_witness,
    }));
    //r<b
    let r_expr = Arithmetic::from(Linear::from_witness(r_witness));
    let r_var = InternalVar {
        expression: r_expr,
        witness: Some(r_witness),
        id: None,
    };
    bound_check(&r_var, rhs, true, 32, evaluator); //TODO bit size! should be max(a.bit, b.bit)
                                                   //range check q<=a
    range_constraint(q_witness, 32, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    }); //todo bit size should be a.bits
        // a-b*q-r = 0
    let div_eucl = add(
        &lhs.expression,
        -FieldElement::one(),
        &Arithmetic {
            mul_terms: vec![(FieldElement::one(), b_witness, q_witness)],
            linear_combinations: vec![(FieldElement::one(), r_witness)],
            q_c: FieldElement::zero(),
        },
    );

    evaluator.gates.push(Gate::Arithmetic(div_eucl));
    (q_witness, r_witness)
}

//Zero Equality gate: returns 1 if x is not null and 0 else
pub fn evaluate_zero_equality(x: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    let x_witness = x.witness.unwrap(); //todo we need a witness because of the directive, but we should use an expression

    let m = evaluator.add_witness_to_cs(); //'inverse' of x
    evaluator.gates.push(Gate::Directive(Directive::Invert {
        x: x_witness,
        result: m,
    }));

    //y=x*m         y is 1 if x is not null, and 0 else
    let y_witness = evaluator.add_witness_to_cs();
    evaluator.gates.push(Gate::Arithmetic(Arithmetic {
        mul_terms: vec![(FieldElement::one(), x_witness, m)],
        linear_combinations: vec![(FieldElement::one().neg(), y_witness)],
        q_c: FieldElement::zero(),
    }));

    //x=y*x
    let y_expr = from_witness(y_witness);
    let xy = mul(&x.expression, &y_expr);
    evaluator.gates.push(Gate::Arithmetic(subtract(
        &xy,
        FieldElement::one(),
        &from_witness(x_witness),
    )));
    y_witness
}

/// Creates a new witness and constrains it to be the inverse of x
pub fn evaluate_inverse(x: &Arithmetic, evaluator: &mut Evaluator) -> Witness {
    // Create a fresh witness - n.b we could check if x is constant or not
    let inverse_witness = evaluator.add_witness_to_cs();
    let inverse_expr = from_witness(inverse_witness);
    //x*inverse = 1
    Arithmetic::default();
    evaluator.gates.push(Gate::Arithmetic(add(
        &mul(x, &inverse_expr),
        FieldElement::one(),
        &Arithmetic {
            mul_terms: Vec::new(),
            linear_combinations: Vec::new(),
            q_c: FieldElement::from(-1_i128),
        },
    )));
    inverse_witness
}

pub fn evaluate_sdiv(
    _lhs: &InternalVar,
    _rhs: &InternalVar,
    _evaluator: &mut Evaluator,
) -> (Arithmetic, Arithmetic) {
    todo!(); //TODO
}

pub fn is_const(expr: &Arithmetic) -> bool {
    expr.mul_terms.is_empty() && expr.linear_combinations.is_empty()
}

//a*b
pub fn mul(a: &Arithmetic, b: &Arithmetic) -> Arithmetic {
    if !(a.mul_terms.is_empty() && b.mul_terms.is_empty()) {
        todo!("PANIC");
    }

    let mut output = Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: Vec::new(),
        q_c: FieldElement::zero(),
    };

    //TODO to optimise...
    for lc in &a.linear_combinations {
        let single = single_mul(lc.1, b);
        output = add(&output, lc.0, &single);
    }

    //linear terms
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        let coef_a = b.q_c * a.linear_combinations[i1].0;
        let coef_b = a.q_c * b.linear_combinations[i2].0;
        match a.linear_combinations[i1]
            .1
            .cmp(&b.linear_combinations[i2].1)
        {
            Ordering::Greater => {
                if coef_b != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_b, b.linear_combinations[i2].1));
                }
                if i2 + 1 >= b.linear_combinations.len() {
                    i1 += 1;
                } else {
                    i2 += 1;
                }
            }
            Ordering::Less => {
                if coef_a != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_a, a.linear_combinations[i1].1));
                }
                if i1 + 1 >= a.linear_combinations.len() {
                    i2 += 1;
                } else {
                    i1 += 1;
                }
            }
            Ordering::Equal => {
                if coef_a + coef_b != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_a + coef_b, a.linear_combinations[i1].1));
                }
                if (i1 + 1 >= a.linear_combinations.len())
                    && (i2 + 1 >= b.linear_combinations.len())
                {
                    i1 += 1;
                    i2 += 1;
                } else {
                    if i1 + 1 < a.linear_combinations.len() {
                        i1 += 1;
                    }
                    if i2 + 1 < a.linear_combinations.len() {
                        i2 += 1;
                    }
                }
            }
        }
    }
    //Constant term:
    output.q_c = a.q_c * b.q_c;
    output
}

// returns a - k*b
pub fn subtract(a: &Arithmetic, k: FieldElement, b: &Arithmetic) -> Arithmetic {
    add(a, k.neg(), b)
}

// returns a + k*b
pub fn add(a: &Arithmetic, k: FieldElement, b: &Arithmetic) -> Arithmetic {
    let mut output = Arithmetic::default();

    //linear combinations
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        match a.linear_combinations[i1]
            .1
            .cmp(&b.linear_combinations[i2].1)
        {
            Ordering::Greater => {
                let coef = b.linear_combinations[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef, b.linear_combinations[i2].1));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.linear_combinations.push(a.linear_combinations[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coef = a.linear_combinations[i1].0 + b.linear_combinations[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef, a.linear_combinations[i1].1));
                }
                i2 += 1;
                i1 += 1;
            }
        }
    }
    while i1 < a.linear_combinations.len() {
        output.linear_combinations.push(a.linear_combinations[i1]);
        i1 += 1;
    }
    while i2 < b.linear_combinations.len() {
        let coef = b.linear_combinations[i2].0 * k;
        if coef != FieldElement::zero() {
            output
                .linear_combinations
                .push((coef, b.linear_combinations[i2].1));
        }
        i2 += 1;
    }

    //mul terms

    i1 = 0; //a
    i2 = 0; //b

    while i1 < a.mul_terms.len() && i2 < b.mul_terms.len() {
        match (a.mul_terms[i1].1, a.mul_terms[i1].2).cmp(&(b.mul_terms[i2].1, b.mul_terms[i2].2)) {
            Ordering::Greater => {
                let coef = b.mul_terms[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .mul_terms
                        .push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.mul_terms.push(a.mul_terms[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coef = a.mul_terms[i1].0 + b.mul_terms[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .mul_terms
                        .push((coef, a.mul_terms[i1].1, a.mul_terms[i1].2));
                }
                i2 += 1;
                i1 += 1;
            }
        }
    }
    while i1 < a.mul_terms.len() {
        output.mul_terms.push(a.mul_terms[i1]);
        i1 += 1;
    }

    while i2 < b.mul_terms.len() {
        let coef = b.mul_terms[i2].0 * k;
        if coef != FieldElement::zero() {
            output
                .mul_terms
                .push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
        }
        i2 += 1;
    }

    output.q_c = a.q_c + k * b.q_c;
    output
}

// returns w*b.linear_combinations
pub fn single_mul(w: Witness, b: &Arithmetic) -> Arithmetic {
    let mut output = Arithmetic::default();
    let mut i1 = 0;
    while i1 < b.linear_combinations.len() {
        if (w, b.linear_combinations[i1].1) < (b.linear_combinations[i1].1, w) {
            output
                .mul_terms
                .push((b.linear_combinations[i1].0, w, b.linear_combinations[i1].1));
        } else {
            output
                .mul_terms
                .push((b.linear_combinations[i1].0, b.linear_combinations[i1].1, w));
        }
        i1 += 1;
    }
    output
}

pub fn boolean(witness: Witness) -> Arithmetic {
    Arithmetic {
        mul_terms: vec![(FieldElement::one(), witness, witness)],
        linear_combinations: vec![(-FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

//contrain witness a to be num_bits-size integer, i.e between 0 and 2^num_bits-1
pub fn range_constraint(
    witness: Witness,
    num_bits: u32,
    evaluator: &mut Evaluator,
) -> Result<(), RuntimeErrorKind> {
    if num_bits == 1 {
        // Add a bool gate
        let bool_constraint = boolean(witness);
        evaluator.gates.push(Gate::Arithmetic(bool_constraint));
    } else if num_bits == FieldElement::max_num_bits() {
        // Don't apply any constraints if the range is for the maximum number of bits
        let message = format!(
            "All Witnesses are by default u{}. Applying this type does not apply any constraints.",
            FieldElement::max_num_bits()
        );
        return Err(RuntimeErrorKind::UnstructuredError { message });
    } else if num_bits % 2 == 1 {
        // Note if the number of bits is odd, then Barretenberg will panic
        // new witnesses; r is constrained to num_bits-1 and b is 1 bit
        let r_witness = evaluator.add_witness_to_cs();
        let b_witness = evaluator.add_witness_to_cs();
        evaluator.gates.push(Gate::Directive(Directive::Oddrange {
            a: witness,
            b: b_witness,
            r: r_witness,
            bit_size: num_bits,
        }));
        range_constraint(r_witness, num_bits - 1, evaluator).unwrap_or_else(|err| {
            dbg!(err);
        });
        range_constraint(b_witness, 1, evaluator).unwrap_or_else(|err| {
            dbg!(err);
        });
        //Add the constraint a = r + 2^N*b
        let mut f = FieldElement::from(2_i128);
        f = f.pow(&FieldElement::from((num_bits - 1) as i128));
        let res = add(&from_witness(r_witness), f, &from_witness(b_witness));
        let my_constraint = add(&res, -FieldElement::one(), &from_witness(witness));
        evaluator.gates.push(Gate::Arithmetic(my_constraint));
    } else {
        evaluator.gates.push(Gate::Range(witness, num_bits));
    }

    Ok(())
}

// Generate constraints that are satisfied iff
// a < b , when strict is true, or
// a <= b, when strict is false
// bits is the bit size of a and b (or an upper bound of the bit size)
///////////////
// a<=b is done by constraining b-a to a bit size of 'bits':
// if a<=b, 0 <= b-a <= b < 2^bits
// if a>b, b-a = p+b-a > p-2^bits >= 2^bits  (if log(p) >= bits + 1)
// n.b: we do NOT check here that a and b are indeed 'bits' size
// a < b <=> a+1<=b
fn bound_check(
    a: &InternalVar,
    b: &InternalVar,
    strict: bool,
    bits: u32,
    evaluator: &mut Evaluator,
) {
    //todo appeler bound_constrains et rajouter les gates a l'evaluator
    if bits > FieldElement::max_num_bits() - 1
    //TODO max_num_bits() is not log(p)?
    {
        todo!("ERROR");
    }
    let offset = if strict {
        FieldElement::one()
    } else {
        FieldElement::zero()
    };
    let mut sub_expression = add(&b.expression, -FieldElement::one(), &a.expression); //a-b
    sub_expression.q_c += offset; //a-b+offset
    let w = evaluator.add_witness_to_cs(); //range_check requires a witness - TODO may be this can be avoided?
    evaluator
        .gates
        .push(Gate::Arithmetic(&sub_expression - &Arithmetic::from(&w)));
    range_constraint(w, bits, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    });
}

pub fn is_unit(arith: &Arithmetic) -> Option<Witness> {
    if arith.mul_terms.is_empty()
        && arith.linear_combinations.len() == 1
        && arith.linear_combinations[0].0 == FieldElement::one()
        && arith.q_c == FieldElement::zero()
    {
        return Some(arith.linear_combinations[0].1);
    }
    if arith.mul_terms.is_empty() && arith.linear_combinations.len() == 1 {
        //todo!("should be simplified");
    }
    None
}
pub fn from_witness(witness: Witness) -> Arithmetic {
    Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: vec![(FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}
