use crate::Evaluator;
use crate::{
    errors::RuntimeError,
    ssa::{
        block::BasicBlock,
        builtin,
        context::SsaContext,
        node::{Instruction, Operation},
    },
};
use acvm::{
    acir::native_types::{Expression, Witness},
    FieldElement,
};

mod operations;

mod internal_var;
pub(crate) use internal_var::InternalVar;
mod constraints;
mod internal_var_cache;
use internal_var_cache::InternalVarCache;
// Expose this to the crate as we need to apply range constraints when
// converting the ABI(main parameters) to Noir types
pub(crate) use constraints::range_constraint;
mod acir_mem;
use acir_mem::AcirMem;

#[derive(Default)]
pub struct Acir {
    memory: AcirMem,
    var_cache: InternalVarCache,
}

impl Acir {
    pub fn acir_gen(
        &mut self,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
        root: &BasicBlock,
        show_output: bool,
    ) -> Result<(), RuntimeError> {
        let mut current_block = Some(root);
        while let Some(block) = current_block {
            for iter in &block.instructions {
                let ins = ctx.instruction(*iter);
                self.acir_gen_instruction(ins, evaluator, ctx, show_output)?;
            }
            //TODO we should rather follow the jumps
            current_block = block.left.map(|block_id| &ctx[block_id]);
        }
        self.memory.acir_gen(evaluator);
        Ok(())
    }

    /// Generate ACIR opcodes based on the given instruction
    pub fn acir_gen_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
        show_output: bool,
    ) -> Result<(), RuntimeError> {
        use operations::{
            binary, condition, constrain, intrinsics, load, not, r#return, store, truncate,
        };

        let acir_mem = &mut self.memory;
        let var_cache = &mut self.var_cache;

        let output = match &ins.operation {
            Operation::Binary(binary) => {
                binary::evaluate(binary, ins.res_type, var_cache, acir_mem, evaluator, ctx)
            }
            Operation::Constrain(value, ..) => {
                constrain::evaluate(value, var_cache, evaluator, ctx)
            }
            Operation::Not(value) => not::evaluate(value, ins.res_type, var_cache, evaluator, ctx),
            Operation::Cast(value) => {
                self.var_cache.get_or_compute_internal_var(*value, evaluator, ctx)
            }
            Operation::Truncate { value, bit_size, max_bit_size } => {
                truncate::evaluate(value, *bit_size, *max_bit_size, var_cache, evaluator, ctx)
            }
            Operation::Intrinsic(opcode, args) => {
                let opcode = match opcode {
                    builtin::Opcode::Println(print_info) => {
                        builtin::Opcode::Println(builtin::PrintlnInfo {
                            is_string_output: print_info.is_string_output,
                            show_output,
                        })
                    }
                    _ => *opcode,
                };
                intrinsics::evaluate(args, ins, opcode, var_cache, acir_mem, ctx, evaluator)
            }
            Operation::Return(node_ids) => {
                r#return::evaluate(node_ids, acir_mem, var_cache, evaluator, ctx)?
            }
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                condition::evaluate(*condition, *lhs, *rhs, var_cache, evaluator, ctx)
            }
            Operation::Load { array_id, index, location } => Some(load::evaluate(
                *array_id, *index, acir_mem, var_cache, *location, evaluator, ctx,
            )?),
            Operation::Store { .. } => {
                store::evaluate(&ins.operation, acir_mem, var_cache, evaluator, ctx)
            }
            Operation::Nop => None,
            i @ Operation::Jne(..)
            | i @ Operation::Jeq(..)
            | i @ Operation::Jmp(_)
            | i @ Operation::Phi { .. }
            | i @ Operation::Call { .. }
            | i @ Operation::Result { .. } => {
                unreachable!("Invalid instruction: {:?}", i);
            }
        };

        // If the operation returned an `InternalVar`
        // then we add it to the `InternalVar` cache
        if let Some(mut output) = output {
            output.set_id(ins.id);
            self.var_cache.update(ins.id, output);
        }

        Ok(())
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
pub(crate) fn expression_to_witness(expr: Expression, evaluator: &mut Evaluator) -> Witness {
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
