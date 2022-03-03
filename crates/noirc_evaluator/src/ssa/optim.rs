use super::{
    block::BlockId,
    code_gen::IRGenerator,
    node::{self, Instruction, Node, NodeEval, NodeId, NodeObj, Operation},
};
use acvm::FieldElement;
use std::collections::{HashMap, VecDeque};

//returns the NodeObj index of a NodeEval object
//if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
pub fn to_index(eval: &mut IRGenerator, obj: NodeEval) -> NodeId {
    match obj {
        NodeEval::Const(c, t) => eval.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    }
}

// If NodeEval refers to a constant NodeObj, we return a constant NodeEval
pub fn to_const(eval: &IRGenerator, obj: NodeEval) -> NodeEval {
    match obj {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(i) => {
            if let Some(NodeObj::Const(c)) = eval.try_get_node(i) {
                return NodeEval::Const(
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
    let l_eval = to_const(eval, NodeEval::VarOrInstruction(ins.lhs));
    let r_eval = to_const(eval, NodeEval::VarOrInstruction(ins.rhs));
    let idx = match ins.evaluate(&l_eval, &r_eval) {
        NodeEval::Const(c, t) => eval.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    };
    if idx != ins.id {
        ins.is_deleted = true;
        ins.rhs = idx;
        return;
    }

    //2. standard form
    ins.standard_form();
    if ins.operator == node::Operation::Cast {
        if let Some(lhs_obj) = eval.try_get_node(ins.lhs) {
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
            node::Operation::Udiv => {
                //TODO handle other bitsize, not only u32!!
                ins.rhs = eval.get_or_create_const(
                    FieldElement::from((1_u32 / (r_const.to_u128() as u32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::Mul
            }
            node::Operation::Sdiv => {
                //TODO handle other bitsize, not only i32!!
                ins.rhs = eval.get_or_create_const(
                    FieldElement::from((1_i32 / (r_const.to_u128() as i32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::Mul
            }
            node::Operation::Div => {
                ins.rhs = eval.get_or_create_const(r_const.inverse(), r_type);
                ins.operator = node::Operation::Mul
            }
            node::Operation::Xor => {
                if !r_const.is_zero() {
                    ins.operator = node::Operation::Not;
                    return;
                }
            }
            _ => (),
        }
    }
    if let NodeEval::Const(l_const, _) = l_eval {
        if !l_const.is_zero() && ins.operator == node::Operation::Xor {
            ins.operator = node::Operation::Not;
            ins.lhs = ins.rhs;
        }
    }
}

////////////////////CSE////////////////////////////////////////

pub fn find_similar_instruction(
    igen: &IRGenerator,
    lhs: NodeId,
    rhs: NodeId,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if ins.lhs == lhs && ins.rhs == rhs {
                return Some(*iter);
            }
        }
    }
    None
}

pub fn propagate(igen: &IRGenerator, id: NodeId) -> NodeId {
    let mut result = id;
    if let Some(obj) = igen.try_get_instruction(id) {
        if obj.operator == node::Operation::Ass || obj.is_deleted {
            result = obj.rhs;
        }
    }
    result
}

//common subexpression elimination, starting from the root
pub fn cse(igen: &mut IRGenerator) {
    let mut anchor = HashMap::new();
    cse_tree(igen, igen.first_block, &mut anchor);
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
pub fn cse_tree(
    igen: &mut IRGenerator,
    block_id: BlockId,
    anchor: &mut HashMap<Operation, VecDeque<NodeId>>,
) {
    let mut instructions = Vec::new();
    block_cse(igen, block_id, anchor, &mut instructions);
    for b in igen[block_id].dominated.clone() {
        cse_tree(igen, b, &mut anchor.clone());
    }
}

//Performs common subexpression elimination and copy propagation on a block
pub fn block_cse(
    igen: &mut IRGenerator,
    block_id: BlockId,
    anchor: &mut HashMap<Operation, VecDeque<NodeId>>,
    instructions: &mut Vec<NodeId>,
) {
    let mut new_list = Vec::new();
    let bb = &igen[block_id];

    if instructions.is_empty() {
        instructions.append(&mut bb.instructions.clone());
    }

    for iter in instructions {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            let mut to_delete = false;
            let mut i_lhs = ins.lhs;
            let mut i_rhs = ins.rhs;
            let mut phi_args = Vec::new();
            let mut to_update_phi = false;
            anchor.entry(ins.operator).or_insert_with(VecDeque::new);
            if ins.is_deleted {
                continue;
            }
            if node::is_binary(ins.operator) {
                //binary operation:
                i_lhs = propagate(igen, ins.lhs);
                i_rhs = propagate(igen, ins.rhs);
                if let Some(j) =
                    find_similar_instruction(igen, i_lhs, i_rhs, &anchor[&ins.operator])
                {
                    to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                    i_rhs = j;
                } else {
                    new_list.push(*iter);
                    anchor.get_mut(&ins.operator).unwrap().push_front(*iter);
                    //TODO - Take into account store and load for arrays
                }
            } else if ins.operator == node::Operation::Ass {
                //assignement
                i_rhs = propagate(igen, ins.rhs);
                to_delete = true;
            } else if ins.operator == node::Operation::Cast {
                i_lhs = propagate(igen, ins.lhs);
                i_rhs = propagate(igen, ins.rhs);
            } else if ins.operator == node::Operation::Phi {
                // propagate phi arguments
                for a in &ins.phi_arguments {
                    phi_args.push((propagate(igen, a.0), a.1));
                    if phi_args.last().unwrap().0 != a.0 {
                        to_update_phi = true;
                    }
                }
                if let Some(first) = node::Instruction::simplify_phi(ins.id, &phi_args) {
                    if first == ins.id {
                        new_list.push(*iter);
                    } else {
                        to_delete = true;
                        i_rhs = first;
                        to_update_phi = false;
                    }
                } else {
                    to_delete = true;
                }
            } else {
                new_list.push(*iter);
            }
            if to_update_phi {
                let update = igen.get_mut_instruction(*iter);
                update.phi_arguments = phi_args;
            } else if to_delete || ins.lhs != i_lhs || ins.rhs != i_rhs {
                //update i:
                let ii_l = ins.lhs;
                let ii_r = ins.rhs;
                let update = igen.get_mut_instruction(*iter);
                update.lhs = i_lhs;
                update.rhs = i_rhs;
                update.is_deleted = to_delete;
                //update instruction name - for debug/pretty print purposes only /////////////////////
                if let Some(Instruction {
                    operator: Operation::Ass,
                    lhs,
                    ..
                }) = igen.try_get_instruction(ii_l)
                {
                    if let Ok(lv) = igen.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = igen.try_get_mut_instruction(i_lhs) {
                            if p_ins.res_name.is_empty() {
                                p_ins.res_name = i_name;
                            }
                        }
                    }
                }
                if let Some(Instruction {
                    operator: Operation::Ass,
                    lhs,
                    ..
                }) = igen.try_get_instruction(ii_r)
                {
                    if let Ok(lv) = igen.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = igen.try_get_mut_instruction(i_rhs) {
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

    igen[block_id].instructions = new_list;
}
