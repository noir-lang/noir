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
use acvm::acir::native_types::{Expression, Witness};

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
pub(crate) struct Acir {
    memory: AcirMem,
    var_cache: InternalVarCache,
}

impl Acir {
    pub(crate) fn acir_gen(
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
        self.memory.acir_gen(evaluator, ctx);
        Ok(())
    }

    /// Generate ACIR opcodes based on the given instruction
    pub(crate) fn acir_gen_instruction(
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
                binary::evaluate(binary, ins.res_type, self, evaluator, ctx)
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
                intrinsics::evaluate(args, ins, opcode, self, ctx, evaluator)
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
                store::evaluate(&ins.operation, acir_mem, var_cache, evaluator, ctx)?
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
            self.var_cache.update(output);
        }

        Ok(())
    }
}

/// Converts an `Expression` into a `Witness`
/// - If the `Expression` is a degree-1 univariate polynomial
/// then this conversion is a simple coercion.
/// - Otherwise, we create a new `Witness` and set it to be equal to the
/// `Expression`.
pub(crate) fn expression_to_witness(expr: Expression, evaluator: &mut Evaluator) -> Witness {
    expr.to_witness().unwrap_or_else(|| evaluator.create_intermediate_variable(expr))
}
