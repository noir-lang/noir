use std::collections::{hash_map::Entry, HashMap};

use acvm::FieldElement;
use noirc_frontend::node_interner::FuncId;

use crate::ssa::{
    node::{Node, Operation},
    optim,
};

use super::{
    block::{self, BlockId},
    context::SsaContext,
    function,
    mem::ArrayId,
    node::{self, Instruction, Mark, NodeId},
};

// Number of allowed times for inlining function calls inside a code block.
// If a function calls another function, the inlining of the first function will leave the second function call that needs to be inlined as well.
// In case of recursive calls, this iterative inlining does not end so we arbitraty limit it. 100 nested calls should already support very complex programs.
const MAX_INLINE_TRIES: u32 = 100;

//inline main
pub fn inline_tree(ctx: &mut SsaContext, block_id: BlockId) {
    //inline all function calls
    let mut retry = MAX_INLINE_TRIES;
    while retry > 0 && !inline_block(ctx, block_id, None) {
        retry -= 1;
    }
    assert!(retry > 0, "Error - too many nested calls");
    for b in ctx[block_id].dominated.clone() {
        inline_tree(ctx, b);
    }
}

pub fn inline_cfg(ctx: &mut SsaContext, entry_block: BlockId, to_inline: Option<FuncId>) -> bool {
    let mut result = true;
    let func_cfg = block::bfs(entry_block, None, ctx);
    for block_id in func_cfg {
        if !inline_block(ctx, block_id, to_inline) {
            result = false;
        }
    }
    result
}

//Return false if some inlined function performs a function call
fn inline_block(ctx: &mut SsaContext, block_id: BlockId, to_inline: Option<FuncId>) -> bool {
    let mut call_ins = vec![];
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted() {
                if let Operation::Call(f, args, arrays) = &ins.operation {
                    if let Some(func_to_inline) = to_inline {
                        if *f == func_to_inline {
                            call_ins.push((
                                ins.id,
                                *f,
                                args.clone(),
                                arrays.clone(),
                                ins.parent_block,
                            ));
                        }
                    } else {
                        call_ins.push((ins.id, *f, args.clone(), arrays.clone(), ins.parent_block));
                    }
                }
            }
        }
    }
    let mut result = true;
    for (ins_id, f, args, arrays, parent_block) in call_ins {
        let f_copy = ctx.get_ssafunc(f).unwrap().clone();
        if !inline(ctx, &f_copy, &args, &arrays, parent_block, ins_id) {
            result = false;
        }
    }

    if to_inline.is_none() {
        ctx.print_block(&ctx[block_id]);
        optim::cse(ctx, block_id); //handles the deleted call instructions
    }
    result
}

pub struct StackFrame {
    stack: Vec<NodeId>,
    pub block: BlockId,
    array_map: HashMap<ArrayId, ArrayId>,
}

impl StackFrame {
    pub fn new(block: BlockId) -> StackFrame {
        StackFrame { stack: Vec::new(), block, array_map: HashMap::new() }
    }

    pub fn push(&mut self, ins_id: NodeId) {
        self.stack.push(ins_id);
    }

    pub fn get_or_default(&self, array_idx: ArrayId) -> ArrayId {
        if let Some(&b) = self.try_get(array_idx) {
            b
        } else {
            array_idx
        }
    }

    pub fn try_get(&self, array_idx: ArrayId) -> Option<&ArrayId> {
        self.array_map.get(&array_idx)
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
    arrays: &[ArrayId],
    block: BlockId,
    call_id: NodeId,
) -> bool {
    let func_arg = ssa_func.arguments.clone();

    //map nodes from the function cfg to the caller cfg
    let mut inline_map = HashMap::<NodeId, NodeId>::new();
    let mut stack_frame = StackFrame::new(block);

    //1. return arrays
    for (arg_caller, arg_function) in arrays.iter().zip(&ssa_func.result_types) {
        if let node::ObjectType::Pointer(a) = arg_function {
            stack_frame.array_map.insert(*a, *arg_caller);
        }
    }

    //2. by copy parameters:
    for (&arg_caller, &arg_function) in args.iter().zip(&func_arg) {
        ctx.handle_assign_inline(arg_function, arg_caller, &mut stack_frame, block);
    }

    let mut result = true;
    //3. inline in the block: we assume the function cfg is already flatened.
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
        );
        if result && nested_call {
            result = false
        }
    }
    result
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
) -> Option<BlockId> {
    let block_func = &ctx[block_id];
    let next_block = block_func.left;
    let block_func_instructions = &block_func.instructions.clone();
    *nested_call = false;
    for &i_id in block_func_instructions {
        if let Some(ins) = ctx.try_get_instruction(i_id) {
            if ins.is_deleted() {
                continue;
            }
            let mut array_id = None;
            let mut clone = ins.clone();

            if let node::ObjectType::Pointer(id) = ins.res_type {
                //We collect data here for potential mapping using the array_map below.
                array_id = Some(id);
            }

            clone.operation.map_id_mut(|id| {
                function::SSAFunction::get_mapped_value(Some(&id), ctx, inline_map, target_block_id)
            });

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
                            let call_ins = ctx.get_mut_instruction(call_id);
                            call_ins.mark = Mark::Deleted;
                        } else {
                            //TODO - when implementing multiple return values: to remove and use result instruction instead
                            let call_ins = ctx.get_mut_instruction(call_id);
                            call_ins.mark = Mark::ReplaceWith(*value);
                            if let Some(a_id) = array_id {
                                if let Some(&i_pointer) = stack_frame.try_get(a_id) {
                                    call_ins.res_type = node::ObjectType::Pointer(i_pointer);
                                }
                            }
                        }
                    }
                }
                Operation::Call(..) => {
                    *nested_call = true;
                    let new_ins = new_cloned_instruction(clone, target_block_id);
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Load { array_id, index } => {
                    //Compute the new address:
                    //TODO use relative addressing, but that requires a few changes, mainly in acir_gen.rs and integer.rs
                    let b = stack_frame.get_or_default(*array_id);
                    let offset = ctx.mem[b].adr as i32 - ctx.mem[*array_id].adr as i32;
                    let index_type = ctx[*index].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);

                    let add =
                        node::Binary { operator: node::BinaryOp::Add, lhs: offset_id, rhs: *index };
                    let adr_id = ctx.new_instruction(Operation::Binary(add), index_type);
                    let mut new_ins = Instruction::new(
                        Operation::Load { array_id: b, index: adr_id },
                        clone.res_type,
                        Some(target_block_id),
                    );
                    new_ins.id = clone.id;
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Store { array_id, index, value } => {
                    let b = stack_frame.get_or_default(*array_id);
                    let offset = ctx.mem[b].adr as i32 - ctx.mem[*array_id].adr as i32;
                    let index_type = ctx[*index].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);

                    let add =
                        node::Binary { operator: node::BinaryOp::Add, lhs: offset_id, rhs: *index };
                    let adr_id = ctx.new_instruction(Operation::Binary(add), index_type);
                    let mut new_ins = Instruction::new(
                        Operation::Store { array_id: b, index: adr_id, value: *value },
                        clone.res_type,
                        Some(target_block_id),
                    );
                    new_ins.id = clone.id;
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Phi { .. } => {
                    unreachable!("Phi instructions should have been simplified");
                }
                _ => {
                    let mut new_ins = new_cloned_instruction(clone, target_block_id);

                    if let Some(id) = array_id {
                        let new_id = stack_frame.get_or_default(id);
                        new_ins.res_type = node::ObjectType::Pointer(new_id);
                    }

                    optim::simplify(ctx, &mut new_ins);
                    if let Mark::ReplaceWith(replacement) = new_ins.mark {
                        if let Some(id) = array_id {
                            if let Entry::Occupied(mut entry) = stack_frame.array_map.entry(id) {
                                if let node::ObjectType::Pointer(new_id) =
                                    ctx[replacement].get_type()
                                {
                                    //we now map the array to rhs array
                                    entry.insert(new_id);
                                }
                            }
                        }

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
    next_block
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
