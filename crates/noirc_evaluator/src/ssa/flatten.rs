use super::{
    block::{self, BlockId},
    context::SsaContext,
    node::{self, Node, NodeEval, NodeId, NodeObj, Operation, BinaryOp},
    optim,
};
use acvm::FieldElement;
use std::collections::HashMap;

//returns the NodeObj index of a NodeEval object
//if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
fn to_index(ctx: &mut SsaContext, obj: NodeEval) -> NodeId {
    match obj {
        NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    }
}

//Unroll the CFG
pub fn unroll_tree(ctx: &mut SsaContext) {
    //Calls outer_unroll() from the root node
    let mut id = ctx.first_block;
    let mut unroll_ins = Vec::new();
    let mut eval_map = HashMap::new();
    while let Some(next) = outer_unroll(&mut unroll_ins, &mut eval_map, id, ctx) {
        id = next;
    }
}

//Update the block instruction list using the eval_map
fn eval_block(block_id: BlockId, eval_map: &HashMap<NodeId, NodeEval>, ctx: &mut SsaContext) {
    for i in &ctx[block_id].instructions.clone() {
        if let Some(ins) = ctx.try_get_mut_instruction(*i) {
            ins.operator = update_operator(ctx, ins.operator, eval_map);
            // TODO: simplify(ctx, ins);
        }
    }
}

fn update_operator(ctx: &mut SsaContext, operator: Operation, eval_map: &HashMap<NodeId, NodeEval>) -> Operation {
    let update = |id| {
        eval_map.get(&id)
            .and_then(|value| value.into_node_id())
            .unwrap_or(id)
    };

    operator.map_id(update)
}

pub fn unroll_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    ctx: &mut SsaContext,
) -> Option<BlockId> {
    if ctx[block_to_unroll].is_join() {
        unroll_join(unrolled_instructions, eval_map, block_to_unroll, ctx)
    } else {
        unroll_std_block(unrolled_instructions, eval_map, block_to_unroll, ctx)
    }
}

//unroll a normal block by generating new instructions into the unroll_ins list, using and updating the eval_map
pub fn unroll_std_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    ctx: &mut SsaContext,
) -> Option<BlockId> // The left block
{
    let block = &ctx[block_to_unroll];
    let b_instructions = block.instructions.clone();
    let next = block.left;

    for i_id in &b_instructions {
        match &ctx[*i_id] {
            node::NodeObj::Instr(i) => {
                let new_op = i.operator.map_id(|id| get_current_value(id, eval_map).into_node_id().unwrap());
                let mut new_ins = node::Instruction::new(
                    new_op, i.res_type, None, //TODO to fix later
                );
                match i.operator {
                    Operation::Binary(node::Binary { operator: BinaryOp::Assign, .. }) => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    Operation::Jmp(block) => {
                        return Some(block);
                    }
                    Operation::Nop => (),
                    _ => {
                        let replacement = optim::simplify(ctx, &mut new_ins);

                        let result_id = if let Some(replacement) = replacement {
                            replacement
                        } else {
                            let id = ctx.add_instruction(new_ins);
                            unrolled_instructions.push(id);
                            id
                        };
                        eval_map.insert(*i_id, NodeEval::VarOrInstruction(result_id));
                    }
                }
            }
            _ => todo!(), //ERROR
        }
    }
    next
}

//Unroll a for loop: exit <- join <--> body
//join block is given in argumemt, it will evaluate the join condition, starting from 'start' until it reaches 'end'
//and write the unrolled instructions of the body block into the unroll_ins list
//If the body does not ends with a jump back to the join block, we continue to unroll the next block, until we reach the join.
//If there is a nested loop, unroll_block will call recursively unroll_join, keeping unroll list and eval map from the previous one
pub fn unroll_join(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    ctx: &mut SsaContext,
) -> Option<BlockId> {
    //Returns the exit block of the loop
    let join = &ctx[block_to_unroll];
    let join_instructions = join.instructions.clone();
    let join_left = join.left; //XXX.clone();
    let prev = *join.predecessor.first().unwrap(); //todo predecessor.first or .last?

    let mut from = prev; //todo caller?
    assert!(join.is_join());
    let body_id = join.right.unwrap();
    if unrolled_instructions.is_empty() {
        unrolled_instructions.push(*join_instructions.first().unwrap()); //TODO is it needed? we also should assert it is a nop instruction.
    }
    while {
        //evaluate the join  block:
        evaluate_phi(&join_instructions, from, eval_map, ctx);
        evaluate_conditional_jump(*join_instructions.last().unwrap(), eval_map, ctx)
    } {
        from = block_to_unroll;
        let mut b_id = body_id;
        while let Some(next) = unroll_block(unrolled_instructions, eval_map, b_id, ctx) {
            //process next block:
            from = b_id;
            b_id = next;
            if b_id == block_to_unroll {
                //looping back to the join block; we are done
                break;
            }
        }
    }
    join_left
}

// Unrolling outer loops, i.e non-nested loops
pub fn outer_unroll(
    unroll_ins: &mut Vec<NodeId>, //unrolled instructions
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_id: BlockId, //block to unroll
    ctx: &mut SsaContext,
) -> Option<BlockId> //next block
{
    assert!(unroll_ins.is_empty());
    let block = &ctx[block_id];
    let b_right = block.right;
    let b_left = block.left;
    let block_instructions = block.instructions.clone();
    if block.is_join() {
        //1. unroll the block into the unroll_ins
        unroll_join(unroll_ins, eval_map, block_id, ctx);
        //2. map the Phis variables to their unrolled values:
        for ins in &block_instructions {
            if let Some(ins_obj) = ctx.try_get_instruction(*ins) {
                if let Operation::Phi { root, block_args } = ins_obj.operator {
                    if eval_map.contains_key(&root) {
                        eval_map.insert(root, eval_map[&root]);
                        //todo test with constants
                    } else if eval_map.contains_key(&ins_obj.id) {
                        //   unroll_map.insert(ins_obj.lhs, eval_map[&ins_obj.idx].to_index().unwrap());
                        eval_map.insert(root, eval_map[&ins_obj.id]);
                        //todo test with constants
                    }
                } else if ins_obj.operator != node::Operation::Nop {
                    break; //no more phis
                }
            }
        }
        //3. Merge the unrolled blocks into the join
        for ins in unroll_ins.iter() {
            ctx[*ins].set_id(*ins);
        }
        let join_mut = &mut ctx[block_id];
        join_mut.instructions = unroll_ins.clone();
        join_mut.right = None;
        join_mut.kind = block::BlockType::Normal;

        //4. Remove the right sub-graph of the join block
        //Note that this is not done in unroll_join because in case of nested loops we need to unroll_join a block several times.
        if b_left.is_some() {
            join_mut.dominated = vec![b_left.unwrap()];
        } else {
            join_mut.dominated.clear();
        }
        //we get the subgraph, however we could retrieve the list of processed blocks directly in unroll_join (cf. processed)
        if let Some(body_id) = b_right {
            let sub_graph = block::bfs(body_id, block_id, ctx);
            for b in sub_graph {
                ctx.remove_block(b);
            }
        }

        //5.Finally we clear the unroll_list and go the the next block
        unroll_ins.clear();
    } else {
        //We update block instructions from the eval_map
        eval_block(block_id, eval_map, ctx);
    }
    b_left //returns the next block to process
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[NodeId],
    from: BlockId,
    to: &mut HashMap<NodeId, NodeEval>,
    ctx: &mut SsaContext,
) {
    for i in instructions {
        let mut to_process = Vec::new();
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if let Operation::Phi { root, block_args } = ins.operator {
                for phi in &block_args {
                    if phi.1 == from {
                        //we evaluate the phi instruction value
                        to_process.push((
                            ins.id,
                            evaluate_one(NodeEval::VarOrInstruction(phi.0), to, ctx),
                        ));
                    }
                }
            } else if ins.operator != node::Operation::Nop {
                break; //phi instructions are placed at the beginning (and after the first dummy instruction)
            }
        }
        //Update the evaluation map.
        for obj in to_process {
            to.insert(
                obj.0,
                NodeEval::VarOrInstruction(to_index(ctx, obj.1)),
            );
        }
    }
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: NodeId,
    value_array: &mut HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> bool {
    let jump_ins = ctx.try_get_instruction(jump).unwrap();

    let cond_id = match jump_ins.operator {
        Operation::Jeq(cond_id, block) => cond_id,
        Operation::Jne(cond_id, block) => cond_id,
        Operation::Jmp(block) => return true,
        _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
    };

    let cond = get_current_value(cond_id, value_array);
    let cond = evaluate_object(cond, value_array, ctx).into_const_value().unwrap();

    match jump_ins.operator {
        node::Operation::Jeq { .. } => cond.is_zero(),
        node::Operation::Jne { .. } => !cond.is_zero(),
        _ => unreachable!(),
    }
}

//Retrieve the NodeEval value of the index in the evaluation map
fn get_current_value(id: NodeId, value_array: &HashMap<NodeId, NodeEval>) -> NodeEval {
    *value_array
        .get(&id)
        .unwrap_or(&NodeEval::VarOrInstruction(id))
}

//Same as get_current_value but for a NodeEval object instead of a NodeObj
fn get_current_value_for_node_eval(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
) -> NodeEval {
    match obj {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => get_current_value(obj_id, value_array),
    }
}

//evaluate the object without recursion, doing only one step of evaluation
fn evaluate_one(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                return obj;
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    if let Operation::Phi { .. } = i.operator {
                        //n.b phi are handled before, else we should know which block we come from
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    let result = i.evaluate(ctx);
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
                            return NodeEval::VarOrInstruction(obj_id);
                        }
                    }
                    result
                }
                NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    NodeEval::Const(value, c.get_type())
                }
                NodeObj::Obj(_) => NodeEval::VarOrInstruction(obj_id),
            }
        }
    }
}

//Evaluate an object recursively
fn evaluate_object(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                dbg!(obj_id);
                return obj;
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    if let Operation::Phi { .. } = i.operator {
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    //n.b phi are handled before, else we should know which block we come from
                    let result = i.evaluate_with(ctx, |ctx, id| {
                        evaluate_object(get_current_value(id, value_array), value_array, ctx)
                    });

                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
                            return NodeEval::VarOrInstruction(obj_id);
                        }
                    }
                    result
                }
                NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    NodeEval::Const(value, c.get_type())
                }
                NodeObj::Obj(_) => NodeEval::VarOrInstruction(obj_id),
            }
        }
    }
}
