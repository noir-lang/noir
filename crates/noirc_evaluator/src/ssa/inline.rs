use std::collections::{hash_map::Entry, HashMap};

use noirc_frontend::monomorphization::ast::FuncId;

use crate::{
    errors::RuntimeError,
    ssa::{
        node::{Node, Operation},
        optim,
    },
};

use super::{
    block::{self, BlockId},
    conditional::DecisionTree,
    context::SsaContext,
    function,
    mem::{ArrayId, Memory},
    node::{self, Instruction, Mark, NodeId, ObjectType},
};

// Number of allowed times for inlining function calls inside a code block.
// If a function calls another function, the inlining of the first function will leave the second function call that needs to be inlined as well.
// In case of recursive calls, this iterative inlining does not end so we arbitrarily limit it. 100 nested calls should already support very complex programs.
const MAX_INLINE_TRIES: u32 = 100;

//inline main
pub fn inline_tree(
    ctx: &mut SsaContext,
    block_id: BlockId,
    decision: &DecisionTree,
) -> Result<(), RuntimeError> {
    //inline all function calls
    let mut retry = MAX_INLINE_TRIES;
    while retry > 0 && !inline_block(ctx, block_id, None, decision)? {
        retry -= 1;
    }
    assert!(retry > 0, "Error - too many nested calls");
    for b in ctx[block_id].dominated.clone() {
        inline_tree(ctx, b, decision)?;
    }
    Ok(())
}

pub fn inline_cfg(
    ctx: &mut SsaContext,
    func_id: FuncId,
    to_inline: Option<FuncId>,
) -> Result<bool, RuntimeError> {
    let mut result = true;
    let func = ctx.get_ssafunc(func_id).unwrap();
    let func_cfg = block::bfs(func.entry_block, None, ctx);
    let decision = func.decision.clone();
    for block_id in func_cfg {
        if !inline_block(ctx, block_id, to_inline, &decision)? {
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
    decision: &DecisionTree,
) -> Result<bool, RuntimeError> {
    let mut call_ins = vec![];
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted() {
                if let Operation::Call { func, arguments, returned_arrays, .. } = &ins.operation {
                    if to_inline.is_none() || to_inline == ctx.try_get_funcid(*func) {
                        call_ins.push((
                            ins.id,
                            *func,
                            arguments.clone(),
                            returned_arrays.clone(),
                            block_id,
                        ));
                    }
                }
            }
        }
    }
    let mut result = true;
    for (ins_id, f, args, arrays, parent_block) in call_ins {
        if let Some(func_id) = ctx.try_get_funcid(f) {
            let f_copy = ctx.get_ssafunc(func_id).unwrap().clone();
            if !inline(ctx, &f_copy, &args, &arrays, parent_block, ins_id, decision)? {
                result = false;
            }
        }
    }

    if to_inline.is_none() {
        optim::simple_cse(ctx, block_id)?;
    }
    Ok(result)
}

pub struct StackFrame {
    pub stack: Vec<NodeId>,
    pub block: BlockId,
    array_map: HashMap<ArrayId, ArrayId>,
    pub created_arrays: HashMap<ArrayId, BlockId>,
    zeros: HashMap<ObjectType, NodeId>,
    pub return_arrays: Vec<ArrayId>,
    lca_cache: HashMap<(BlockId, BlockId), BlockId>,
}

impl StackFrame {
    pub fn new(block: BlockId) -> StackFrame {
        StackFrame {
            stack: Vec::new(),
            block,
            array_map: HashMap::new(),
            created_arrays: HashMap::new(),
            zeros: HashMap::new(),
            return_arrays: Vec::new(),
            lca_cache: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.array_map.clear();
        self.created_arrays.clear();
        self.lca_cache.clear();
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

    // add instructions to target_block, after/before the provided instruction
    pub fn apply(&mut self, ctx: &mut SsaContext, block: BlockId, ins_id: NodeId, after: bool) {
        let mut pos = ctx[block].instructions.iter().position(|x| *x == ins_id).unwrap();
        if after {
            pos += 1;
        }
        let after = ctx[block].instructions.split_off(pos);
        ctx[block].instructions.extend_from_slice(&self.stack);
        ctx[block].instructions.extend_from_slice(&after);
        self.stack.clear();
    }

    pub fn set_zero(&mut self, ctx: &mut SsaContext, o_type: ObjectType) {
        self.zeros.entry(o_type).or_insert_with(|| ctx.zero_with_type(o_type));
    }
    pub fn get_zero(&self, o_type: ObjectType) -> NodeId {
        self.zeros[&o_type]
    }

    // returns the lca of x and y, using a cache
    pub fn lca(&mut self, ctx: &SsaContext, x: BlockId, y: BlockId) -> BlockId {
        let ordered_blocks = if x.0 < y.0 { (x, y) } else { (y, x) };
        *self.lca_cache.entry(ordered_blocks).or_insert_with(|| block::lca(ctx, x, y))
    }

    // returns true if the array_id is created in the block of the stack
    pub fn is_new_array(&mut self, ctx: &SsaContext, array_id: &ArrayId) -> bool {
        if self.return_arrays.contains(array_id) {
            //array is defined by the caller
            return false;
        }
        if self.created_arrays[array_id] != self.block {
            let lca = self.lca(ctx, self.block, self.created_arrays[array_id]);
            if lca != self.block && lca != self.created_arrays[array_id] {
                //if the array is defined in a parallel branch, it is new in this branch
                return true;
            }
            false
        } else {
            true
        }
    }
}

//inline a function call
//Return false if the inlined function performs a function call
pub fn inline(
    ctx: &mut SsaContext,
    ssa_func: &function::SSAFunction,
    args: &[NodeId],
    arrays: &[(ArrayId, u32)],
    block: BlockId,
    call_id: NodeId,
    decision: &DecisionTree,
) -> Result<bool, RuntimeError> {
    let func_arg = ssa_func.arguments.clone();

    //map nodes from the function cfg to the caller cfg
    let mut inline_map = HashMap::<NodeId, NodeId>::new();
    let mut stack_frame = StackFrame::new(block);

    //1. return arrays
    for arg_caller in arrays.iter() {
        if let node::ObjectType::Pointer(a) = ssa_func.result_types[arg_caller.1 as usize] {
            stack_frame.array_map.insert(a, arg_caller.0);
            stack_frame.return_arrays.push(arg_caller.0);
        }
    }

    //2. by copy parameters:
    for (&arg_caller, &arg_function) in args.iter().zip(&func_arg) {
        //pass by-ref const array arguments
        if let node::ObjectType::Pointer(x) = ctx.get_object_type(arg_function.0) {
            if let node::ObjectType::Pointer(y) = ctx.get_object_type(arg_caller) {
                if !arg_function.1 && !stack_frame.array_map.contains_key(&x) {
                    stack_frame.array_map.insert(x, y);
                    continue;
                }
            }
        }
        ctx.handle_assign_inline(arg_function.0, arg_caller, &mut stack_frame, block);
    }

    let mut result = true;
    //3. inline in the block: we assume the function cfg is already flattened.
    let mut next_block = Some(ssa_func.entry_block);
    while let Some(next_b) = next_block {
        let mut nested_call = false;
        next_block = inline_in_block(
            next_b,
            &mut inline_map,
            &mut stack_frame,
            call_id,
            &mut nested_call,
            ctx,
            decision,
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
    inline_map: &mut HashMap<NodeId, NodeId>,
    stack_frame: &mut StackFrame,
    call_id: NodeId,
    nested_call: &mut bool,
    ctx: &mut SsaContext,
    decision: &DecisionTree,
) -> Result<Option<BlockId>, RuntimeError> {
    let block_func = &ctx[block_id];
    let next_block = block_func.left;
    let block_func_instructions = &block_func.instructions.clone();
    let predicate =
        if let Operation::Call { predicate, .. } = &ctx.get_instruction(call_id).operation {
            *predicate
        } else {
            unreachable!("invalid call id");
        };
    let mut short_circuit = false;

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

            clone.operation.map_values_for_inlining(
                ctx,
                inline_map,
                stack_frame,
                stack_frame.block,
            );

            match &clone.operation {
                Operation::Nop => (),
                //Return instruction:
                Operation::Return(values) => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    for (i, value) in values.iter().enumerate() {
                        if let Some(result) =
                            ctx.get_result_instruction_mut(stack_frame.block, call_id, i as u32)
                        {
                            result.mark = Mark::ReplaceWith(*value);
                        }
                    }
                    let call_ins = ctx.get_mut_instruction(call_id);
                    call_ins.mark = Mark::Deleted;
                }
                Operation::Call { .. } => {
                    *nested_call = true;
                    let new_ins = new_cloned_instruction(clone, stack_frame.block);
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Load { array_id, index } => {
                    //Compute the new address:
                    let b = stack_frame.get_or_default(*array_id);
                    let mut new_ins = Instruction::new(
                        Operation::Load { array_id: b, index: *index },
                        clone.res_type,
                        Some(stack_frame.block),
                    );
                    new_ins.id = clone.id;
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Store { array_id, index, value } => {
                    let b = stack_frame.get_or_default(*array_id);
                    let mut new_ins = Instruction::new(
                        Operation::Store { array_id: b, index: *index, value: *value },
                        clone.res_type,
                        Some(stack_frame.block),
                    );
                    new_ins.id = clone.id;
                    push_instruction(ctx, new_ins, stack_frame, inline_map);
                }
                Operation::Phi { .. } => {
                    unreachable!("Phi instructions should have been simplified");
                }
                _ => {
                    let mut new_ins = new_cloned_instruction(clone, stack_frame.block);
                    if let Some(id) = array_id {
                        let new_id = stack_frame.get_or_default(id);
                        new_ins.res_type = node::ObjectType::Pointer(new_id);
                    }

                    let err = optim::simplify(ctx, &mut new_ins);
                    if err.is_err() {
                        //add predicate if under condition, else short-circuit the target block.
                        let ass_value = decision.get_assumption_value(predicate);
                        if ass_value.map_or(false, |value| ctx.under_assumption(value)) {
                            ctx.add_predicate(ass_value.unwrap(), &mut new_ins, stack_frame);
                        } else {
                            short_circuit = true;
                            break;
                        }
                    }
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

    // we conditionalize the stack frame into a new stack frame (to avoid ownership issues)
    let mut stack2 = StackFrame::new(stack_frame.block);
    stack2.return_arrays = stack_frame.return_arrays.clone();
    if short_circuit {
        super::block::short_circuit_inline(ctx, stack_frame.block);
    } else {
        decision.conditionalize_inline(ctx, &stack_frame.stack, &mut stack2, predicate)?;
        // we add the conditionalized instructions to the target_block, at proper location (really need a linked list!)
        stack2.apply(ctx, stack_frame.block, call_id, false);
    }

    stack_frame.stack.clear();
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
        match self {
            //default way to handle arrays during inlining; we map arrays using the stack_frame
            Operation::Binary(_) | Operation::Constrain(..) | Operation::Intrinsic(_,_)
            => {
                self.map_id_mut(|id| {
                    if let Some(a) = Memory::deref(ctx, id) {
                        let b = stack_frame.get_or_default(a);
                        if b != a {
                            let new_var = node::Variable {
                                id: NodeId::dummy(),
                                obj_type: node::ObjectType::Pointer(b),
                                name: String::new(),
                                root: None,
                                def: None,
                                witness: None,
                                parent_block: block_id,
                            };
                            return ctx.add_variable(new_var, None);
                        } else {
                            return id;
                        }
                    }
                    function::SSAFunction::get_mapped_value(Some(&id), ctx, inline_map, block_id)
                });
            }
            //However we deliberately not use the default case to force review of the behavior if a new type of operation is added.
            //These types do not handle arrays:
            Operation::Cast(_) | Operation::Truncate { .. } | Operation::Not(_) | Operation::Nop
            | Operation::Jne(_,_) | Operation::Jeq(_,_) | Operation::Jmp(_) |  Operation::Phi { .. } | Operation::Cond { .. }
            //These types handle arrays via their return type (done in inline_in_block)
            |  Operation::Result { .. }
            //These types handle arrays in a specific way (done in inline_in_block)
            | Operation::Return(_) | Operation::Load {.. } | Operation::Store { .. } | Operation::Call { .. }
            => {
                self.map_id_mut(|id| {
                    function::SSAFunction::get_mapped_value(Some(&id), ctx, inline_map, block_id)
                });
            }
        }
    }
}
