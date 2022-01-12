use super::{
    code_gen::ParsingContext,
    node::{self, Node, NodeEval},
};
use acvm::FieldElement;
use arena::Index;
use std::collections::HashMap;

//Unroll a for loop: exit <- join <--> body
//join block is given in argumemt, it will evaluate the join condition, starting from 'start' until it reaches 'end'
//and write the unrolled instructions into the join block, then delete the body block at the end.
pub fn unroll_block(join_id: Index, eval: &mut ParsingContext) {
    let join = eval.get_block(join_id).unwrap();
    let join_instructions = join.instructions.clone();
    let mut from = join.predecessor.first().unwrap().clone(); //todo predecessor.first or .last?
    if join.kind == node::BlockType::ForJoin {
        let body = eval.get_block(join.right.unwrap()).unwrap();
        let body_id = join.right.unwrap();
        let mut body_instructions = body.instructions.clone();
        body_instructions.pop(); //we remove the last jump instruction
        body_instructions.remove(0); //we remove the first dummy instruction

        let mut eval_map: HashMap<Index, node::NodeEval> = HashMap::new();
        let mut new_instructions_id: Vec<arena::Index> = Vec::new();
        new_instructions_id.push(*join_instructions.first().unwrap()); //TODO we should assert it is a nop instruction
        while {
            //evaluate the join  block:
            evaluate_phi(&join_instructions, from, &mut eval_map, eval);
            evaluate_conditional_jump(*join_instructions.last().unwrap(), &mut eval_map, eval)
        } {
            //Process body
            for i_id in &body_instructions {
                let ins = eval.get_object(*i_id).unwrap();

                match ins {
                    node::NodeObj::Instr(i) => {
                        let new_left = get_current_value(i.lhs, &eval_map).to_index().unwrap();
                        let new_right = get_current_value(i.rhs, &eval_map).to_index().unwrap();
                        let new_ins = node::Instruction::new(
                            i.operator,
                            new_left,
                            new_right,
                            i.res_type,
                            Some(join_id),
                        );
                        if i.operator == node::Operation::ass {
                            unreachable!("unsupported instruction type when unrolling: assign");
                            //To support assignments, we should create a new variable and updates the eval_map with it
                            //however assignments should have already been removed by copy propagation.
                        } else {
                            //we do some constant folding here because the iterator is a constant, however this
                            // should be built-in whenever we generate an instruction...TODO
                            let lhs = get_current_value(i.lhs, &eval_map);
                            let lhr = get_current_value(i.rhs, &eval_map);
                            let result = i.evaluate(&to_const(eval, lhs), &to_const(eval, lhr));
                            let result_id;
                            if let Some(_) = result.to_const_value() {
                                result_id = to_index(eval, result);
                            } else {
                                result_id = eval.nodes.insert(node::NodeObj::Instr(new_ins));
                                new_instructions_id.push(result_id);
                            }
                            eval_map.insert(*i_id, node::NodeEval::Idx(result_id));
                        }
                    }
                    _ => todo!(),
                }
            }
            from = body_id;
        }
        //TODO add the jmp to the exit block
        for ins in &new_instructions_id {
            let ins_obj = eval.get_as_mut_instruction(*ins);
            ins_obj.unwrap().idx = *ins;
        }

        let mut join_mut = eval.get_block_mut(join_id).unwrap();
        join_mut.right = None;
        join_mut.kind = node::BlockType::Normal;
        join_mut.instructions.clear();
        for ins in &new_instructions_id {
            join_mut.instructions.push(*ins);
        }
        //remove body block
        if let Some(index) = join_mut
            .dominated
            .iter()
            .position(|value| *value == body_id)
        {
            join_mut.dominated.swap_remove(index);
        }
        eval.blocks.remove(body_id);
    }
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &Vec<arena::Index>,
    from: Index,
    to: &mut HashMap<Index, node::NodeEval>,
    eval: &mut ParsingContext,
) {
    for i in instructions {
        let mut to_process: Vec<(Index, node::NodeEval)> = Vec::new();
        if let Some(ins) = eval.get_as_instruction(*i) {
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
            to.insert(obj.0, node::NodeEval::Idx(to_index(eval, obj.1)));
        }
    }
}

//returns the NodeObj index of a NodeEval object
//if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
fn to_index(eval: &mut ParsingContext, obj: node::NodeEval) -> Index {
    match obj {
        node::NodeEval::Const(c, t) => eval.get_const(c, t),
        node::NodeEval::Idx(i) => i,
    }
}

// If NodeEval refers to a constant NodeObj, we return a constant NodeEval
fn to_const(eval: &ParsingContext, obj: node::NodeEval) -> node::NodeEval {
    match obj {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(i) => {
            let node_obj = eval.get_object(i).unwrap();
            match node_obj {
                node::NodeObj::Const(c) => node::NodeEval::Const(
                    FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()),
                    c.get_type(),
                ),
                _ => obj,
            }
        }
    }
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
    eval: &ParsingContext,
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
                        dbg!(&i.phi_arguments);
                        return node::NodeEval::Idx(i.idx);
                    }

                    let lhs = get_current_value(i.lhs, value_array);
                    let lhr = get_current_value(i.rhs, value_array);
                    let result = i.evaluate(&lhs, &lhr);
                    match result {
                        node::NodeEval::Idx(idx) => {
                            if eval.get_object(idx).is_none() {
                                return node::NodeEval::Idx(obj_id);
                            }
                        }
                        _ => (),
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
    eval: &ParsingContext,
) -> node::NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(obj_id) => {
            if eval.get_object(obj_id).is_none() {
                dbg!(obj_id);
                dbg!(obj);
                return obj;
            }
            let value = eval.get_object(obj_id).unwrap();
            match value {
                node::NodeObj::Instr(i) => {
                    if i.operator == node::Operation::phi {
                        dbg!(i.idx);
                        dbg!(&i.phi_arguments);
                        return node::NodeEval::Idx(i.idx);
                    }
                    //n.b phi are handled before, else we should know which block we come from
                    let lhs =
                        evaluate_object(get_current_value(i.lhs, value_array), value_array, eval);
                    let lhr =
                        evaluate_object(get_current_value(i.rhs, value_array), value_array, eval);
                    let result = i.evaluate(&lhs, &lhr);
                    match result {
                        node::NodeEval::Idx(idx) => {
                            if eval.get_object(idx).is_none() {
                                return node::NodeEval::Idx(obj_id);
                            }
                        }
                        _ => (),
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

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: Index,
    value_array: &mut HashMap<arena::Index, node::NodeEval>,
    eval: &ParsingContext,
) -> bool {
    let jump_ins = eval.get_as_instruction(jump).unwrap();
    let lhs = get_current_value(jump_ins.lhs, value_array);
    let cond = evaluate_object(lhs, value_array, eval);
    let cond_const = cond.to_const_value();
    if cond_const.is_some() {
        let result = !cond_const.unwrap().is_zero();
        match jump_ins.operator {
            node::Operation::jeq => return result,
            node::Operation::jne => return !result,
            node::Operation::jmp => return true,
            _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
        }
    }
    unreachable!("Condition should be constant");
    true
}

pub fn unroll(eval: &mut ParsingContext) {
    unroll_tree(eval, eval.first_block);
    //which order should we follow?
}

pub fn unroll_tree(eval: &mut ParsingContext, b_idx: arena::Index) {
    unroll_block(b_idx, eval);
    let block = eval.get_block(b_idx).unwrap();
    let bd = block.dominated.clone();
    for b in bd {
        unroll_block(b, eval);
    }
}
