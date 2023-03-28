use crate::ssa::mem::Memory;
use crate::ssa::node::{self, FunctionKind, NodeObject};
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
use acvm::acir::circuit::opcodes::OracleData;
use acvm::acir::circuit::Opcode;
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
use iter_extended::vecmap;

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
            Operation::Call { func, arguments, returned_arrays, predicate, location } => {
                //TODO : handle predicate
                if let NodeObject::Function(
                    FunctionKind::Builtin(builtin::Opcode::Oracle(name, func_id)),
                    ..,
                ) = ctx[*func]
                {
                    let mut inputs = Vec::new();
                    for argument in arguments {
                        let ivar = self
                            .var_cache
                            .get_or_compute_internal_var(*argument, evaluator, ctx)
                            .unwrap();
                        if let Some(a) = Memory::deref(ctx, *argument) {
                            let array = &ctx.mem[a];
                            for i in 0..array.len {
                                let arr_element = self
                                    .memory
                                    .load_array_element_constant_index(array, i)
                                    .expect("array index out of bounds");
                                inputs.push(arr_element.expression().clone());
                            }
                        } else {
                            inputs.push(ivar.expression().clone());
                        }
                    }
                    let ssa_func = ctx.ssa_func(func_id).unwrap();
                    let mut outputs = Vec::new();
                    let mut ret_arrays = returned_arrays.iter();
                    for (i, typ) in ssa_func.result_types.iter().enumerate() {
                        match typ {
                            node::ObjectType::Pointer(a) => {
                                let ret_array = ret_arrays.next().unwrap();
                                assert_eq!(ret_array.1, i as u32);
                                let a_witess = vecmap(0..ctx.mem[ret_array.0].len, |_| {
                                    evaluator.add_witness_to_cs()
                                });
                                self.memory.map_array(ret_array.0, &a_witess, ctx);
                                outputs.extend(a_witess);
                            }
                            _ => outputs.push(evaluator.add_witness_to_cs()),
                        }
                    }
                    let mut ivar =
                        self.var_cache.get_or_compute_internal_var(ins.id, evaluator, ctx).unwrap();
                    if let Some(w) = outputs.first() {
                        ivar.set_witness(*w);
                        self.var_cache.update(ivar);
                    }

                    evaluator.push_opcode(Opcode::Oracle(OracleData {
                        name: name.to_string(),
                        inputs,
                        input_values: Vec::new(),
                        outputs,
                        output_values: Vec::new(),
                    }));
                } else {
                    unreachable!();
                }

                None
            }
            Operation::Result { call_instruction, index } => {
                let mut cached_witness = None;
                if let NodeObject::Function(
                    FunctionKind::Builtin(builtin::Opcode::Oracle(name, func_id)),
                    ..,
                ) = ctx[*call_instruction]
                {
                    let ssa_func = ctx.ssa_func(func_id).unwrap();
                    let mut idx = 0;
                    for (i, typ) in ssa_func.result_types.iter().enumerate() {
                        if i == *index as usize {
                            cached_witness = Some(Witness(idx));
                            break;
                        }
                        match typ {
                            node::ObjectType::Pointer(a) => idx += ctx.mem[*a].len,
                            _ => idx += 1,
                        }
                    }
                }
                let mut ivar = InternalVar::zero_expr();
                ivar.set_id(ins.id);
                if let Some(w) = cached_witness {
                    ivar.set_witness(w);
                }
                Some(ivar)
            }
            Operation::Nop => None,
            i @ Operation::Jne(..)
            | i @ Operation::Jeq(..)
            | i @ Operation::Jmp(_)
            | i @ Operation::Phi { .. } => {
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
