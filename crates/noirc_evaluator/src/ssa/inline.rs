use std::collections::HashMap;

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
    node::{self, NodeId},
};

// Number of allowed times for inlining function calls inside a code block.
// If a function calls another function, the inlining of the first function will leave the second function call that needs to be inlined as well.
// In case of recursive calls, this iterative inlining does not end so we arbitraty limit it. 100 nested calls should already support very complex programs.
const MAX_INLINE_TRIES: u32 = 100;

//inline main
pub fn inline_tree(ctx: &mut SsaContext, block_id: BlockId) {
    //inline all function calls
    let mut retry = MAX_INLINE_TRIES;
    while retry > 0 && !inline_block(ctx, ctx.first_block, None) {
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

//inline all function calls of the block
//Return false if some inlined function performs a function call
fn inline_block(ctx: &mut SsaContext, block_id: BlockId, to_inline: Option<FuncId>) -> bool {
    let mut call_ins = Vec::<NodeId>::new();
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted && matches!(ins.operator, node::Operation::Call(_)) {
                {
                    if let Some(f) = to_inline {
                        if ins.operator == node::Operation::Call(f) {
                            call_ins.push(*i);
                        }
                    } else {
                        call_ins.push(*i);
                    }
                }
            }
        }
    }

    let mut result = true;
    for ins_id in call_ins {
        let ins = ctx.try_get_instruction(ins_id).unwrap().clone();
        if let node::Instruction { operator: node::Operation::Call(f), .. } = ins {
            let f_copy = ctx.get_ssafunc(f).unwrap().clone();
            if !inline(&ins.ins_arguments, ctx, ins.parent_block, ins.id, &f_copy) {
                result = false;
            }
        }
    }
    optim::cse(ctx, block_id); //handles the deleted call instructions
    result
}

pub struct StackFrame {
    stack: Vec<NodeId>,
    pub block: BlockId,
    array_map: HashMap<u32, u32>,
}

impl StackFrame {
    pub fn new(block: BlockId) -> StackFrame {
        StackFrame { stack: Vec::new(), block, array_map: HashMap::new() }
    }

    pub fn push(&mut self, ins_id: NodeId) {
        self.stack.push(ins_id);
    }

    pub fn get_or_default(&self, array_idx: u32) -> u32 {
        if let Some(&b) = self.try_get(array_idx) {
            b
        } else {
            array_idx
        }
    }

    pub fn try_get(&self, array_idx: u32) -> Option<&u32> {
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
    args: &function::CallStack,
    ctx: &mut SsaContext,
    block: BlockId,
    call_id: NodeId,
    ssa_func: &function::SSAFunction,
) -> bool {
    let func_arg = ssa_func.arguments.clone();
    //map nodes from the function cfg to the caller cfg
    let mut inline_map: HashMap<NodeId, NodeId> = HashMap::new();
    let mut stack_frame = StackFrame::new(block);

    //1. return values
    for (arg_caller, arg_function) in args.return_values.iter().zip(&ssa_func.result) {
        if let node::ObjectType::Pointer(a) = arg_function {
            if let node::ObjectType::Pointer(b) = ctx.get_object_type(*arg_caller) {
                stack_frame.array_map.insert(*a, b);
            } else {
                unreachable!("Error: expected array, got {:?}", ctx.get_object_type(*arg_caller));
            }
        }
    }

    //2. by copy parameters:
    for (&arg_caller, &arg_function) in args.arguments.iter().zip(&func_arg) {
        ctx.handle_assign_inline(arg_function, arg_caller, &mut stack_frame);
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
        let mut array_func_idx = u32::MAX;
        if let Some(ins) = ctx.try_get_instruction(i_id) {
            if ins.is_deleted {
                continue;
            }
            let clone = ins.clone();
            if let node::ObjectType::Pointer(a) = ins.res_type {
                //We collect data here for potential mapping using the array_map below.
                array_func_idx = a;
            }

            let new_left =
                function::SSAFunction::get_mapped_value(Some(&clone.lhs), ctx, inline_map);
            let new_right =
                function::SSAFunction::get_mapped_value(Some(&clone.rhs), ctx, inline_map);
            let new_arg = function::SSAFunction::get_mapped_value(
                clone.ins_arguments.arguments.first(),
                ctx,
                inline_map,
            );

            match clone.operator {
                Operation::Nop => (),
                //Return instruction:
                Operation::Ret => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    if let Some(ret_id) = ctx[target_block_id].get_result_instruction(call_id, ctx)
                    {
                        //we support only one result for now, should use 'ins.lhs.get_value()'
                        if let node::NodeObj::Instr(i) = &mut ctx[ret_id] {
                            i.is_deleted = true;
                            i.rhs = new_arg;
                        }
                        let call_ins = ctx.get_mut_instruction(call_id);
                        call_ins.is_deleted = true;
                        call_ins.operator = Operation::Nop;
                        call_ins.rhs = NodeId::dummy();
                    } else {
                        //we use the call instruction instead
                        //we could use the ins_arguments to get the results here, and since we have the input arguments (in the ssafunction) we know how many there are.
                        //for now the call instruction is replaced by the (one) result
                        let call_ins = ctx.get_mut_instruction(call_id);
                        call_ins.is_deleted = true;
                        call_ins.rhs = new_arg;
                        if let Some(&i_pointer) = stack_frame.try_get(array_func_idx) {
                            call_ins.res_type = node::ObjectType::Pointer(i_pointer);
                        }
                    }
                }
                Operation::Call(_) => {
                    *nested_call = true;

                    let mut new_ins = node::Instruction::new(
                        clone.operator,
                        new_left,
                        new_right,
                        clone.res_type,
                        Some(target_block_id),
                    );

                    let mut call_stack = function::CallStack::new();
                    for i in clone.ins_arguments.arguments {
                        call_stack.arguments.push(function::SSAFunction::get_mapped_value(
                            Some(&i),
                            ctx,
                            inline_map,
                        ));
                    }
                    new_ins.ins_arguments = call_stack;
                    let result_id = ctx.add_instruction(new_ins);
                    stack_frame.stack.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                Operation::Load(a) => {
                    //Compute the new address:
                    //TODO use relative addressing, but that requires a few changes, mainly in acir_gen.rs and integer.rs
                    let b = stack_frame.get_or_default(a);
                    //n.b. this offset is always positive
                    let offset = ctx.mem.arrays[b as usize].adr - ctx.mem.arrays[a as usize].adr;
                    let index_type = ctx[new_left].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);
                    let adr_id =
                        ctx.new_instruction(offset_id, new_left, node::Operation::Add, index_type);
                    let new_ins = node::Instruction::new(
                        node::Operation::Load(b),
                        adr_id,
                        adr_id,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    let result_id = ctx.add_instruction(new_ins);
                    stack_frame.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                Operation::Store(a) => {
                    let b = stack_frame.get_or_default(a);
                    let offset = ctx.mem.arrays[b as usize].adr - ctx.mem.arrays[a as usize].adr;
                    let index_type = ctx[new_right].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);
                    let adr_id =
                        ctx.new_instruction(offset_id, new_right, node::Operation::Add, index_type);
                    let new_ins = node::Instruction::new(
                        node::Operation::Store(b),
                        new_left,
                        adr_id,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    let result_id = ctx.add_instruction(new_ins);
                    stack_frame.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                _ => {
                    let mut new_ins = node::Instruction::new(
                        clone.operator,
                        new_left,
                        new_right,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    let i_pointer = stack_frame.get_or_default(array_func_idx);
                    if (i_pointer as usize) < ctx.mem.arrays.len() {
                        new_ins.res_type = node::ObjectType::Pointer(i_pointer);
                    }
                    optim::simplify(ctx, &mut new_ins);
                    let result_id;
                    let mut to_delete = false;
                    if new_ins.is_deleted {
                        result_id = new_ins.rhs;
                        if let std::collections::hash_map::Entry::Occupied(mut e) =
                            stack_frame.array_map.entry(array_func_idx)
                        {
                            if let node::ObjectType::Pointer(a) = ctx[result_id].get_type() {
                                //we now map the array to rhs array
                                e.insert(a);
                            }
                        }
                        if new_ins.rhs == new_ins.id {
                            to_delete = true;
                        }
                    } else {
                        result_id = ctx.add_instruction(new_ins);
                        stack_frame.push(result_id);
                    }
                    //ignore self-deleted instructions
                    if !to_delete {
                        inline_map.insert(i_id, result_id);
                    }
                }
            }
        }
    }

    // add instruction to target_block, at proper location (really need a linked list!)
    stack_frame.apply(ctx, target_block_id, call_id);
    next_block
}
