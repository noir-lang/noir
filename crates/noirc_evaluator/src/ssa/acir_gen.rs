use crate::ssa::{
    builtin::Opcode,
    context::SsaContext,
    mem::{ArrayId, MemArray, Memory},
    node::{BinaryOp, Instruction, Node, NodeId, ObjectType, Operation},
    {builtin, mem, node},
};
use crate::{Evaluator, RuntimeErrorKind};
use acvm::{
    acir::circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    },
    acir::native_types::{Expression, Linear, Witness},
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::{collections::HashMap, ops::Mul};

mod internal_var;
pub(crate) use internal_var::InternalVar;

mod constraints;
// Expose this to the crate as we need to apply range constraints when
// converting the ABI(main parameters) to Noir types
pub(crate) use constraints::range_constraint;
mod intrinsics;

#[derive(Default)]
pub struct Acir {
    arith_cache: HashMap<NodeId, InternalVar>,
    memory_map: HashMap<u32, InternalVar>, //maps memory address to expression
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a node object as an arithmetic expression
    fn substitute(
        &mut self,
        id: NodeId,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        if let Some(internal_var) = self.arith_cache.get(&id) {
            return internal_var.clone();
        }
        let var = match ctx.try_get_node(id) {
            Some(node::NodeObject::Const(c)) => {
                let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                let expr = Expression::from_field(f_value);
                InternalVar::new(expr, None, Some(id))
            }
            Some(node::NodeObject::Obj(v)) => match v.get_type() {
                node::ObjectType::Pointer(_) => InternalVar::default(),
                _ => {
                    let w = v.witness.unwrap_or_else(|| evaluator.add_witness_to_cs());
                    let expr = Expression::from(&w);
                    InternalVar::new(expr, Some(w), Some(id))
                }
            },
            _ => {
                let w = evaluator.add_witness_to_cs();
                let expr = Expression::from(&w);
                InternalVar::new(expr, Some(w), Some(id))
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
    ) -> Result<(), RuntimeErrorKind> {
        if ins.operation == Operation::Nop {
            return Ok(());
        }

        let mut output = match &ins.operation {
            Operation::Binary(binary) => self.evaluate_binary(binary, ins.res_type, evaluator, ctx),
            Operation::Constrain(value, ..) => {
                let value = self.substitute(*value, evaluator, ctx);
                let subtract = constraints::subtract(
                    &Expression::one(),
                    FieldElement::one(),
                    value.expression(),
                );
                evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract));
                value
            }
            Operation::Not(value) => {
                let a = (1_u128 << ins.res_type.bits()) - 1;
                let l_c = self.substitute(*value, evaluator, ctx);
                constraints::subtract(
                    &Expression {
                        mul_terms: Vec::new(),
                        linear_combinations: Vec::new(),
                        q_c: FieldElement::from(a),
                    },
                    FieldElement::one(),
                    l_c.expression(),
                )
                .into()
            }
            Operation::Cast(value) => self.substitute(*value, evaluator, ctx),
            i @ Operation::Jne(..)
            | i @ Operation::Jeq(..)
            | i @ Operation::Jmp(_)
            | i @ Operation::Phi { .. }
            | i @ Operation::Result { .. } => {
                unreachable!("Invalid instruction: {:?}", i);
            }
            Operation::Truncate { value, bit_size, max_bit_size } => {
                let value = self.substitute(*value, evaluator, ctx);
                evaluate_truncate(value, *bit_size, *max_bit_size, evaluator)
            }
            Operation::Intrinsic(opcode, args) => {
                let v = self.evaluate_opcode(ins.id, *opcode, args, ins.res_type, ctx, evaluator);
                InternalVar::from(v)
            }
            Operation::Call { .. } => unreachable!("call instruction should have been inlined"),
            Operation::Return(node_ids) => {
                // XXX: When we return a node_id that was created from
                // the UnitType, there is a witness associated with it
                // Ideally no witnesses are created for such types.

                // This can only ever be called in the main context.
                // In all other context's, the return operation is transformed.

                for node_id in node_ids {
                    // An array produces a single node_id
                    // We therefore need to check if the node_id is referring to an array
                    // and deference to get the elements
                    let objects = match Memory::deref(ctx, *node_id) {
                        Some(a) => {
                            let array = &ctx.mem[a];
                            load_array(&mut self.memory_map, array, false, evaluator)
                        }
                        None => vec![self.substitute(*node_id, evaluator, ctx)],
                    };

                    for mut object in objects {
                        let witness = if object.expression().is_const() {
                            evaluator.create_intermediate_variable(object.expression().clone())
                        } else {
                            object.witness(evaluator).expect("unexpected constant expression")
                        };

                        // Before pushing to the public inputs, we need to check that
                        // it was not a private ABI input
                        if evaluator.is_private_abi_input(witness) {
                            return Err(RuntimeErrorKind::Spanless(String::from(
                                "we do not allow private ABI inputs to be returned as public outputs",
                            )));
                        }
                        evaluator.public_inputs.push(witness);
                    }
                }

                InternalVar::default()
            }
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                let cond = self.substitute(*condition, evaluator, ctx);
                let l_c = self.substitute(*lhs, evaluator, ctx);
                let r_c = self.substitute(*rhs, evaluator, ctx);
                let sub =
                    constraints::subtract(l_c.expression(), FieldElement::one(), r_c.expression());
                let result = constraints::add(
                    &constraints::mul_with_witness(evaluator, cond.expression(), &sub),
                    FieldElement::one(),
                    r_c.expression(),
                );
                result.into()
            }
            Operation::Nop => InternalVar::default(),
            Operation::Load { array_id, index } => {
                //retrieves the value from the map if address is known at compile time:
                //address = l_c and should be constant
                let index = self.substitute(*index, evaluator, ctx);

                match index.to_const() {
                    Some(index) => {
                        let idx = mem::Memory::as_u32(index);
                        let mem_array = &ctx.mem[*array_id];
                        let absolute_adr = mem_array.absolute_adr(idx);

                        match self.memory_map.get(&absolute_adr) {
                            Some(internal_var) => {
                                InternalVar::from(internal_var.expression().clone())
                            }
                            None => {
                                //if not found, then it must be a witness (else it is non-initialized memory)
                                let index = idx as usize;
                                if mem_array.values.len() > index {
                                    mem_array.values[index].clone()
                                } else {
                                    unreachable!("Could not find value at index {}", index);
                                }
                            }
                        }
                    }
                    None => unimplemented!("dynamic arrays are not implemented yet"),
                }
            }
            Operation::Store { array_id, index, value } => {
                //maps the address to the rhs if address is known at compile time
                let index = self.substitute(*index, evaluator, ctx);
                let value = self.substitute(*value, evaluator, ctx);

                match index.to_const() {
                    Some(index) => {
                        let idx = mem::Memory::as_u32(index);
                        let absolute_adr = ctx.mem[*array_id].absolute_adr(idx);
                        self.memory_map.insert(absolute_adr, value);
                        //we do not generate constraint, so no output.
                        InternalVar::default()
                    }
                    None => todo!("dynamic arrays are not implemented yet"),
                }
            }
        };
        *output.id_mut() = Some(ins.id);
        self.arith_cache.insert(ins.id, output);
        Ok(())
    }

    fn get_predicate(
        &mut self,
        binary: &node::Binary,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        match binary.predicate {
            Some(pred) => self.substitute(pred, evaluator, ctx),
            None => InternalVar::from(Expression::one()),
        }
    }

    fn evaluate_binary(
        &mut self,
        binary: &node::Binary,
        res_type: ObjectType,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        let l_c = self.substitute(binary.lhs, evaluator, ctx);
        let r_c = self.substitute(binary.rhs, evaluator, ctx);
        let r_size = ctx[binary.rhs].size_in_bits();
        let l_size = ctx[binary.lhs].size_in_bits();
        let max_size = u32::max(r_size, l_size);

        match &binary.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => InternalVar::from(constraints::add(
                l_c.expression(),
                FieldElement::one(),
                r_c.expression(),
            )),
            BinaryOp::Sub { max_rhs_value } | BinaryOp::SafeSub { max_rhs_value } => {
                if res_type == node::ObjectType::NativeField {
                    InternalVar::from(constraints::subtract(
                        l_c.expression(),
                        FieldElement::one(),
                        r_c.expression(),
                    ))
                } else {
                    //we need the type of rhs and its max value, then:
                    //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                    let bit_size = r_size;
                    let r_big = BigUint::one() << bit_size;
                    let mut k = max_rhs_value / &r_big;
                    if max_rhs_value % &r_big != BigUint::zero() {
                        k = &k + BigUint::one();
                    }
                    k = &k * r_big;
                    let f = FieldElement::from_be_bytes_reduce(&k.to_bytes_be());
                    let mut sub_expr = constraints::subtract(
                        l_c.expression(),
                        FieldElement::one(),
                        r_c.expression(),
                    );
                    sub_expr.q_c += f;
                    let mut sub_var = sub_expr.into();
                    //TODO: uses interval analysis for more precise check
                    if let Some(lhs_const) = l_c.to_const() {
                        if max_rhs_value <= &BigUint::from_bytes_be(&lhs_const.to_be_bytes()) {
                            sub_var = InternalVar::from(constraints::subtract(
                                l_c.expression(),
                                FieldElement::one(),
                                r_c.expression(),
                            ));
                        }
                    }
                    sub_var
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => InternalVar::from(constraints::mul_with_witness(
                evaluator,
                l_c.expression(),
                r_c.expression(),
            )),
            BinaryOp::Udiv => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let (q_wit, _) = evaluate_udiv(&l_c, &r_c, max_size, &predicate, evaluator);
                InternalVar::from(q_wit)
            }
            BinaryOp::Sdiv => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).0),
            BinaryOp::Urem => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let (_, r_wit) = evaluate_udiv(&l_c, &r_c, max_size, &predicate, evaluator);
                InternalVar::from(r_wit)
            }
            BinaryOp::Srem => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).1),
            BinaryOp::Div => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let inverse = expression_from_witness(evaluate_inverse(r_c, &predicate, evaluator));
                InternalVar::from(constraints::mul_with_witness(
                    evaluator,
                    l_c.expression(),
                    &inverse,
                ))
            }
            BinaryOp::Eq => InternalVar::from(
                self.evaluate_eq(binary.lhs, binary.rhs, &l_c, &r_c, ctx, evaluator),
            ),
            BinaryOp::Ne => InternalVar::from(
                self.evaluate_neq(binary.lhs, binary.rhs, &l_c, &r_c, ctx, evaluator),
            ),
            BinaryOp::Ult => {
                let size = ctx[binary.lhs].get_type().bits();
                evaluate_cmp(&l_c, &r_c, size, false, evaluator).into()
            }
            BinaryOp::Ule => {
                let size = ctx[binary.lhs].get_type().bits();
                let e = evaluate_cmp(&r_c, &l_c, size, false, evaluator);
                constraints::subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Slt => {
                let s = ctx[binary.lhs].get_type().bits();
                evaluate_cmp(&l_c, &r_c, s, true, evaluator).into()
            }
            BinaryOp::Sle => {
                let s = ctx[binary.lhs].get_type().bits();
                let e = evaluate_cmp(&r_c, &l_c, s, true, evaluator);
                constraints::subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Lt => unimplemented!(
                "Field comparison is not implemented yet, try to cast arguments to integer type"
            ),
            BinaryOp::Lte => unimplemented!(
                "Field comparison is not implemented yet, try to cast arguments to integer type"
            ),
            BinaryOp::And => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::And,
            )),
            BinaryOp::Or => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::Or,
            )),
            BinaryOp::Xor => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::Xor,
            )),
            BinaryOp::Shl | BinaryOp::Shr => unreachable!(),
            i @ BinaryOp::Assign => unreachable!("Invalid Instruction: {:?}", i),
        }
    }

    fn evaluate_neq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
            let array_a = &ctx.mem[a];
            let array_b = &ctx.mem[b];

            if array_a.len == array_b.len {
                let mut x = InternalVar::from(self.zero_eq_array_sum(array_a, array_b, evaluator));
                x.witness(evaluator);
                expression_from_witness(evaluate_zero_equality(&x, evaluator))
            } else {
                //If length are different, then the arrays are different
                Expression::one()
            }
        } else {
            if let (Some(l), Some(r)) = (l_c.to_const(), r_c.to_const()) {
                if l == r {
                    return Expression::default();
                } else {
                    return Expression::one();
                }
            }
            let mut x = InternalVar::from(constraints::subtract(
                l_c.expression(),
                FieldElement::one(),
                r_c.expression(),
            ));
            x.witness(evaluator);
            expression_from_witness(evaluate_zero_equality(&x, evaluator))
        }
    }

    fn evaluate_eq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let neq = self.evaluate_neq(lhs, rhs, l_c, r_c, ctx, evaluator);
        constraints::subtract(&Expression::one(), FieldElement::one(), &neq)
    }

    //Generates gates for the expression: \sum_i(zero_eq(A[i]-B[i]))
    //N.b. We assumes the lengths of a and b are the same but it is not checked inside the function.
    fn zero_eq_array_sum(
        &mut self,
        a: &MemArray,
        b: &MemArray,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let mut sum = Expression::default();

        let a_values = load_array(&mut self.memory_map, a, false, evaluator);
        let b_values = load_array(&mut self.memory_map, b, false, evaluator);

        for (a_iter, b_iter) in a_values.into_iter().zip(b_values) {
            let diff_expr = constraints::subtract(
                a_iter.expression(),
                FieldElement::one(),
                b_iter.expression(),
            );

            let diff_witness = evaluator.add_witness_to_cs();

            let diff_var = InternalVar::new(
                //in cache??
                diff_expr.clone(),
                Some(diff_witness),
                None,
            );
            evaluator.opcodes.push(AcirOpcode::Arithmetic(constraints::subtract(
                &diff_expr,
                FieldElement::one(),
                &expression_from_witness(diff_witness),
            )));
            //TODO: avoid creating witnesses for diff
            sum = constraints::add(
                &sum,
                FieldElement::one(),
                &expression_from_witness(evaluate_zero_equality(&diff_var, evaluator)),
            );
        }
        sum
    }

    fn evaluate_opcode(
        &mut self,
        instruction_id: NodeId,
        opcode: builtin::Opcode,
        args: &[NodeId],
        res_type: ObjectType,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let outputs;
        match opcode {
            Opcode::ToBits => {
                let bit_size = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let l_c = self.substitute(args[0], evaluator, ctx);
                outputs = to_radix_base(&l_c, 2, bit_size, evaluator);
                if let node::ObjectType::Pointer(a) = res_type {
                    map_array(&mut self.memory_map, a, &outputs, ctx);
                }
            }
            Opcode::ToRadix => {
                let radix = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let limb_size = ctx.get_as_constant(args[2]).unwrap().to_u128() as u32;
                let l_c = self.substitute(args[0], evaluator, ctx);
                outputs = to_radix_base(&l_c, radix, limb_size, evaluator);
                if let node::ObjectType::Pointer(a) = res_type {
                    map_array(&mut self.memory_map, a, &outputs, ctx);
                }
            }
            Opcode::LowLevel(op) => {
                let inputs = intrinsics::prepare_inputs(
                    &mut self.arith_cache,
                    &mut self.memory_map,
                    args,
                    ctx,
                    evaluator,
                );
                let output_count = op.definition().output_size.0 as u32;
                outputs = intrinsics::prepare_outputs(
                    &mut self.memory_map,
                    instruction_id,
                    output_count,
                    ctx,
                    evaluator,
                );

                let call_gate = BlackBoxFuncCall {
                    name: op,
                    inputs,                   //witness + bit size
                    outputs: outputs.clone(), //witness
                };
                evaluator.opcodes.push(AcirOpcode::BlackBoxFuncCall(call_gate));
            }
        }

        if outputs.len() == 1 {
            expression_from_witness(outputs[0])
        } else {
            //if there are more than one witness returned, the result is inside ins.res_type as a pointer to an array
            Expression::default()
        }
    }
}

//Map the outputs into the array
fn map_array(
    memory_map: &mut HashMap<u32, InternalVar>,
    a: ArrayId,
    outputs: &[Witness],
    ctx: &SsaContext,
) {
    let array = &ctx.mem[a];
    let adr = array.adr;
    for i in 0..array.len {
        if i < outputs.len() as u32 {
            let var = InternalVar::from(outputs[i as usize]);
            memory_map.insert(adr + i, var);
        } else {
            let var = InternalVar::from(Expression::zero());
            memory_map.insert(adr + i, var);
        }
    }
}

//Load array values into InternalVars
//If create_witness is true, we create witnesses for values that do not have witness
fn load_array(
    memory_map: &mut HashMap<u32, InternalVar>,
    array: &MemArray,
    create_witness: bool,
    evaluator: &mut Evaluator,
) -> Vec<InternalVar> {
    (0..array.len)
        .map(|i| {
            let address = array.adr + i;
            match memory_map.get_mut(&address) {
                Some(memory) => {
                    if create_witness && memory.cached_witness().is_none() {
                        let w = evaluator.create_intermediate_variable(memory.expression().clone());
                        *memory_map.get_mut(&address).unwrap().cached_witness_mut() = Some(w);
                    }
                    memory_map[&address].clone()
                }
                None => array.values[i as usize].clone(),
            }
        })
        .collect()
}

fn evaluate_sdiv(
    _lhs: &InternalVar,
    _rhs: &InternalVar,
    _evaluator: &mut Evaluator,
) -> (Expression, Expression) {
    todo!();
}

//Returns 1 if lhs < rhs
fn evaluate_cmp(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    signed: bool,
    evaluator: &mut Evaluator,
) -> Expression {
    if signed {
        //TODO use range_constraints instead of bit decomposition, like in the unsigned case
        let mut sub_expr =
            constraints::subtract(lhs.expression(), FieldElement::one(), rhs.expression());
        let two_pow = BigUint::one() << (bit_size + 1);
        sub_expr.q_c += FieldElement::from_be_bytes_reduce(&two_pow.to_bytes_be());
        let bits = to_radix_base(&sub_expr.into(), 2, bit_size + 2, evaluator);
        expression_from_witness(bits[(bit_size - 1) as usize])
    } else {
        let is_greater = expression_from_witness(constraints::bound_check(
            lhs.expression(),
            rhs.expression(),
            bit_size,
            evaluator,
        ));
        constraints::subtract(&Expression::one(), FieldElement::one(), &is_greater)
    }
}

//Decomposition into b-base: \sum ai b^i, where 0<=ai<b
// radix: the base, (it is a constant, not a witness)
// num_limbs: the number of elements in the decomposition
// output: (the elements of the decomposition as witness, the sum expression)
fn to_radix(radix: u32, num_limbs: u32, evaluator: &mut Evaluator) -> (Vec<Witness>, Expression) {
    let mut digits = Expression::default();
    let mut radix_pow = FieldElement::one();

    let shift = FieldElement::from(radix as i128);
    let mut result = Vec::new();
    let bit_size = bit_size_u32(radix);
    for _ in 0..num_limbs {
        let limb_witness = evaluator.add_witness_to_cs();
        result.push(limb_witness);
        let limb_expr = expression_from_witness(limb_witness);
        digits = constraints::add(&digits, radix_pow, &limb_expr);
        radix_pow = radix_pow.mul(shift);

        if 1_u128 << (bit_size - 1) != radix as u128 {
            constraints::try_range_constraint(limb_witness, bit_size, evaluator);
        }
        constraints::bound_constraint_with_offset(
            &expression_from_witness(limb_witness),
            &Expression::from_field(shift),
            &Expression::one(),
            bit_size,
            evaluator,
        );
    }

    (result, digits)
}

//decompose lhs onto radix-base with limb_size limbs
fn to_radix_base(
    lhs: &InternalVar,
    radix: u32,
    limb_size: u32,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    // ensure there is no overflow
    let mut max = BigUint::from(radix);
    max = max.pow(limb_size) - BigUint::one();
    assert!(max < FieldElement::modulus());

    let (result, bytes) = to_radix(radix, limb_size, evaluator);
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::ToRadix {
        a: lhs.expression().clone(),
        b: result.clone(),
        radix,
    }));

    evaluator.opcodes.push(AcirOpcode::Arithmetic(constraints::subtract(
        lhs.expression(),
        FieldElement::one(),
        &bytes,
    )));

    result
}

fn simplify_bitwise(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    opcode: &BinaryOp,
) -> Option<InternalVar> {
    if lhs == rhs {
        //simplify bitwise operation of the form: a OP a
        return Some(match opcode {
            BinaryOp::And => lhs.clone(),
            BinaryOp::Or => lhs.clone(),
            BinaryOp::Xor => InternalVar::from(FieldElement::zero()),
            _ => unreachable!(),
        });
    }

    assert!(bit_size < FieldElement::max_num_bits());
    let max = FieldElement::from((1_u128 << bit_size) - 1);
    let mut field = None;
    let mut var = lhs;
    if let Some(l_c) = lhs.to_const() {
        if l_c == FieldElement::zero() || l_c == max {
            field = Some(l_c);
            var = rhs
        }
    } else if let Some(r_c) = rhs.to_const() {
        if r_c == FieldElement::zero() || r_c == max {
            field = Some(r_c);
        }
    }
    if let Some(field) = field {
        //simplify bitwise operation of the form: 0 OP var or 1 OP var
        return Some(match opcode {
            BinaryOp::And => {
                if field.is_zero() {
                    InternalVar::from(field)
                } else {
                    var.clone()
                }
            }
            BinaryOp::Xor => {
                if field.is_zero() {
                    var.clone()
                } else {
                    InternalVar::from(constraints::subtract(
                        &Expression::from_field(field),
                        FieldElement::one(),
                        var.expression(),
                    ))
                }
            }
            BinaryOp::Or => {
                if field.is_zero() {
                    var.clone()
                } else {
                    InternalVar::from(field)
                }
            }
            _ => unreachable!(),
        });
    }

    None
}

fn evaluate_bitwise(
    mut lhs: InternalVar,
    mut rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
    opcode: BinaryOp,
) -> Expression {
    if let Some(var) = simplify_bitwise(&lhs, &rhs, bit_size, &opcode) {
        return var.expression().clone();
    }
    if bit_size == 1 {
        match opcode {
            BinaryOp::And => {
                return constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression())
            }
            BinaryOp::Xor => {
                let sum = constraints::add(lhs.expression(), FieldElement::one(), rhs.expression());
                let mul =
                    constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression());
                return constraints::subtract(&sum, FieldElement::from(2_i128), &mul);
            }
            BinaryOp::Or => {
                let sum = constraints::add(lhs.expression(), FieldElement::one(), rhs.expression());
                let mul =
                    constraints::mul_with_witness(evaluator, lhs.expression(), rhs.expression());
                return constraints::subtract(&sum, FieldElement::one(), &mul);
            }
            _ => unreachable!(),
        }
    }
    //We generate witness from const values in order to use the ACIR bitwise gates
    // If the gate is implemented, it is expected to be better than going through bit decomposition, even if one of the operand is a constant
    // If the gate is not implemented, we rely on the ACIR simplification to remove these witnesses
    if rhs.to_const().is_some() && rhs.cached_witness().is_none() {
        *rhs.cached_witness_mut() =
            Some(evaluator.create_intermediate_variable(rhs.expression().clone()));
        assert!(lhs.to_const().is_none());
    } else if lhs.to_const().is_some() && lhs.cached_witness().is_none() {
        assert!(rhs.to_const().is_none());
        *lhs.cached_witness_mut() =
            Some(evaluator.create_intermediate_variable(lhs.expression().clone()));
    }

    let mut a_witness = lhs.witness(evaluator).expect("unexpected constant expression");
    let mut b_witness = rhs.witness(evaluator).expect("unexpected constant expression");

    let result = evaluator.add_witness_to_cs();
    let bit_size = if bit_size % 2 == 1 { bit_size + 1 } else { bit_size };
    assert!(bit_size < FieldElement::max_num_bits() - 1);
    let max = FieldElement::from((1_u128 << bit_size) - 1);
    let bit_gate = match opcode {
        BinaryOp::And => acvm::acir::BlackBoxFunc::AND,
        BinaryOp::Xor => acvm::acir::BlackBoxFunc::XOR,
        BinaryOp::Or => {
            a_witness = evaluator.create_intermediate_variable(constraints::subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                lhs.expression(),
            ));
            b_witness = evaluator.create_intermediate_variable(constraints::subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                rhs.expression(),
            ));
            acvm::acir::BlackBoxFunc::AND
        }
        _ => unreachable!(),
    };

    let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
        name: bit_gate,
        inputs: vec![
            FunctionInput { witness: a_witness, num_bits: bit_size },
            FunctionInput { witness: b_witness, num_bits: bit_size },
        ],
        outputs: vec![result],
    });
    evaluator.opcodes.push(gate);

    if opcode == BinaryOp::Or {
        constraints::subtract(
            &Expression::from_field(max),
            FieldElement::one(),
            &expression_from_witness(result),
        )
    } else {
        expression_from_witness(result)
    }
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
fn evaluate_truncate(
    lhs: InternalVar,
    rhs: u32,
    max_bits: u32,
    evaluator: &mut Evaluator,
) -> InternalVar {
    assert!(max_bits > rhs, "max_bits = {max_bits}, rhs = {rhs}");

    //0. Check for constant expression. This can happen through arithmetic simplifications
    if let Some(a_c) = lhs.to_const() {
        let mut a_big = BigUint::from_bytes_be(&a_c.to_be_bytes());
        let two = BigUint::from(2_u32);
        a_big %= two.pow(rhs);
        return InternalVar::from(FieldElement::from_be_bytes_reduce(&a_big.to_bytes_be()));
    }
    //1. Generate witnesses a,b,c
    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();

    constraints::try_range_constraint(b_witness, rhs, evaluator); //TODO propagate the error using ?
    constraints::try_range_constraint(c_witness, max_bits - rhs, evaluator);

    //2. Add the constraint a = b+2^Nc
    let mut f = FieldElement::from(2_i128);
    f = f.pow(&FieldElement::from(rhs as i128));
    let b_arith = expression_from_witness(b_witness);
    let c_arith = expression_from_witness(c_witness);
    let res = constraints::add(&b_arith, f, &c_arith); //b+2^Nc
    let my_constraint = constraints::add(&res, -FieldElement::one(), lhs.expression());
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Truncate {
        a: lhs.expression().clone(),
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));
    evaluator.opcodes.push(AcirOpcode::Arithmetic(my_constraint));
    InternalVar::from(b_witness)
}

fn evaluate_udiv(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    predicate: &InternalVar,
    evaluator: &mut Evaluator,
) -> (Witness, Witness) {
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();
    let pa = constraints::mul_with_witness(evaluator, lhs.expression(), predicate.expression());
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Quotient {
        a: lhs.expression().clone(),
        b: rhs.expression().clone(),
        q: q_witness,
        r: r_witness,
        predicate: Some(predicate.expression().clone()),
    }));

    //r<b
    let r_expr = Expression::from(Linear::from_witness(r_witness));
    constraints::try_range_constraint(r_witness, bit_size, evaluator);
    constraints::bound_constraint_with_offset(
        &r_expr,
        rhs.expression(),
        predicate.expression(),
        bit_size,
        evaluator,
    );
    //range check q<=a
    constraints::try_range_constraint(q_witness, bit_size, evaluator);
    // a-b*q-r = 0
    let mut d =
        constraints::mul_with_witness(evaluator, rhs.expression(), &Expression::from(&q_witness));
    d = constraints::add(&d, FieldElement::one(), &Expression::from(&r_witness));
    d = constraints::mul_with_witness(evaluator, &d, predicate.expression());
    let div_euclidean = constraints::subtract(&pa, FieldElement::one(), &d);

    evaluator.opcodes.push(AcirOpcode::Arithmetic(div_euclidean));
    (q_witness, r_witness)
}

//Zero Equality gate: returns 1 if x is not null and 0 else
fn evaluate_zero_equality(x: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    let x_witness = x.cached_witness().unwrap(); //todo we need a witness because of the directive, but we should use an expression

    let m = evaluator.add_witness_to_cs(); //'inverse' of x
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: m }));

    //y=x*m         y is 1 if x is not null, and 0 else
    let y_witness = evaluator.add_witness_to_cs();
    evaluator.opcodes.push(AcirOpcode::Arithmetic(Expression {
        mul_terms: vec![(FieldElement::one(), x_witness, m)],
        linear_combinations: vec![(-FieldElement::one(), y_witness)],
        q_c: FieldElement::zero(),
    }));

    //x=y*x
    let y_expr = expression_from_witness(y_witness);
    let xy = constraints::mul(&expression_from_witness(x_witness), &y_expr);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(constraints::subtract(
        &xy,
        FieldElement::one(),
        &expression_from_witness(x_witness),
    )));
    y_witness
}

/// Creates a new witness and constrains it to be the inverse of x
fn evaluate_inverse(
    mut x: InternalVar,
    predicate: &InternalVar,
    evaluator: &mut Evaluator,
) -> Witness {
    // Create a fresh witness - n.b we could check if x is constant or not
    let inverse_witness = evaluator.add_witness_to_cs();
    let inverse_expr = expression_from_witness(inverse_witness);
    let x_witness = x.witness(evaluator).expect("unexpected constant expression"); //TODO avoid creating witnesses here.
    evaluator
        .opcodes
        .push(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: inverse_witness }));

    //x*inverse = 1
    Expression::default();
    let one = constraints::mul(&expression_from_witness(x_witness), &inverse_expr);
    let lhs = constraints::mul_with_witness(evaluator, &one, predicate.expression());
    evaluator.opcodes.push(AcirOpcode::Arithmetic(constraints::subtract(
        &lhs,
        FieldElement::one(),
        predicate.expression(),
    )));
    inverse_witness
}

// Creates an Expression from a Witness.
//
// This is infallible since an expression is
// a multi-variate polynomial and a Witness
// can be seen as a univariate polynomial
//
// TODO: Possibly remove this small shim.
// TODO: Lets first see how the rest of the code looks after
// TODO further refactor.
fn expression_from_witness(witness: Witness) -> Expression {
    Expression::from(&witness)
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}
fn bit_size_u32(a: u32) -> u32 where {
    num_bits::<u32>() as u32 - a.leading_zeros()
}

fn bit_size_u128(a: u128) -> u32 where {
    num_bits::<u128>() as u32 - a.leading_zeros()
}
