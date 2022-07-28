use std::collections::HashMap;

use noirc_frontend::node_interner::FuncId;

use crate::{
    errors::RuntimeError,
    ssa::{node::Operation, optim},
};

use super::{
    block::{self, BlockId},
    context::SsaContext,
    function,
    node::{self, Instruction, Mark, NodeId},
};

// Number of allowed times for inlining function calls inside a code block.
// If a function calls another function, the inlining of the first function will leave the second function call that needs to be inlined as well.
// In case of recursive calls, this iterative inlining does not end so we arbitraty limit it. 100 nested calls should already support very complex programs.
const MAX_INLINE_TRIES: u32 = 100;

//inline main
pub fn inline_tree(ctx: &mut SsaContext, block_id: BlockId) -> Result<(), RuntimeError> {
    //inline all function calls
    let mut retry = MAX_INLINE_TRIES;
    while retry > 0 && !inline_block(ctx, block_id, None)? {
        retry -= 1;
    }
    assert!(retry > 0, "Error - too many nested calls");
    for b in ctx[block_id].dominated.clone() {
        inline_tree(ctx, b)?;
    }
    Ok(())
}

pub fn inline_cfg(
    ctx: &mut SsaContext,
    entry_block: BlockId,
    to_inline: Option<FuncId>,
) -> Result<bool, RuntimeError> {
    let mut result = true;
    let func_cfg = block::bfs(entry_block, None, ctx);
    for block_id in func_cfg {
        if !inline_block(ctx, block_id, to_inline)? {
            result = false;
        }
    }
    Ok(result)
}

//Return false if some inlined function performs a function call
fn inline_block(
    ctx: &mut SsaContext,
    block_id: BlockId,
    to_inline: Option<FuncId>,
) -> Result<bool, RuntimeError> {
    let mut call_ins = vec![];
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted() {
                if let Operation::Call(f, args) = &ins.operation {
                    if to_inline.is_none() || to_inline == Some(*f) {
                        call_ins.push((ins.id, *f, args.clone(), ins.parent_block));
                    }
                }
            }
        }
    }
    let mut result = true;
    for (ins_id, f, args, parent_block) in call_ins {
        let f_copy = ctx.get_ssafunc(f).unwrap().clone();
        if !inline(ctx, &f_copy, &args, parent_block, ins_id)? {
            result = false;
        }
    }

    if to_inline.is_none() {
        optim::cse(ctx, block_id)?; //handles the deleted call instructions
    }
    Ok(result)
}

pub struct StackFrame {
    stack: Vec<NodeId>,
    pub block: BlockId,
}

impl StackFrame {
    pub fn new(block: BlockId) -> StackFrame {
        StackFrame { stack: Vec::new(), block }
    }

    pub fn push(&mut self, ins_id: NodeId) {
        self.stack.push(ins_id);
    }

    // add instructions to target_block
    pub fn apply(&mut self, ctx: &mut SsaContext, block: BlockId, call_id: NodeId) {
        let mut pos = ctx[block].instructions.iter().position(|x| *x == call_id).unwrap();
        for new_id in self.stack.iter_mut() {
            ctx[block].instructions.insert(pos, *new_id);
            pos += 1;
        }
        self.stack.clear();
    }
}

//inline a function call
//Return false if the inlined function performs a function call
pub fn inline(
    ctx: &mut SsaContext,
    ssa_func: &function::SSAFunction,
    args: &[NodeId],
    block: BlockId,
    call_id: NodeId,
) -> Result<bool, RuntimeError> {
    let func_arg = ssa_func.arguments.clone();

    //map nodes from the function cfg to the caller cfg
    let mut inline_map = HashMap::<NodeId, NodeId>::new();
    let mut stack_frame = StackFrame::new(block);

    //1. by copy parameters:
    for (&arg_caller, &arg_function) in args.iter().zip(&func_arg) {
        //pass by-ref const array arguments
        ctx.handle_assign_inline(arg_function.0, arg_caller, &mut stack_frame, block);
    }

    let mut result = true;
    //2. inline in the block: we assume the function cfg is already flatened.
    let mut next_block = Some(ssa_func.entry_block);
    while let Some(next_b) = next_block {
        let mut nested_call = false;
        next_block = inline_in_block(
            next_b,
            block,
            &mut inline_map,
            &mut stack_frame,
            call_id,
            &mut nested_call,
            ctx,
        )?;
        if result && nested_call {
            result = false
        }
    }
    Ok(result)
}

//inline the given block of the function body into the target_block
pub fn inline_in_block(
    block_id: BlockId,
    target_block_id: BlockId,
    inline_map: &mut HashMap<NodeId, NodeId>,
    stack_frame: &mut StackFrame,
    call_id: NodeId,
    nested_call: &mut bool,
    ctx: &mut SsaContext,
) -> Result<Option<BlockId>, RuntimeError> {
    let block_func = &ctx[block_id];
    let next_block = block_func.left;
    let block_func_instructions = &block_func.instructions.clone();
    *nested_call = false;
    for &i_id in block_func_instructions {
        if let Some(ins) = ctx.try_get_instruction(i_id) {
            if ins.is_deleted() {
                continue;
            }
            let mut clone = ins.clone();
            clone.operation.map_values_for_inlining(ctx, inline_map, stack_frame, target_block_id);

            match &clone.operation {
                Operation::Nop => (),
                //Return instruction:
                Operation::Return(values) => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    for (i, value) in values.iter().enumerate() {
                        if ctx
                            .get_result_instruction_mut(target_block_id, call_id, i as u32)
                            .is_some()
                        {
                            ctx.get_result_instruction_mut(target_block_id, call_id, i as u32)
                                .unwrap()
                                .mark = Mark::ReplaceWith(*value);
                        }
                    }
                    let call_ins = ctx.get_mut_instruction(call_id);
                    call_ins.mark = Mark::Deleted;
                }
                Operation::Call(..) => {
                    *nested_call = true;
                    let new_ins = new_cloned_instruction(clone, target_block_id);
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Phi { .. } => {
                    unreachable!("Phi instructions should have been simplified");
                }
                _ => {
                    let mut new_ins = new_cloned_instruction(clone, target_block_id);
                    optim::simplify(ctx, &mut new_ins)?;

                    if let Mark::ReplaceWith(replacement) = new_ins.mark {
                        if replacement != new_ins.id {
                            inline_map.insert(i_id, replacement);
                            debug_assert!(stack_frame.stack.contains(&replacement));
                        }
                    } else {
                        push_instruction(ctx, new_ins, stack_frame, inline_map);
                    }
                }
            }
        }
    }

    // add instruction to target_block, at proper location (really need a linked list!)
    stack_frame.apply(ctx, target_block_id, call_id);
    Ok(next_block)
}

fn new_cloned_instruction(original: Instruction, block: BlockId) -> Instruction {
    let mut clone = Instruction::new(original.operation, original.res_type, Some(block));
    // Take the original's ID, it will be used to map it as a replacement in push_instruction later
    clone.id = original.id;
    clone
}

fn push_instruction(
    ctx: &mut SsaContext,
    instruction: Instruction,
    stack_frame: &mut StackFrame,
    inline_map: &mut HashMap<NodeId, NodeId>,
) {
    let old_id = instruction.id;
    let new_id = ctx.add_instruction(instruction);
    stack_frame.push(new_id);
    inline_map.insert(old_id, new_id);
}

impl node::Operation {
    pub fn map_values_for_inlining(
        &mut self,
        ctx: &mut SsaContext,
        inline_map: &HashMap<NodeId, NodeId>,
        stack_frame: &StackFrame,
        block_id: BlockId,
    ) {
        self.map_id_mut(|id| {
            function::SSAFunction::get_mapped_value(Some(&id), ctx, inline_map, block_id)
        });
    }
}
