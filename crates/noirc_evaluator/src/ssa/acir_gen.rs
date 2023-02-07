use crate::ssa::{
    context::SsaContext,
    mem::Memory,
    node::{Instruction, NodeId, ObjectType, Operation},
    {builtin, mem},
};
use crate::{Evaluator, RuntimeErrorKind};
use acvm::{
    acir::circuit::opcodes::{BlackBoxFuncCall, Opcode as AcirOpcode},
    acir::native_types::{Expression, Witness},
    FieldElement,
};

mod operations;

mod internal_var;
pub(crate) use internal_var::InternalVar;
mod constraints;
use constraints::to_radix_base;
mod internal_var_cache;
use internal_var_cache::InternalVarCache;
// Expose this to the crate as we need to apply range constraints when
// converting the ABI(main parameters) to Noir types
pub(crate) use constraints::range_constraint;
mod intrinsics;
mod memory_map;
use memory_map::MemoryMap;

#[derive(Default)]
pub struct Acir {
    memory_map: MemoryMap,
    var_cache: InternalVarCache,
}

impl Acir {
    pub fn evaluate_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Result<(), RuntimeErrorKind> {
        if ins.operation == Operation::Nop {
            return Ok(());
        }

        let output = match &ins.operation {
            Operation::Binary(binary) => Some(operations::binary::evaluate_binary(
                &mut self.var_cache,
                &mut self.memory_map,
                binary,
                ins.res_type,
                evaluator,
                ctx,
            )),
            Operation::Constrain(value, ..) => operations::constrain::evaluate_constrain_op(
                value,
                &mut self.var_cache,
                evaluator,
                ctx,
            ),
            Operation::Not(value) => {
                let a = (1_u128 << ins.res_type.bits()) - 1;
                let l_c = self.var_cache.get_or_compute_internal_var_unwrap(*value, evaluator, ctx);
                Some(
                    constraints::subtract(
                        &Expression::from(&FieldElement::from(a)),
                        FieldElement::one(),
                        l_c.expression(),
                    )
                    .into(),
                )
            }
            Operation::Cast(value) => {
                self.var_cache.get_or_compute_internal_var(*value, evaluator, ctx)
            }
            Operation::Truncate { value, bit_size, max_bit_size } => {
                let value =
                    self.var_cache.get_or_compute_internal_var_unwrap(*value, evaluator, ctx);
                Some(InternalVar::from_expression(constraints::evaluate_truncate(
                    value.expression(),
                    *bit_size,
                    *max_bit_size,
                    evaluator,
                )))
            }
            Operation::Intrinsic(opcode, args) => self
                .evaluate_opcode(ins.id, *opcode, args, ins.res_type, ctx, evaluator)
                .map(InternalVar::from),
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
                            self.memory_map.load_array(array)
                        }
                        None => vec![self
                            .var_cache
                            .get_or_compute_internal_var_unwrap(*node_id, evaluator, ctx)],
                    };

                    for mut object in objects {
                        let witness = object
                            .get_or_compute_witness(evaluator, true)
                            .expect("infallible: `None` can only be returned when we disallow constant Expressions.");
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

                None
            }
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                let cond =
                    self.var_cache.get_or_compute_internal_var_unwrap(*condition, evaluator, ctx);
                let l_c = self.var_cache.get_or_compute_internal_var_unwrap(*lhs, evaluator, ctx);
                let r_c = self.var_cache.get_or_compute_internal_var_unwrap(*rhs, evaluator, ctx);
                let sub =
                    constraints::subtract(l_c.expression(), FieldElement::one(), r_c.expression());
                let result = constraints::add(
                    &constraints::mul_with_witness(evaluator, cond.expression(), &sub),
                    FieldElement::one(),
                    r_c.expression(),
                );
                Some(result.into())
            }
            Operation::Nop => None,
            Operation::Load { array_id, index } => {
                //retrieves the value from the map if address is known at compile time:
                //address = l_c and should be constant
                let index =
                    self.var_cache.get_or_compute_internal_var_unwrap(*index, evaluator, ctx);

                let array_element = match index.to_const() {
                    Some(index) => {
                        let idx = mem::Memory::as_u32(index);
                        let mem_array = &ctx.mem[*array_id];

                        self.memory_map.load_array_element_constant_index(mem_array, idx).expect(
                            "ICE: index {idx} was out of bounds for array of length {mem_array.len}",
                        )
                    }
                    None => unimplemented!("dynamic arrays are not implemented yet"),
                };
                Some(array_element)
            }
            Operation::Store { array_id, index, value } => {
                //maps the address to the rhs if address is known at compile time
                let index =
                    self.var_cache.get_or_compute_internal_var_unwrap(*index, evaluator, ctx);
                let value =
                    self.var_cache.get_or_compute_internal_var_unwrap(*value, evaluator, ctx);

                match index.to_const() {
                    Some(index) => {
                        let idx = mem::Memory::as_u32(index);
                        let absolute_adr = ctx.mem[*array_id].absolute_adr(idx);
                        self.memory_map.insert(absolute_adr, value);
                        //we do not generate constraint, so no output.
                        None
                    }
                    None => todo!("dynamic arrays are not implemented yet"),
                }
            }
            i @ Operation::Jne(..)
            | i @ Operation::Jeq(..)
            | i @ Operation::Jmp(_)
            | i @ Operation::Phi { .. }
            | i @ Operation::Call { .. }
            | i @ Operation::Result { .. } => {
                unreachable!("Invalid instruction: {:?}", i);
            }
        };

        // If the output returned an `InternalVar` then we add it to the cache
        if let Some(mut output) = output {
            output.set_id(ins.id);
            self.var_cache.update(ins.id, output);
        }

        Ok(())
    }

    // Generate constraints for two types of functions:
    // - Builtin functions: These are functions that
    // are implemented by the compiler.
    // - ACIR black box functions. These are referred
    // to as `LowLevel`
    fn evaluate_opcode(
        &mut self,
        instruction_id: NodeId,
        opcode: builtin::Opcode,
        args: &[NodeId],
        res_type: ObjectType,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Option<Expression> {
        use builtin::Opcode;

        let outputs;
        match opcode {
            Opcode::ToBits => {
                // TODO: document where `0` and `1` are coming from, for args[0], args[1]
                let bit_size = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let l_c =
                    self.var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
                outputs = to_radix_base(l_c.expression(), 2, bit_size, evaluator);
                if let ObjectType::Pointer(a) = res_type {
                    self.memory_map.map_array(a, &outputs, ctx);
                }
            }
            Opcode::ToRadix => {
                // TODO: document where `0`, `1` and `2` are coming from, for args[0],args[1], args[2]
                let radix = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let limb_size = ctx.get_as_constant(args[2]).unwrap().to_u128() as u32;
                let l_c =
                    self.var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
                outputs = to_radix_base(l_c.expression(), radix, limb_size, evaluator);
                if let ObjectType::Pointer(a) = res_type {
                    self.memory_map.map_array(a, &outputs, ctx);
                }
            }
            Opcode::LowLevel(op) => {
                let inputs = intrinsics::prepare_inputs(
                    &mut self.var_cache,
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

                let func_call = BlackBoxFuncCall {
                    name: op,
                    inputs,                   //witness + bit size
                    outputs: outputs.clone(), //witness
                };
                evaluator.opcodes.push(AcirOpcode::BlackBoxFuncCall(func_call));
            }
        }
        // TODO: document why we only return something when outputs.len()==1
        // TODO what about outputs.len() > 1
        //if there are more than one witness returned, the result is inside ins.res_type as a pointer to an array
        (outputs.len() == 1).then_some(Expression::from(&outputs[0]))
    }
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

/// Returns a `FieldElement` if the expression represents
/// a constant polynomial
///
// TODO we should have a method in ACVM
// TODO which returns the constant term if its a constant
// TODO expression. ie `self.expression.to_const()`
fn const_from_expression(expression: &Expression) -> Option<FieldElement> {
    expression.is_const().then_some(expression.q_c)
}

// Returns a `Witness` if the `Expression` can be represented as a degree-1
// univariate polynomial. Otherwise, Return None.
//
// Note that `Witness` is only capable of expressing polynomials of the form
// f(x) = x and not polynomials of the form f(x) = mx+c , so this method has
// extra checks to ensure that m=1 and c=0
//
// TODO: move to ACVM repo
fn optional_expression_to_witness(arith: &Expression) -> Option<Witness> {
    let is_deg_one_univariate = expression_is_deg_one_univariate(arith);

    if is_deg_one_univariate {
        // If we get here, we know that our expression is of the form `f(x) = mx+c`
        // We want to now restrict ourselves to expressions of the form f(x) = x
        // ie where the constant term is 0 and the coefficient in front of the variable is
        // one.
        let coefficient = arith.linear_combinations[0].0;
        let variable = arith.linear_combinations[0].1;
        let constant = arith.q_c;

        let coefficient_is_one = coefficient.is_one();
        let constant_term_is_zero = constant.is_zero();

        if coefficient_is_one && constant_term_is_zero {
            return Some(variable);
        }
    }

    None
}
/// Converts an `Expression` into a `Witness`
/// - If the `Expression` is a degree-1 univariate polynomial
/// then this conversion is a simple coercion.
/// - Otherwise, we create a new `Witness` and set it to be equal to the
/// `Expression`.
pub(crate) fn expression_to_witness<A: constraints::ACIRState>(
    expr: Expression,
    evaluator: &mut A,
) -> Witness {
    match optional_expression_to_witness(&expr) {
        Some(witness) => witness,
        None => evaluator.create_intermediate_variable(expr),
    }
}
// Returns true if highest degree term in the expression is one.
//
// - `mul_term` in an expression contains degree-2 terms
// - `linear_combinations` contains degree-1 terms
// Hence, it is sufficient to check that there are no `mul_terms`
//
// Examples:
// -  f(x,y) = x + y would return true
// -  f(x,y) = xy would return false, the degree here is 2
// -  f(x,y) = 0 would return true, the degree is 0
//
// TODO: move to ACVM repo
fn expression_is_degree_1(expression: &Expression) -> bool {
    expression.mul_terms.is_empty()
}
// Returns true if the expression can be seen as a degree-1 univariate polynomial
//
// - `mul_terms` in an expression can be univariate, however unless the coefficient
// is zero, it is always degree-2.
// - `linear_combinations` contains the sum of degree-1 terms, these terms do not
// need to contain the same variable and so it can be multivariate. However, we
// have thus far only checked if `linear_combinations` contains one term, so this
// method will return false, if the `Expression` has not been simplified.
//
// Hence, we check in the simplest case if an expression is a degree-1 univariate,
// by checking if it contains no `mul_terms` and it contains one `linear_combination` term.
//
// Examples:
// - f(x,y) = x would return true
// - f(x,y) = x + 6 would return true
// - f(x,y) = 2*y + 6 would return true
// - f(x,y) = x + y would return false
// - f(x,y) = x + x should return true, but we return false *** (we do not simplify)
// - f(x,y) = 5 would return false
// TODO move to ACVM repo
// TODO: ACVM has a method called is_linear, we should change this to `max_degree_one`
fn expression_is_deg_one_univariate(expression: &Expression) -> bool {
    let has_one_univariate_term = expression.linear_combinations.len() == 1;
    expression_is_degree_1(expression) && has_one_univariate_term
}
