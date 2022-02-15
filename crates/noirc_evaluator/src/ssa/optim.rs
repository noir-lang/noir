use super::{
    code_gen::IRGenerator,
    node::{self, Node, NodeEval},
};
use acvm::FieldElement;
use arena::Index;
use std::collections::{HashMap, VecDeque};

//returns the NodeObj index of a NodeEval object
//if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
pub fn to_index(eval: &mut IRGenerator, obj: node::NodeEval) -> Index {
    match obj {
        node::NodeEval::Const(c, t) => eval.get_const(c, t),
        node::NodeEval::Idx(i) => i,
    }
}

// If NodeEval refers to a constant NodeObj, we return a constant NodeEval
pub fn to_const(eval: &IRGenerator, obj: node::NodeEval) -> node::NodeEval {
    match obj {
        node::NodeEval::Const(_, _) => obj,
        node::NodeEval::Idx(i) => {
            if let Some(node::NodeObj::Const(c)) = eval.get_object(i) {
                return node::NodeEval::Const(
                    FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()),
                    c.get_type(),
                );
            }
            obj
        }
    }
}

// Performs constant folding, arithmetic simplifications and move to standard form
pub fn simplify(eval: &mut IRGenerator, ins: &mut node::Instruction) {
    //1. constant folding
    let l_eval = to_const(eval, node::NodeEval::Idx(ins.lhs));
    let r_eval = to_const(eval, node::NodeEval::Idx(ins.rhs));
    let idx = match ins.evaluate(&l_eval, &r_eval) {
        NodeEval::Const(c, t) => eval.get_const(c, t),
        NodeEval::Idx(i) => i,
    };
    if idx != ins.idx {
        ins.is_deleted = true;
        ins.rhs = idx;
        return;
    }

    //2. standard form
    ins.standard_form();
    if ins.operator == node::Operation::cast {
        if let Some(lhs_obj) = eval.get_object(ins.lhs) {
            if lhs_obj.get_type() == ins.res_type {
                ins.is_deleted = true;
                ins.rhs = ins.lhs;
                return;
            }
        }
    }

    //3. left-overs (it requires &mut eval)
    if let NodeEval::Const(r_const, r_type) = r_eval {
        match ins.operator {
            node::Operation::udiv => {
                //TODO handle other bitsize, not only u32!!
                ins.rhs = eval.get_const(
                    FieldElement::from((1_u32 / (r_const.to_u128() as u32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::mul
            }
            node::Operation::sdiv => {
                //TODO handle other bitsize, not only i32!!
                ins.rhs = eval.get_const(
                    FieldElement::from((1_i32 / (r_const.to_u128() as i32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::mul
            }
            node::Operation::div => {
                ins.rhs = eval.get_const(r_const.inverse(), r_type);
                ins.operator = node::Operation::mul
            }
            node::Operation::xor => {
                if !r_const.is_zero() {
                    ins.operator = node::Operation::not;
                    return;
                }
            }
            _ => (),
        }
    }
    if let NodeEval::Const(l_const, _) = l_eval {
        if !l_const.is_zero() && ins.operator == node::Operation::xor {
            ins.operator = node::Operation::not;
            ins.lhs = ins.rhs;
        }
    }
}

////////////////////CSE////////////////////////////////////////

pub fn find_similar_instruction(
    eval: &IRGenerator,
    lhs: arena::Index,
    rhs: arena::Index,
    prev_ins: &VecDeque<arena::Index>,
) -> Option<arena::Index> {
    for iter in prev_ins {
        if let Some(ins) = eval.try_get_instruction(*iter) {
            if ins.lhs == lhs && ins.rhs == rhs {
                return Some(*iter);
            }
        }
    }
    None
}

pub fn propagate(eval: &IRGenerator, idx: arena::Index) -> arena::Index {
    let mut result = idx;
    if let Some(obj) = eval.try_get_instruction(idx) {
        if obj.operator == node::Operation::ass || obj.is_deleted {
            result = obj.rhs;
        }
    }
    result
}

//common subexpression elimination, starting from the root
pub fn cse(eval: &mut IRGenerator) {
    let mut anchor: HashMap<node::Operation, VecDeque<arena::Index>> = HashMap::new();
    cse_tree(eval, eval.first_block, &mut anchor);
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
pub fn cse_tree(
    eval: &mut IRGenerator,
    b_idx: arena::Index,
    anchor: &mut HashMap<node::Operation, VecDeque<arena::Index>>,
) {
    let mut i_list: Vec<arena::Index> = Vec::new();
    block_cse(eval, b_idx, anchor, &mut i_list);
    let block = eval.get_block(b_idx);
    let bd = block.dominated.clone();
    for b in bd {
        cse_tree(eval, b, &mut anchor.clone());
    }
}

//Performs common subexpression elimination and copy propagation on a block
pub fn block_cse(
    eval: &mut IRGenerator,
    b_idx: arena::Index,
    anchor: &mut HashMap<node::Operation, VecDeque<arena::Index>>,
    block_list: &mut Vec<arena::Index>,
) {
    let mut new_list: Vec<arena::Index> = Vec::new();
    let bb = eval.blocks.get(b_idx).unwrap();

    if block_list.is_empty() {
        //RIA...
        for iter in &bb.instructions {
            block_list.push(*iter);
        }
    }

    for iter in block_list {
        if let Some(ins) = eval.try_get_instruction(*iter) {
            let mut to_delete = false;
            let mut i_lhs = ins.lhs;
            let mut i_rhs = ins.rhs;
            let mut phi_args: Vec<(arena::Index, arena::Index)> = Vec::new();
            let mut to_update_phi = false;

            anchor.entry(ins.operator).or_insert_with(VecDeque::new);
            if ins.is_deleted {
                continue;
            }

            if node::is_binary(ins.operator) {
                //binary operation:
                i_lhs = propagate(eval, ins.lhs);
                i_rhs = propagate(eval, ins.rhs);
                if let Some(j) =
                    find_similar_instruction(eval, i_lhs, i_rhs, &anchor[&ins.operator])
                {
                    to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                    i_rhs = j;
                } else {
                    new_list.push(*iter);
                    anchor.get_mut(&ins.operator).unwrap().push_front(*iter);
                }
            } else {
                match ins.operator {
                    node::Operation::load(_) | node::Operation::store(_) => {
                        i_lhs = propagate(eval, ins.lhs);
                        i_rhs = propagate(eval, ins.rhs);
                        new_list.push(*iter);
                        //TODO CSE for load and store:
                        //find_similar_instruction..
                        //anchor list for 'mem_a' operator, i.e. load or store on array a
                        //specific cse rule (store kill loads and loads kill store)
                        //handle control flow before hand by adding dummy stores on merged blocks
                    }
                    node::Operation::ass => {
                        //assignement
                        i_rhs = propagate(eval, ins.rhs);
                        to_delete = true;
                    }
                    node::Operation::phi => {
                        // propagate phi arguments
                        for a in &ins.phi_arguments {
                            phi_args.push((propagate(eval, a.0), a.1));
                            if phi_args.last().unwrap().0 != a.0 {
                                to_update_phi = true;
                            }
                        }
                        if let Some(first) = node::Instruction::simplify_phi(ins.idx, &phi_args) {
                            if first == ins.idx {
                                new_list.push(*iter);
                            } else {
                                to_delete = true;
                                i_rhs = first;
                                to_update_phi = false;
                            }
                        } else {
                            to_delete = true;
                        }
                    }
                    _ => {
                        new_list.push(*iter);
                    }
                }
            }

            if to_update_phi {
                let update = eval.get_mut_instruction(*iter);
                update.phi_arguments = phi_args;
            } else if to_delete || ins.lhs != i_lhs || ins.rhs != i_rhs {
                //update i:
                let ii_l = ins.lhs;
                let ii_r = ins.rhs;
                let update = eval.get_mut_instruction(*iter);
                update.lhs = i_lhs;
                update.rhs = i_rhs;
                update.is_deleted = to_delete;
                //update instruction name - for debug/pretty print purposes only /////////////////////
                if let Some(node::Instruction {
                    operator: node::Operation::ass,
                    lhs,
                    ..
                }) = eval.try_get_instruction(ii_l)
                {
                    if let Ok(lv) = eval.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = eval.try_get_mut_instruction(i_lhs) {
                            if p_ins.res_name.is_empty() {
                                p_ins.res_name = i_name;
                            }
                        }
                    }
                }
                if let Some(node::Instruction {
                    operator: node::Operation::ass,
                    lhs,
                    ..
                }) = eval.try_get_instruction(ii_r)
                {
                    if let Ok(lv) = eval.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = eval.try_get_mut_instruction(i_rhs) {
                            if p_ins.res_name.is_empty() {
                                p_ins.res_name = i_name;
                            }
                        }
                    }
                }
                ////////////////////////////////////////update instruction name for debug purposes////////////////////////////////
            }
        }
    }
    let bb = eval.blocks.get_mut(b_idx).unwrap();
    bb.instructions = new_list;
}
