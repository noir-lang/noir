use crate::errors::RuntimeError;

use super::{
    block::{self, BlockId},
    context::SsaContext,
    node::{self, BinaryOp, Mark, Node, NodeEval, NodeId, NodeObj, Operation},
    optim,
};
use acvm::FieldElement;
use std::collections::HashMap;

//Unroll the CFG
pub fn unroll_tree(
    ctx: &mut SsaContext,
    block_id: BlockId,
) -> Result<HashMap<NodeId, NodeEval>, RuntimeError> {
    //call unroll_tree from the root
    assert!(!ctx[block_id].is_join());
    let mut unroll_ctx = UnrollContext {
        deprecated: Vec::new(),
        to_unroll: block_id,
        unroll_into: block_id,
        eval_map: HashMap::new(),
    };
    while !unroll_ctx.to_unroll.is_dummy() {
        unroll_block(ctx, &mut unroll_ctx)?;
    }
    //clean-up
    for b in unroll_ctx.deprecated {
        debug_assert!(b != ctx.first_block);
        ctx.remove_block(b);
    }
    block::compute_dom(ctx);

    Ok(unroll_ctx.eval_map)
}

//Update the block instruction list using the eval_map
fn eval_block(block_id: BlockId, eval_map: &HashMap<NodeId, NodeEval>, ctx: &mut SsaContext) {
    for i in &ctx[block_id].instructions.clone() {
        if let Some(ins) = ctx.try_get_mut_instruction(*i) {
            ins.operation = update_operator(&ins.operation, eval_map);
            let ins_id = ins.id;
            // We ignore RunTimeErrors at this stage because unrolling is done before conditionals
            // While failures must be managed after handling conditionals: For instance if false { b } should not fail whatever b is doing.
            optim::simplify_id(ctx, ins_id).ok();
        }
    }
}

fn update_operator(operator: &Operation, eval_map: &HashMap<NodeId, NodeEval>) -> Operation {
    operator.map_id(|id| eval_map.get(&id).and_then(|value| value.into_node_id()).unwrap_or(id))
}

//Unroll from unroll_ctx.to_unroll until it reaches unroll_ctx.unroll_into
pub fn unroll_until(
    ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
    end: BlockId,
) -> Result<BlockId, RuntimeError> {
    let mut b = unroll_ctx.to_unroll;
    let mut prev = BlockId::dummy();

    while b != end {
        assert!(!b.is_dummy(), "could not reach end block");
        prev = b;
        unroll_block(ctx, unroll_ctx)?;
        b = unroll_ctx.to_unroll;
    }
    Ok(prev)
}

pub fn unroll_block(
    ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
) -> Result<(), RuntimeError> {
    match ctx[unroll_ctx.to_unroll].kind {
        block::BlockType::ForJoin => {
            unroll_join(ctx, unroll_ctx)?;
        }
        _ => {
            if ctx[unroll_ctx.to_unroll].right.is_some() {
                if unroll_ctx.unroll_into == BlockId::dummy() {
                    unroll_ctx.unroll_into = unroll_ctx.to_unroll;
                }
                crate::ssa::conditional::unroll_if(ctx, unroll_ctx)?;
            } else {
                unroll_std_block(ctx, unroll_ctx)?;
            }
        }
    }
    Ok(())
}

//unroll a normal block by generating new instructions into the target block, or by updating its instructions if no target is specified, using and updating the eval_map
pub fn unroll_std_block(
    ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
) -> Result<Option<BlockId>, RuntimeError> // The left block
{
    if unroll_ctx.to_unroll == unroll_ctx.unroll_into {
        //We update block instructions from the eval_map
        eval_block(unroll_ctx.to_unroll, &unroll_ctx.eval_map, ctx);
        let block = &ctx[unroll_ctx.to_unroll];
        unroll_ctx.to_unroll = block.left.unwrap_or_else(BlockId::dummy);
        return Ok(block.left);
    }
    let block = &ctx[unroll_ctx.to_unroll];
    let b_instructions = block.instructions.clone();
    let next = block.left.unwrap_or_else(BlockId::dummy);
    ctx.current_block = unroll_ctx.unroll_into;

    for i_id in &b_instructions {
        match &ctx[*i_id] {
            node::NodeObj::Instr(i) => {
                let new_op = i.operation.map_id(|id| {
                    get_current_value(id, &unroll_ctx.eval_map).into_node_id().unwrap()
                });
                let mut new_ins =
                    node::Instruction::new(new_op, i.res_type, Some(unroll_ctx.unroll_into));
                match i.operation {
                    Operation::Binary(node::Binary { operator: BinaryOp::Assign, .. }) => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    Operation::Jmp(block) => assert_eq!(block, next),
                    Operation::Nop => (),
                    _ => {
                        optim::simplify(ctx, &mut new_ins).ok(); //ignore RuntimeErrors until conditionals are processed
                        match new_ins.mark {
                            Mark::None => {
                                let id = ctx.push_instruction(new_ins);
                                unroll_ctx.eval_map.insert(*i_id, NodeEval::VarOrInstruction(id));
                            }
                            Mark::Deleted => (),
                            Mark::ReplaceWith(replacement) => {
                                // TODO: Should we insert into unrolled_instructions as well?
                                // If optim::simplify replaces with a constant then we should not,
                                // otherwise it may make sense if it is not already inserted.
                                unroll_ctx
                                    .eval_map
                                    .insert(*i_id, NodeEval::VarOrInstruction(replacement));
                            }
                        }
                    }
                }
            }
            _ => unreachable!("Block instruction list should only only contain instruction"),
        }
    }
    if unroll_ctx.to_unroll != unroll_ctx.unroll_into
        && !unroll_ctx.deprecated.contains(&unroll_ctx.to_unroll)
    {
        unroll_ctx.deprecated.push(unroll_ctx.to_unroll);
    }
    unroll_ctx.to_unroll = next;
    Ok(Some(next))
}

pub fn unroll_join(
    ssa_ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
) -> Result<BlockId, RuntimeError> {
    let join_id = unroll_ctx.to_unroll;
    let join = &ssa_ctx[unroll_ctx.to_unroll];

    let r = join.right.unwrap();

    let join_instructions = join.instructions.clone();
    let join_left = join.left.unwrap();
    let mut prev = *join.predecessor.first().unwrap();

    let mut from = prev;
    assert!(join.is_join());
    let body_id = join.right.unwrap();
    let end = unroll_ctx.to_unroll;
    if !unroll_ctx.unroll_into.is_dummy() {
        prev = unroll_ctx.unroll_into;
    }
    ssa_ctx.current_block = prev;
    let new_body = block::new_sealed_block(ssa_ctx, block::BlockType::Normal, true);
    let prev_block = ssa_ctx.try_get_block_mut(prev).unwrap();
    prev_block.dominated = vec![new_body];
    unroll_ctx.unroll_into = new_body;

    while {
        //evaluate the join  block:
        evaluate_phi(&join_instructions, from, &mut unroll_ctx.eval_map, ssa_ctx)?;
        evaluate_conditional_jump(
            *join_instructions.last().unwrap(),
            &unroll_ctx.eval_map,
            ssa_ctx,
        )?
    } {
        unroll_ctx.to_unroll = body_id;
        from = unroll_until(ssa_ctx, unroll_ctx, end)?;
    }
    debug_assert!(ssa_ctx.current_block == unroll_ctx.unroll_into);
    let next_block = block::new_sealed_block(ssa_ctx, block::BlockType::Normal, true);
    unroll_ctx.deprecate(join_id);
    unroll_ctx.deprecate(r);

    unroll_ctx.unroll_into = next_block;
    unroll_ctx.to_unroll = join_left;
    Ok(join_left)
}

#[derive(Debug)]
pub struct UnrollContext {
    pub deprecated: Vec<BlockId>,
    pub to_unroll: BlockId,
    pub unroll_into: BlockId,
    pub eval_map: HashMap<NodeId, NodeEval>,
}

impl UnrollContext {
    pub fn deprecate(&mut self, block_id: BlockId) {
        if !self.deprecated.contains(&block_id) {
            self.deprecated.push(block_id);
        }
    }
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[NodeId],
    from: BlockId,
    to: &mut HashMap<NodeId, NodeEval>,
    ctx: &mut SsaContext,
) -> Result<(), RuntimeError> {
    let mut to_process = Vec::new();
    for i in instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if let Operation::Phi { block_args, .. } = &ins.operation {
                for (arg, block) in block_args {
                    if *block == from {
                        //we evaluate the phi instruction value
                        let arg = *arg;
                        let id = ins.id;
                        to_process
                            .push((id, evaluate_one(NodeEval::VarOrInstruction(arg), to, ctx)?));
                    }
                }
            } else if ins.operation != node::Operation::Nop {
                break; //phi instructions are placed at the beginning (and after the first dummy instruction)
            }
        }
    }
    //Update the evaluation map.
    for obj in to_process {
        to.insert(obj.0, NodeEval::VarOrInstruction(obj.1.to_index(ctx)));
    }
    Ok(())
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: NodeId,
    value_array: &HashMap<NodeId, NodeEval>,
    ctx: &mut SsaContext,
) -> Result<bool, RuntimeError> {
    let jump_ins = ctx.try_get_instruction(jump).unwrap();

    let (cond_id, should_jump): (_, fn(FieldElement) -> bool) = match jump_ins.operation {
        Operation::Jeq(cond_id, _) => (cond_id, |field| !field.is_zero()),
        Operation::Jne(cond_id, _) => (cond_id, |field| field.is_zero()),
        Operation::Jmp(_) => return Ok(true),
        _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
    };

    let cond = get_current_value(cond_id, value_array);
    let cond = match evaluate_object(cond, value_array, ctx)?.into_const_value() {
        Some(c) => c,
        None => unreachable!(
            "Conditional jump argument is non-const: {:?}",
            evaluate_object(cond, value_array, ctx)
        ),
    };

    Ok(should_jump(cond))
}

//Retrieve the NodeEval value of the index in the evaluation map
fn get_current_value(id: NodeId, value_array: &HashMap<NodeId, NodeEval>) -> NodeEval {
    *value_array.get(&id).unwrap_or(&NodeEval::VarOrInstruction(id))
}

//Same as get_current_value but for a NodeEval object instead of a NodeObj
fn get_current_value_for_node_eval(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
) -> NodeEval {
    match obj {
        NodeEval::VarOrInstruction(obj_id) => get_current_value(obj_id, value_array),
        NodeEval::Const(..) | NodeEval::Function(..) => obj,
    }
}

//evaluate the object without recursion, doing only one step of evaluation
fn evaluate_one(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> Result<NodeEval, RuntimeError> {
    let mut modified = false;
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(..) | NodeEval::Function(..) => Ok(obj),
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                return Ok(obj);
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    let new_id = optim::propagate(ctx, obj_id, &mut modified);
                    if new_id != obj_id {
                        return evaluate_one(NodeEval::VarOrInstruction(new_id), value_array, ctx);
                    }
                    if let Operation::Phi { .. } = i.operation {
                        //n.b phi are handled before, else we should know which block we come from
                        return Ok(NodeEval::VarOrInstruction(i.id));
                    }

                    let result =
                        i.evaluate_with(ctx, |_, id| Ok(get_current_value(id, value_array)))?;

                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
                            return Ok(NodeEval::VarOrInstruction(obj_id));
                        }
                    }
                    Ok(result)
                }
                NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    Ok(NodeEval::Const(value, c.get_type()))
                }
                NodeObj::Obj(_) => Ok(NodeEval::VarOrInstruction(obj_id)),
                NodeObj::Function(f, id, typ) => Ok(NodeEval::Function(*f, *id, *typ)),
            }
        }
    }
}

//Evaluate an object recursively
fn evaluate_object(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> Result<NodeEval, RuntimeError> {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => Ok(obj),
        NodeEval::Function(..) => Ok(obj),
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                return Ok(obj);
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    if let Operation::Phi { .. } = i.operation {
                        return Ok(NodeEval::VarOrInstruction(i.id));
                    }

                    //n.b phi are handled before, else we should know which block we come from
                    let result = i.evaluate_with(ctx, |ctx, id| {
                        evaluate_object(get_current_value(id, value_array), value_array, ctx)
                    })?;

                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
                            return Ok(NodeEval::VarOrInstruction(obj_id));
                        }
                    }
                    Ok(result)
                }
                NodeObj::Const(c) => {
                    // TODO: Is this needed? Can't we just .clone() here?
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    Ok(NodeEval::Const(value, c.get_type()))
                }
                NodeObj::Obj(_) => Ok(NodeEval::VarOrInstruction(obj_id)),
                NodeObj::Function(f, id, typ) => Ok(NodeEval::Function(*f, *id, *typ)),
            }
        }
    }
}
