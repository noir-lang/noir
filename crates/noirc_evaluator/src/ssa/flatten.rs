use super::{
    block,
    code_gen::IRGenerator,
    node::{self, Node, NodeEval, Operation},
    optim,
};
use acvm::FieldElement;
use arena::Index;
use std::collections::HashMap;

//Unroll the CFG
pub fn unroll_tree(eval: &mut IRGenerator) {
    //Calls outer_unroll() from the root node
    let mut id = eval.first_block;
    let mut unroll_ins: Vec<Index> = Vec::new();
    let mut eval_map: HashMap<Index, node::NodeEval> = HashMap::new();
    let mut from = id;
    while let Some(next) = outer_unroll(&mut unroll_ins, &mut eval_map, id, from, eval) {
        from = id;
        id = next;
    }
}

//Update the block instruction list using the eval_map
fn eval_block(block_id: Index, eval_map: &HashMap<Index, NodeEval>, eval: &mut IRGenerator) {
    let block = eval.get_block(block_id);
    for i in &block.instructions.clone() {
        //RIA
        if let Some(ins) = eval.try_get_mut_instruction(*i) {
            if eval_map.contains_key(&ins.rhs) {
                ins.rhs = eval_map[&ins.rhs].to_index().unwrap();
            }
            if eval_map.contains_key(&ins.lhs) {
                ins.lhs = eval_map[&ins.lhs].to_index().unwrap();
            }
            if ins.operator == Operation::eq_gate {}
            //TODO simplify(eval, ins);
        }
    }
}

pub fn unroll_block(
    unroll_ins: &mut Vec<Index>, //unrolled instructions
    eval_map: &mut HashMap<Index, node::NodeEval>,
    block_id: Index, //block to unroll
    caller: Index,   //previous block
    eval: &mut IRGenerator,
) -> Option<Index> {
    let block = eval.get_block(block_id);
    if block.is_join() {
        return unroll_join(unroll_ins, eval_map, block_id, caller, eval);
    } else if let Some(i) = unroll_std_block(unroll_ins, eval_map, block_id, caller, eval) {
        if let Some(ins) = eval.try_get_instruction(i) {
            return Some(ins.parent_block);
        }
    }

    None
}

//unroll a normal block by generating new instructions into the unroll_ins list, using and updating the eval_map
pub fn unroll_std_block(
    unroll_ins: &mut Vec<Index>, //unrolled instructions
    eval_map: &mut HashMap<Index, node::NodeEval>,
    block_id: Index, //block to unroll
    _caller: Index,  //previous block  TODO to check whether it is needed
    eval: &mut IRGenerator,
) -> Option<Index> //first instruction of the left block
{
    let block = eval.get_block(block_id);
    let b_instructions = block.instructions.clone();
    let mut next = None;
    if let Some(left) = block.left {
        let left_block = eval.get_block(left);
        if let Some(f) = left_block.instructions.first() {
            next = Some(*f);
        }
    }
    for i_id in &b_instructions {
        let ins = eval.get_object(*i_id).unwrap();
        match ins {
            node::NodeObj::Instr(i) => {
                let new_left = get_current_value(i.lhs, eval_map).to_index().unwrap();
                let new_right = get_current_value(i.rhs, eval_map).to_index().unwrap();
                let mut new_ins = node::Instruction::new(
                    i.operator, new_left, new_right, i.res_type, None, //TODO to fix later
                );
                match i.operator {
                    node::Operation::ass => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    node::Operation::jmp => {
                        return Some(i.rhs);
                    }
                    node::Operation::nop => (),
                    _ => {
                        optim::simplify(eval, &mut new_ins);
                        let result_id;
                        let mut to_delete = false;
                        if new_ins.is_deleted {
                            result_id = new_ins.rhs;
                            if new_ins.rhs == new_ins.idx {
                                to_delete = true;
                            }
                        } else {
                            result_id = eval.nodes.insert(node::NodeObj::Instr(new_ins));
                            unroll_ins.push(result_id);
                        }
                        //ignore self-deleted instructions
                        if !to_delete {
                            eval_map.insert(*i_id, node::NodeEval::Idx(result_id));
                        }
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
    unroll_ins: &mut Vec<Index>, //unrolled instructions
    eval_map: &mut HashMap<Index, node::NodeEval>,
    block_id: Index, //block to unroll
    _caller: Index,  //previous block TODO is it needed?
    eval: &mut IRGenerator,
) -> Option<Index> {
    //Returns the exit block of the loop
    let join = eval.get_block(block_id);
    let join_instructions = join.instructions.clone();
    let join_left = join.left; //XXX.clone();
    let prev = *join.predecessor.first().unwrap(); //todo predecessor.first or .last?

    let mut from = prev; //todo caller?
    assert!(join.is_join());
    let body_id = join.right.unwrap();
    if unroll_ins.is_empty() {
        unroll_ins.push(*join_instructions.first().unwrap()); //TODO is it needed? we also should assert it is a nop instruction.
    }
    let mut processed: Vec<Index> = Vec::new();
    while {
        //evaluate the join  block:
        evaluate_phi(&join_instructions, from, eval_map, eval);
        evaluate_conditional_jump(*join_instructions.last().unwrap(), eval_map, eval)
    } {
        from = block_id;
        let mut b_id = body_id;
        let mut next;
        while {
            next = unroll_block(unroll_ins, eval_map, b_id, from, eval);
            processed.push(b_id);
            next.is_some()
        } {
            //process next block:
            from = b_id;
            b_id = next.unwrap();
            if b_id == block_id {
                //looping back to the join block; we are done
                break;
            }
        }
    }
    join_left
}

// Unrolling outer loops, i.e non-nested loops
pub fn outer_unroll(
    unroll_ins: &mut Vec<Index>, //unrolled instructions
    eval_map: &mut HashMap<Index, node::NodeEval>,
    block_id: Index, //block to unroll
    caller: Index,   //previous block
    eval: &mut IRGenerator,
) -> Option<Index> //next block
{
    assert!(unroll_ins.is_empty());
    let block = eval.get_block(block_id);
    let b_right = block.right;
    let b_left = block.left;
    let block_instructions = block.instructions.clone();
    if block.is_join() {
        //1. unroll the block into the unroll_ins
        unroll_join(unroll_ins, eval_map, block_id, caller, eval);
        //2. map the Phis variables to their unrolled values:
        for ins in &block_instructions {
            if let Some(ins_obj) = eval.try_get_instruction(*ins) {
                if ins_obj.operator == node::Operation::phi {
                    if eval_map.contains_key(&ins_obj.rhs) {
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.rhs]);
                        //todo test with constants
                    } else if eval_map.contains_key(&ins_obj.idx) {
                        //   unroll_map.insert(ins_obj.lhs, eval_map[&ins_obj.idx].to_index().unwrap());
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.idx]);
                        //todo test with constants
                    }
                } else if ins_obj.operator != node::Operation::nop {
                    break; //no more phis
                }
            }
        }
        //3. Merge the unrolled blocks into the join
        for ins in &unroll_ins.clone() {
            //TODO this clone should not be needed...
            let ins_obj = eval.try_get_mut_instruction(*ins);
            ins_obj.unwrap().idx = *ins;
        }
        let join_mut = eval.get_block_mut(block_id).unwrap();
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
            let sub_graph = block::bfs(body_id, block_id, eval);
            for b in sub_graph {
                eval.blocks.remove(b);
            }
        }

        //5.Finally we clear the unroll_list and go the the next block
        unroll_ins.clear();
    } else {
        //We update block instructions from the eval_map
        eval_block(block_id, eval_map, eval);
    }
    b_left //returns the next block to process
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[arena::Index],
    from: Index,
    to: &mut HashMap<Index, node::NodeEval>,
    eval: &mut IRGenerator,
) {
    for i in instructions {
        let mut to_process: Vec<(Index, node::NodeEval)> = Vec::new();
        if let Some(ins) = eval.try_get_instruction(*i) {
            for phi in &ins.phi_arguments {
                if phi.1 == from {
                    //we evaluate the phi instruction value
                    to_process.push((ins.idx, evaluate_one(node::NodeEval::Idx(phi.0), to, eval)));
                }
            }
            if ins.operator != node::Operation::phi && ins.operator != node::Operation::nop {
                break; //phi instructions are placed at the beginning (and after the first dummy instruction)
            }
        }
        //Update the evaluation map.
        for obj in to_process {
            to.insert(obj.0, node::NodeEval::Idx(optim::to_index(eval, obj.1)));
        }
    }
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: Index,
    value_array: &mut HashMap<arena::Index, node::NodeEval>,
    eval: &IRGenerator,
) -> bool {
    let jump_ins = eval.try_get_instruction(jump).unwrap();
    let lhs = get_current_value(jump_ins.lhs, value_array);
    let cond = evaluate_object(lhs, value_array, eval);
    if let Some(cond_const) = cond.to_const_value() {
        let result = !cond_const.is_zero();
        match jump_ins.operator {
            node::Operation::jeq => return result,
            node::Operation::jne => return !result,
            node::Operation::jmp => return true,
            _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
        }
    }

    unreachable!("Condition should be constant");
}

//Retrieve the NodeEval value of the index in the evaluation map
fn get_current_value(idx: Index, value_array: &HashMap<arena::Index, NodeEval>) -> NodeEval {
    if value_array.contains_key(&idx) {
        return value_array[&idx];
    }
    node::NodeEval::Idx(idx)
}

//Same as get_current_value but for a NodeEval object instead of a NodeObj
fn get_current_value_for_node_eval(
    obj: NodeEval,
    value_array: &HashMap<arena::Index, NodeEval>,
) -> NodeEval {
    match obj {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(obj_id) => get_current_value(obj_id, value_array),
    }
}

//evaluate the object without recursion, doing only one step of evaluation
fn evaluate_one(
    obj: node::NodeEval,
    value_array: &HashMap<arena::Index, NodeEval>,
    eval: &IRGenerator,
) -> node::NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(obj_id) => {
            if eval.get_object(obj_id).is_none() {
                return obj;
            }
            let value = eval.get_object(obj_id).unwrap();
            match value {
                node::NodeObj::Instr(i) => {
                    if i.operator == node::Operation::phi {
                        //n.b phi are handled before, else we should know which block we come from
                        dbg!(i.idx);
                        return node::NodeEval::Idx(i.idx);
                    }

                    let lhs = get_current_value(i.lhs, value_array);
                    let lhr = get_current_value(i.rhs, value_array);
                    let result = i.evaluate(&lhs, &lhr);
                    if let node::NodeEval::Idx(idx) = result {
                        if eval.get_object(idx).is_none() {
                            return node::NodeEval::Idx(obj_id);
                        }
                    }
                    result
                }
                node::NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    node::NodeEval::Const(value, c.get_type())
                }
                _ => node::NodeEval::Idx(obj_id),
            }
        }
    }
}

//Evaluate an object recursively
fn evaluate_object(
    obj: node::NodeEval,
    value_array: &HashMap<arena::Index, NodeEval>,
    eval: &IRGenerator,
) -> node::NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(obj_id) => {
            if eval.get_object(obj_id).is_none() {
                dbg!(obj_id);
                return obj;
            }
            let value = eval.get_object(obj_id).unwrap();
            // dbg!(value);
            match value {
                node::NodeObj::Instr(i) => {
                    if i.operator == node::Operation::phi {
                        dbg!(i.idx);
                        return node::NodeEval::Idx(i.idx);
                    }
                    //n.b phi are handled before, else we should know which block we come from
                    let lhs =
                        evaluate_object(get_current_value(i.lhs, value_array), value_array, eval);
                    let lhr =
                        evaluate_object(get_current_value(i.rhs, value_array), value_array, eval);
                    let result = i.evaluate(&lhs, &lhr);
                    if let node::NodeEval::Idx(idx) = result {
                        if eval.get_object(idx).is_none() {
                            return node::NodeEval::Idx(obj_id);
                        }
                    }
                    result
                }
                node::NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    node::NodeEval::Const(value, c.get_type())
                }
                _ => node::NodeEval::Idx(obj_id),
            }
        }
    }
}
