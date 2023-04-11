use crate::ssa::function::RuntimeType;
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
use acvm::acir::brillig_bytecode::Opcode as BrilligOpcode;
use acvm::acir::circuit::opcodes::{Brillig, JabberingIn, JabberingOut, Opcode as AcirOpcode};
use acvm::acir::native_types::{Expression, Witness};
mod operations;
use iter_extended::vecmap;

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

use self::operations::load;

use super::mem::{ArrayId, Memory};
use super::node::{self, Node, NodeId};

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
        self.memory.acir_gen(evaluator);
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
            Operation::UnsafeCall { func, arguments, returned_values, predicate, .. } => {
                self.unsafe_call(func, arguments, returned_values, *predicate, evaluator, ctx)?
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

    pub(crate) fn unsafe_call(
        &mut self,
        func: &NodeId,
        arguments: &Vec<NodeId>,
        returns: &Vec<NodeId>,
        predicate: Option<NodeId>,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Result<Option<InternalVar>, RuntimeError> {
        let f = ctx.try_get_ssa_func(*func).unwrap();
        if matches!(f.kind, RuntimeType::Oracle(_)) {
            unimplemented!("Oracle functions can, for now, only be called from unsafe function");
        }

        let mut register_load = Vec::with_capacity(arguments.len());
        let mut jabber_inputs = Vec::with_capacity(arguments.len());
        let mut jabber_outputs = Vec::with_capacity(returns.len());

        for (call_argument, func_argument) in arguments.iter().zip(&f.arguments) {
            jabber_inputs.push(jabber_node(call_argument, self, ctx, evaluator)?);
            register_load.push(func_argument.0 .0.into_raw_parts().0 as u32);
        }

        for i in returns {
            let var = self.var_cache.get_or_compute_internal_var_unwrap(*i, evaluator, ctx);
            let witness = self.var_cache.get_or_compute_witness_unwrap(var, evaluator, ctx);
            jabber_outputs.push(jabber_output(self, *i, ctx, evaluator));
        }

        let mut code = f.brillig_code.clone();
        code.push(BrilligOpcode::Bootstrap { register_allocation_indices: register_load });
        if predicate != Some(ctx.zero()) {
            let pred_id = predicate.unwrap_or(ctx.one());
            let pred_var =
                self.var_cache.get_or_compute_internal_var_unwrap(pred_id, evaluator, ctx);
            let brillig_opcde = AcirOpcode::Brillig(Brillig {
                inputs: jabber_inputs,
                outputs: jabber_outputs,
                bytecode: code,
                predicate: Some(pred_var.expression().clone()),
            });
            evaluator.push_opcode(brillig_opcde);
        }

        Ok(None)
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

// Converts a nodeid into a JabberingIn
fn jabber_node(
    node_id: &NodeId,
    acir_gen: &mut Acir,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Result<JabberingIn, RuntimeError> {
    let node_object = cfg.try_get_node(*node_id).expect("could not find node for {node_id}");
    match node_object {
        node::NodeObject::Variable(v) => {
            let node_obj_type = node_object.get_type();
            match node_obj_type {
                // If the `Variable` represents a Pointer
                // Then we know that it is an `Array`
                node::ObjectType::ArrayPointer(a) => return jabber_array(a, acir_gen, cfg, evaluator),
                // If it is not a pointer, we attempt to fetch the witness associated with it
                _ => {
                    if let Some(w) = v.witness {
                        return Ok(JabberingIn::Simple(Expression::from(w)));
                    }
                }
            }
        }
        _ => (),
    }
    let ivar = acir_gen
        .var_cache
        .get_or_compute_internal_var(*node_id, evaluator, cfg)
        .expect("invalid input");
    Ok(JabberingIn::Simple(ivar.to_expression()))
}

fn jabber_array(
    array_id: ArrayId,
    acir_gen: &mut Acir,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Result<JabberingIn, RuntimeError> {
    let mut inputs = Vec::new();

    let array = &cfg.mem[array_id];
    for i in 0..array.len {
        let element = load::evaluate_with_conts_index(
            array_id,
            i,
            &mut acir_gen.memory,
            &mut acir_gen.var_cache,
            None,
            evaluator,
            cfg,
        )?;
        inputs.push(element.expression().clone());
    }
    Ok(JabberingIn::Array(array_id.to_u32(), inputs))
}

fn jabber_output(
    acir_gen: &mut Acir,
    node_id: NodeId,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> JabberingOut {
    let outputs;
    if let Some(array) = Memory::deref(ctx, node_id) {
        let len = ctx.mem[array].len;
        // Create fresh variables that will link to the output
        outputs = vecmap(0..len, |_| evaluator.add_witness_to_cs());

        acir_gen.memory.map_array(array, &outputs, ctx);
        JabberingOut::Array(outputs)
    } else {
        let ivar = acir_gen
            .var_cache
            .get_or_compute_internal_var(node_id, evaluator, ctx)
            .expect("invalid input");
        let w = acir_gen.var_cache.get_or_compute_witness_unwrap(ivar, evaluator, ctx);
        JabberingOut::Simple(w)
    }
}
