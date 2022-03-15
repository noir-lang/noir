use super::{
    block::{self, BlockId},
    code_gen::IRGenerator,
    function,
    node::{self, Node, NodeEval, NodeId, NodeObj, Operation},
    optim,
};
use acvm::FieldElement;
use std::collections::HashMap;

//Unroll the CFG
pub fn unroll_tree(eval: &mut IRGenerator) {
    //Calls outer_unroll() from the root node
    let mut id = eval.first_block;
    let mut unroll_ins = Vec::new();
    let mut eval_map = HashMap::new();
    while let Some(next) = outer_unroll(&mut unroll_ins, &mut eval_map, id, eval) {
        id = next;
    }
}

//Update the block instruction list using the eval_map
fn eval_block(block_id: BlockId, eval_map: &HashMap<NodeId, NodeEval>, igen: &mut IRGenerator) {
    for i in &igen[block_id].instructions.clone() {
        //RIA
        if let Some(ins) = igen.try_get_mut_instruction(*i) {
            if let Some(value) = eval_map.get(&ins.rhs) {
                ins.rhs = value.into_node_id().unwrap();
            }
            if let Some(value) = eval_map.get(&ins.lhs) {
                ins.lhs = value.into_node_id().unwrap();
            }
            //TODO simplify(eval, ins);
        }
    }
}

pub fn unroll_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut IRGenerator,
) -> Option<BlockId> {
    if igen[block_to_unroll].is_join() {
        unroll_join(unrolled_instructions, eval_map, block_to_unroll, igen)
    } else if let Some(i) = unroll_std_block(unrolled_instructions, eval_map, block_to_unroll, igen)
    {
        igen.try_get_instruction(i).map(|ins| ins.parent_block)
    } else {
        None
    }
}

//unroll a normal block by generating new instructions into the unroll_ins list, using and updating the eval_map
pub fn unroll_std_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut IRGenerator,
) -> Option<NodeId> //first instruction of the left block
{
    let block = &igen[block_to_unroll];
    let b_instructions = block.instructions.clone();
    let mut next = None;
    if let Some(left) = block.left {
        if let Some(f) = igen[left].instructions.first() {
            next = Some(*f);
        }
    }
    for i_id in &b_instructions {
        match &igen[*i_id] {
            node::NodeObj::Instr(i) => {
                let new_left = get_current_value(i.lhs, eval_map).into_node_id().unwrap();
                let new_right = get_current_value(i.rhs, eval_map).into_node_id().unwrap();
                let mut new_ins = node::Instruction::new(
                    i.operator, new_left, new_right, i.res_type, None, //TODO to fix later
                );
                match i.operator {
                    Operation::Ass => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    Operation::Jmp => {
                        return Some(i.rhs);
                    }
                    Operation::Nop => (),
                    _ => {
                        optim::simplify(igen, &mut new_ins);
                        let result_id;
                        let mut to_delete = false;
                        if new_ins.is_deleted {
                            result_id = new_ins.rhs;
                            if new_ins.rhs == new_ins.id {
                                to_delete = true;
                            }
                        } else {
                            result_id = igen.add_instruction(new_ins);
                            unrolled_instructions.push(result_id);
                        }
                        //ignore self-deleted instructions
                        if !to_delete {
                            eval_map.insert(*i_id, NodeEval::VarOrInstruction(result_id));
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
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut IRGenerator,
) -> Option<BlockId> {
    //Returns the exit block of the loop
    let join = &igen[block_to_unroll];
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
        evaluate_phi(&join_instructions, from, eval_map, igen);
        evaluate_conditional_jump(*join_instructions.last().unwrap(), eval_map, igen)
    } {
        from = block_to_unroll;
        let mut b_id = body_id;
        while let Some(next) = unroll_block(unrolled_instructions, eval_map, b_id, igen) {
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
    igen: &mut IRGenerator,
) -> Option<BlockId> //next block
{
    assert!(unroll_ins.is_empty());
    let block = &igen[block_id];
    let b_right = block.right;
    let b_left = block.left;
    let block_instructions = block.instructions.clone();
    if block.is_join() {
        //1. unroll the block into the unroll_ins
        unroll_join(unroll_ins, eval_map, block_id, igen);
        //2. map the Phis variables to their unrolled values:
        for ins in &block_instructions {
            if let Some(ins_obj) = igen.try_get_instruction(*ins) {
                if ins_obj.operator == node::Operation::Phi {
                    if eval_map.contains_key(&ins_obj.rhs) {
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.rhs]);
                        //todo test with constants
                    } else if eval_map.contains_key(&ins_obj.id) {
                        //   unroll_map.insert(ins_obj.lhs, eval_map[&ins_obj.idx].to_index().unwrap());
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.id]);
                        //todo test with constants
                    }
                } else if ins_obj.operator != node::Operation::Nop {
                    break; //no more phis
                }
            }
        }
        //3. Merge the unrolled blocks into the join
        for ins in unroll_ins.iter() {
            igen[*ins].set_id(*ins);
        }
        let join_mut = &mut igen[block_id];
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
            let sub_graph = block::bfs(body_id, block_id, igen);
            for b in sub_graph {
                igen.remove_block(b);
            }
        }

        //5.Finally we clear the unroll_list and go the the next block
        unroll_ins.clear();
    } else {
        //We update block instructions from the eval_map
        eval_block(block_id, eval_map, igen);
    }
    b_left //returns the next block to process
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[NodeId],
    from: BlockId,
    to: &mut HashMap<NodeId, NodeEval>,
    igen: &mut IRGenerator,
) {
    for i in instructions {
        let mut to_process = Vec::new();
        if let Some(ins) = igen.try_get_instruction(*i) {
            if ins.operator == node::Operation::Phi {
                for phi in &ins.phi_arguments {
                    if phi.1 == from {
                        //we evaluate the phi instruction value
                        to_process.push((
                            ins.id,
                            evaluate_one(NodeEval::VarOrInstruction(phi.0), to, igen),
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
                NodeEval::VarOrInstruction(optim::to_index(igen, obj.1)),
            );
        }
    }
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: NodeId,
    value_array: &mut HashMap<NodeId, NodeEval>,
    eval: &IRGenerator,
) -> bool {
    let jump_ins = eval.try_get_instruction(jump).unwrap();
    let lhs = get_current_value(jump_ins.lhs, value_array);
    let cond = evaluate_object(lhs, value_array, eval);
    if let Some(cond_const) = cond.into_const_value() {
        let result = !cond_const.is_zero();
        match jump_ins.operator {
            node::Operation::Jeq => return result,
            node::Operation::Jne => return !result,
            node::Operation::Jmp => return true,
            _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
        }
    }

    unreachable!("Condition should be constant");
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
    igen: &IRGenerator,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if igen.try_get_node(obj_id).is_none() {
                return obj;
            }

            match &igen[obj_id] {
                NodeObj::Instr(i) => {
                    if i.operator == node::Operation::Phi {
                        //n.b phi are handled before, else we should know which block we come from
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    let lhs = get_current_value(i.lhs, value_array);
                    let lhr = get_current_value(i.rhs, value_array);
                    let result = i.evaluate(&lhs, &lhr);
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if igen.try_get_node(idx).is_none() {
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
    igen: &IRGenerator,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if igen.try_get_node(obj_id).is_none() {
                dbg!(obj_id);
                return obj;
            }

            match &igen[obj_id] {
                NodeObj::Instr(i) => {
                    if i.operator == Operation::Phi {
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }
                    //n.b phi are handled before, else we should know which block we come from
                    let lhs =
                        evaluate_object(get_current_value(i.lhs, value_array), value_array, igen);
                    let lhr =
                        evaluate_object(get_current_value(i.rhs, value_array), value_array, igen);
                    let result = i.evaluate(&lhs, &lhr);
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if igen.try_get_node(idx).is_none() {
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

pub fn inline_tree(igen: &mut IRGenerator) {
    //inline all function calls
    //todo process tous les blocks!
    //vu qu'on fait ca plein de fois, y'a surement une maniere de faire generic..TODO!!
    //pour le moment, juste le premier block...
    inline_block(igen, igen.first_block);
}

//inline all function calls of the block
pub fn inline_block(igen: &mut IRGenerator, block_id: BlockId) {
    let mut call_ins: Vec<NodeId> = Vec::new();

    for i in &igen[block_id].instructions {
        if let Some(ins) = igen.try_get_instruction(*i) {
            if matches!(ins.operator, node::Operation::Call(_)) {
                call_ins.push(*i);
            }
        }
    }

    for ins_id in call_ins {
        let ins = igen.try_get_instruction(ins_id).unwrap().clone();
        if let node::Instruction {
            operator: node::Operation::Call(f),
            ..
        } = ins
        {
            inline(f, &ins.ins_arguments, igen, ins.parent_block, ins.id);
        }
    }
}

//inline a function call
pub fn inline(
    func_id: noirc_frontend::node_interner::FuncId, //func: &function::SSAFunction,
    args: &[NodeId],
    irgen: &mut IRGenerator,
    block: BlockId,
    call_id: NodeId,
) {
    let ssa_func = irgen.get_function(func_id).unwrap();
    let mut inline_map: HashMap<NodeId, NodeId> = HashMap::new(); //map nodes from the fonction cfg to the caller cfg
                                                                  //1. map function parameters
    for (arg_caller, arg_function) in args.iter().zip(&ssa_func.arguments) {
        inline_map.insert(*arg_function, *arg_caller);
    }
    // let func_block = &ssa_func.cfg[ssa_func.cfg.first_block];
    //2. inline in the block: we assume the function cfg is already flatened.
    inline_in_block(
        ssa_func.cfg.first_block,
        block,
        &mut inline_map,
        func_id,
        call_id,
        irgen,
    );
    //TODO process all the blocks of the function.
}

//inline the given block of the function body into the target_block
pub fn inline_in_block(
    block_id: BlockId,
    target_block_id: BlockId,
    inline_map: &mut HashMap<NodeId, NodeId>,
    func_id: noirc_frontend::node_interner::FuncId, //func_cfg: &IRGenerator,
    call_id: NodeId,
    irgen: &mut IRGenerator,
) {
    let mut new_instructions: Vec<NodeId> = Vec::new();
    // let mut return_values: Vec<(NodeId,NodeId)> = Vec::new();
    let block_func_instructions = &irgen.get_function(func_id).unwrap().cfg[block_id]
        .instructions
        .clone();
    for &i_id in block_func_instructions {
        //    let mut ftbc = false;
        let mut cloned_ins = None;
        if let Some(ins) = irgen
            .get_function(func_id)
            .unwrap()
            .cfg
            .try_get_instruction(i_id)
        {
            cloned_ins = Some(ins.clone());
        }
        if let Some(clone) = cloned_ins {
            let new_left =
                function::SSAFunction::get_mapped_value(func_id, clone.lhs, irgen, inline_map);
            let new_right =
                function::SSAFunction::get_mapped_value(func_id, clone.rhs, irgen, inline_map);
            let new_arg = function::SSAFunction::get_mapped_value(
                func_id,
                clone.ins_arguments[0],
                irgen,
                inline_map,
            );
            match clone.operator {
                Operation::Nop => (),
                //Return instruction:
                Operation::Ret => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    if let Some(ret_id) =
                        irgen[target_block_id].get_result_instruction(call_id, irgen)
                    {
                        //we support only one result for now, should use 'ins.lhs.get_value()'
                        if let node::NodeObj::Instr(i) = &mut irgen[ret_id] {
                            i.is_deleted = true;
                            i.rhs = new_left; //Then we need to ensure there is a CSE.
                        }
                    } else {
                        //we use the call instruction instead
                        //we could use the ins_arguments to get the results here, and since we have the input arguments (in the ssafunction) we know how may there are.
                        //for now the call instruction is replaced by the (one) result
                        let call_ins = irgen.get_mut_instruction(call_id);
                        call_ins.is_deleted = true;
                        call_ins.rhs = new_arg;
                    }
                }
                Operation::Call(_) => todo!("function calling function is not yet supported"), //We mainly need to map the arguments and then decide how to recurse the function call.
                _ => {
                    let mut new_ins = node::Instruction::new(
                        clone.operator,
                        new_left,
                        new_right,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    optim::simplify(irgen, &mut new_ins);
                    let result_id;
                    let mut to_delete = false;
                    if new_ins.is_deleted {
                        result_id = new_ins.rhs;
                        if new_ins.rhs == new_ins.id {
                            to_delete = true;
                        }
                    } else {
                        result_id = irgen.add_instruction(new_ins);
                        new_instructions.push(result_id)
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
    let mut pos = irgen[target_block_id]
        .instructions
        .iter()
        .position(|x| *x == call_id)
        .unwrap();
    for &new_id in &new_instructions {
        irgen[target_block_id].instructions.insert(pos, new_id);
        pos += 1;
    }
}
