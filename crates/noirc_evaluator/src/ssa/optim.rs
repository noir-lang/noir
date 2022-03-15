use super::{
    block::BlockId,
    code_gen::IRGenerator,
    mem,
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
pub fn simplify(irgen: &mut IRGenerator, ins: &mut node::Instruction) {
    //1. constant folding
    let l_eval = to_const(irgen, NodeEval::VarOrInstruction(ins.lhs));
    let r_eval = to_const(irgen, NodeEval::VarOrInstruction(ins.rhs));
    let idx = match ins.evaluate(&l_eval, &r_eval) {
        NodeEval::Const(c, t) => irgen.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    };
    if idx != ins.id {
        ins.is_deleted = true;
        ins.rhs = idx;
        if idx == NodeId::dummy() {
            ins.operator = node::Operation::Nop;
        }
        return;
    }

    //2. standard form
    ins.standard_form();
    match ins.operator {
        node::Operation::Cast => {
            if let Some(lhs_obj) = irgen.try_get_node(ins.lhs) {
                if lhs_obj.get_type() == ins.res_type {
                    ins.is_deleted = true;
                    ins.rhs = ins.lhs;
                    return;
                }
            }
        }
        // node::Operation::Gte => {
        //     //a>=b <=> Not(a<b)
        //     let inv = eval.new_instruction(ins.lhs, ins.rhs, node::Operation::Lt, ins.res_type);
        //     ins.lhs = eval.get_const(FieldElement::one(), ins.res_type);
        //     ins.rhs = inv;
        //     ins.operator = node::Operation::Sub; //n.b. no need to underflow here
        //TODO: inv must be inserted before ins.
        // }
        node::Operation::Constrain(op) => match op {
            node::ConstrainOp::Eq => {
                if let (Some(a), Some(b)) = (
                    mem::Memory::deref(irgen, ins.lhs),
                    mem::Memory::deref(irgen, ins.rhs),
                ) {
                    if a == b {
                        ins.is_deleted = true;
                        ins.operator = node::Operation::Nop;
                    }
                }
            }
            node::ConstrainOp::Neq => {
                if let (Some(a), Some(b)) = (
                    mem::Memory::deref(irgen, ins.lhs),
                    mem::Memory::deref(irgen, ins.rhs),
                ) {
                    assert!(a != b);
                }
            }
        },
        _ => (),
    }

    //3. left-overs (it requires &mut eval)

    if let NodeEval::Const(r_const, r_type) = r_eval {
        match ins.operator {
            node::Operation::Udiv => {
                //TODO handle other bitsize, not only u32!!
                ins.rhs = irgen.get_or_create_const(
                    FieldElement::from((1_u32 / (r_const.to_u128() as u32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::Mul
            }
            node::Operation::Sdiv => {
                //TODO handle other bitsize, not only i32!!
                ins.rhs = irgen.get_or_create_const(
                    FieldElement::from((1_i32 / (r_const.to_u128() as i32)) as i128),
                    r_type,
                );
                ins.operator = node::Operation::Mul
            }
            node::Operation::Div => {
                ins.rhs = irgen.get_or_create_const(r_const.inverse(), r_type);
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

pub fn find_similar_cast(
    igen: &IRGenerator,
    lhs: NodeId,
    res_type: node::ObjectType,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if ins.lhs == lhs && ins.res_type == res_type {
                return Some(*iter);
            }
        }
    }
    None
}
pub enum CseAction {
    Replace, //replace the instruction
    Remove,  //remove the instruction
    Keep,    //keep the instruction
}

//Returns an id and an action:
//- replace => the instruction should be replaced by the returned id
//- remove  => the instruction corresponding to the returned id should be removed
//- keep    => keep the instruction
pub fn find_similar_mem_instruction(
    eval: &IRGenerator,
    op: node::Operation,
    ins_id: NodeId,
    lhs: NodeId,
    rhs: NodeId,
    anchor: &HashMap<node::Operation, VecDeque<NodeId>>,
) -> (NodeId, CseAction) {
    match op {
        node::Operation::Load(_) => {
            for iter in anchor[&op].iter().rev() {
                if let Some(ins_iter) = eval.try_get_instruction(*iter) {
                    match ins_iter.operator {
                        node::Operation::Load(_) => {
                            if ins_iter.lhs == lhs {
                                return (*iter, CseAction::Replace);
                            }
                        }
                        node::Operation::Store(_) => {
                            if ins_iter.rhs == lhs {
                                return (ins_iter.lhs, CseAction::Replace);
                            } else {
                                //TODO: If we know that ins.lhs value cannot be equal to ins_iter.rhs, we could continue instead
                                return (ins_id, CseAction::Keep);
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        node::Operation::Store(x) => {
            let prev_ins = &anchor[&node::Operation::Load(x)];
            for iter in prev_ins.iter().rev() {
                if let Some(ins_iter) = eval.try_get_instruction(*iter) {
                    match ins_iter.operator {
                        node::Operation::Load(_) => {
                            //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                            return (ins_id, CseAction::Keep);
                        }
                        node::Operation::Store(_) => {
                            if ins_iter.rhs == rhs {
                                return (*iter, CseAction::Remove);
                            } else {
                                //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                                return (ins_id, CseAction::Keep);
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        _ => unreachable!("invalid non memory operator"),
    }
    (ins_id, CseAction::Keep)
}
pub fn propagate(eval: &IRGenerator, id: NodeId) -> NodeId {
    let mut result = id;
    if let Some(obj) = eval.try_get_instruction(id) {
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

pub fn anchor_push(op: node::Operation, anchor: &mut HashMap<node::Operation, VecDeque<NodeId>>) {
    match op {
        node::Operation::Store(x) => anchor
            .entry(node::Operation::Load(x))
            .or_insert_with(VecDeque::new),
        _ => anchor.entry(op).or_insert_with(VecDeque::new),
    };
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
            let mut ins_args = Vec::new();
            let mut to_update_phi = false;
            let mut to_update = false;

            if ins.is_deleted {
                continue;
            }
            anchor_push(ins.operator, anchor);
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
                }
            } else {
                match ins.operator {
                    node::Operation::Load(_) | node::Operation::Store(_) => {
                        i_lhs = propagate(igen, ins.lhs);
                        i_rhs = propagate(igen, ins.rhs);
                        let (cse_id, cse_action) = find_similar_mem_instruction(
                            igen,
                            ins.operator,
                            ins.id,
                            i_lhs,
                            i_rhs,
                            anchor,
                        );
                        match cse_action {
                            CseAction::Keep => new_list.push(*iter),
                            CseAction::Replace => {
                                to_delete = true;
                                i_rhs = cse_id;
                            }
                            CseAction::Remove => {
                                new_list.push(*iter);
                                // TODO if not found, it should be removed from other blocks; we could keep a list of instructions to remove
                                if let Some(pos) = new_list.iter().position(|x| *x == cse_id) {
                                    new_list.remove(pos);
                                }
                            }
                        }
                    }
                    node::Operation::Ass => {
                        //assignement
                        i_rhs = propagate(igen, ins.rhs);
                        to_delete = true;
                    }
                    node::Operation::Phi => {
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
                    }
                    node::Operation::Cast => {
                        //Propagate cast argument
                        i_lhs = propagate(igen, ins.lhs);
                        i_rhs = i_lhs;
                        //Similar cast must have same type
                        if let Some(j) =
                            find_similar_cast(igen, i_lhs, ins.res_type, &anchor[&ins.operator])
                        {
                            to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                            i_rhs = j;
                        } else {
                            new_list.push(*iter);
                            anchor.get_mut(&ins.operator).unwrap().push_front(*iter);
                        }
                    }
                    node::Operation::Call(_) | node::Operation::Ret => {
                        //No CSE for function calls because of possible side effect - TODO checks if a function has side effect when parsed and do cse for these.
                        //Propagate arguments:
                        for a in &ins.ins_arguments {
                            let new_a = propagate(igen, *a);
                            if !to_update && new_a != *a {
                                to_update = true;
                            }
                            ins_args.push(new_a);
                        }
                        new_list.push(*iter);
                    }
                    _ => {
                        //TODO: checks we do not need to propagate res arguments
                        new_list.push(*iter);
                    }
                }
            }

            if to_update_phi {
                let update = igen.get_mut_instruction(*iter);
                update.phi_arguments = phi_args;
            } else if to_delete || ins.lhs != i_lhs || ins.rhs != i_rhs || to_update {
                //update i:
                let ii_l = ins.lhs;
                let ii_r = ins.rhs;
                let update = igen.get_mut_instruction(*iter);
                update.lhs = i_lhs;
                update.rhs = i_rhs;
                update.is_deleted = to_delete;
                if to_update {
                    update.ins_arguments = ins_args;
                }
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
