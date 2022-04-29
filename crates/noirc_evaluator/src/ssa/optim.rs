use super::{
    block::BlockId,
    context::SsaContext,
    mem::Memory,
    node::{self, BinaryOp, ConstrainOp, Instruction, Node, NodeEval, NodeId, Operation},
};
use std::collections::{HashMap, VecDeque};

// Performs constant folding, arithmetic simplifications and move to standard form
// Returns a new NodeId to replace with the current if the current is deleted.
pub fn simplify(ctx: &mut SsaContext, ins: &mut node::Instruction) -> Option<NodeId> {
    //1. constant folding
    let idx = match ins.evaluate(ctx) {
        NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    };

    if idx != ins.id {
        ins.is_deleted = true;
        if idx == NodeId::dummy() {
            ins.operator = node::Operation::Nop;
        }
        return Some(idx);
    }

    //2. standard form
    ins.standard_form();
    match ins.operator {
        Operation::Cast(value_id) => {
            if let Some(value) = ctx.try_get_node(value_id) {
                if value.get_type() == ins.res_type {
                    ins.is_deleted = true;
                    return Some(NodeId::dummy());
                }
            }
        }
        Operation::Binary(node::Binary {
            operator: BinaryOp::Constrain(op),
            lhs,
            rhs,
        }) => {
            match (op, Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
                (ConstrainOp::Eq, Some(lhs), Some(rhs)) if lhs == rhs => {
                    ins.is_deleted = true;
                    ins.operator = Operation::Nop;
                }
                (ConstrainOp::Neq, Some(lhs), Some(rhs)) => {
                    // TODO: Why are we asserting here? This seems like a valid case
                    assert_ne!(lhs, rhs);
                }
                _ => (),
            }
        }
        _ => (),
    }

    //3. left-overs (it requires &mut ctx)
    if let Operation::Binary(node::Binary {
        operator: BinaryOp::Div,
        lhs,
        rhs,
    }) = ins.operator
    {
        if let NodeEval::Const(r_const, r_type) = NodeEval::from_id(ctx, rhs) {
            let rhs = ctx.get_or_create_const(r_const.try_inverse().unwrap(), r_type);
            ins.operator = Operation::Binary(node::Binary {
                operator: BinaryOp::Mul,
                lhs,
                rhs,
            });
        }
    }

    None
}

////////////////////CSE////////////////////////////////////////

pub fn find_similar_instruction(
    igen: &SsaContext,
    operation: &Operation,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if &ins.operator == operation {
                return Some(*iter);
            }
        }
    }
    None
}

pub fn find_similar_instruction_with_multiple_arguments(
    igen: &SsaContext,
    lhs: NodeId,
    rhs: NodeId,
    ins_args: &[NodeId],
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if ins.lhs == lhs && ins.rhs == rhs && ins.ins_arguments == ins_args {
                return Some(*iter);
            }
        }
    }
    None
}

pub fn find_similar_cast(
    igen: &SsaContext,
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
    Replace {
        original: NodeId,
        replacement: NodeId,
    },
    Remove(NodeId),
    Keep,
}

pub fn find_similar_mem_instruction(
    ctx: &SsaContext,
    op: &Operation,
    ins_id: NodeId,
    lhs: NodeId,
    rhs: NodeId,
    anchor: &HashMap<node::Operation, VecDeque<NodeId>>,
) -> CseAction {
    match op {
        Operation::Load { array, index } => {
            for iter in anchor[&op].iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*iter) {
                    match &ins_iter.operator {
                        Operation::Load {
                            array,
                            index: index2,
                        } => {
                            if index == index2 {
                                return CseAction::Replace {
                                    original: ins_id,
                                    replacement: *iter,
                                };
                            }
                        }
                        Operation::Store {
                            array: array2,
                            index: index2,
                            value,
                        } => {
                            assert_eq!(array, array2);

                            if index == index2 {
                                return CseAction::Replace {
                                    original: ins_id,
                                    replacement: *value,
                                };
                            } else {
                                //TODO: If we know that ins.lhs value cannot be equal to ins_iter.rhs, we could continue instead
                                return CseAction::Keep;
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        Operation::Store {
            array,
            index,
            value,
        } => {
            let prev_ins = &anchor[&Operation::Load {
                array: *array,
                index: *index,
            }];
            for node_id in prev_ins.iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*node_id) {
                    match ins_iter.operator {
                        Operation::Load {
                            array: array2,
                            index: index2,
                        } => {
                            //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                            return CseAction::Keep;
                        }
                        Operation::Store {
                            array: array2,
                            index: index2,
                            value: value2,
                        } => {
                            if *index == index2 {
                                return CseAction::Remove(*node_id);
                            } else {
                                //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                                return CseAction::Keep;
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        _ => unreachable!("invalid non memory operator"),
    }

    CseAction::Keep
}

pub fn propagate(ctx: &SsaContext, id: NodeId) -> NodeId {
    let mut result = id;
    if let Some(obj) = ctx.try_get_instruction(id) {
        if obj.operator == Operation::Assign || obj.is_deleted {
            result = obj.rhs;
        }
    }
    result
}

//common subexpression elimination, starting from the root
pub fn cse(igen: &mut SsaContext) -> Option<NodeId> {
    let mut anchor = HashMap::new();
    cse_tree(igen, igen.first_block, &mut anchor)
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
pub fn cse_tree(
    igen: &mut SsaContext,
    block_id: BlockId,
    anchor: &mut HashMap<Operation, VecDeque<NodeId>>,
) -> Option<NodeId> {
    let mut instructions = Vec::new();
    let mut res = block_cse(igen, block_id, anchor, &mut instructions);
    for b in igen[block_id].dominated.clone() {
        let sub_res = cse_tree(igen, b, &mut anchor.clone());
        if sub_res.is_some() {
            res = sub_res;
        }
    }
    res
}

pub fn anchor_push(op: Operation, anchor: &mut HashMap<Operation, VecDeque<NodeId>>) {
    match op {
        Operation::Store(x) => anchor
            .entry(Operation::Load(x))
            .or_insert_with(VecDeque::new),
        _ => anchor.entry(op).or_insert_with(VecDeque::new),
    };
}

//Performs common subexpression elimination and copy propagation on a block
pub fn block_cse(
    ctx: &mut SsaContext,
    block_id: BlockId,
    anchor: &mut HashMap<Operation, VecDeque<NodeId>>,
    instructions: &mut Vec<NodeId>,
) -> Option<NodeId> {
    let mut new_list = Vec::new();
    let bb = &ctx[block_id];

    if instructions.is_empty() {
        instructions.append(&mut bb.instructions.clone());
    }

    for iter in instructions {
        if let Some(ins) = ctx.try_get_instruction(*iter) {
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
            if ins.operator.is_binary() {
                //binary operation:
                i_lhs = propagate(ctx, ins.lhs);
                i_rhs = propagate(ctx, ins.rhs);
                if let Some(j) =
                    find_similar_instruction(ctx, &ins.operator, &anchor[&ins.operator])
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
                        i_lhs = propagate(ctx, ins.lhs);
                        i_rhs = propagate(ctx, ins.rhs);
                        let (cse_id, cse_action) = find_similar_mem_instruction(
                            ctx,
                            &ins.operator,
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
                    node::Operation::Assign => {
                        //assignement
                        i_rhs = propagate(ctx, ins.rhs);
                        to_delete = true;
                    }
                    node::Operation::Phi => {
                        // propagate phi arguments
                        for a in &ins.phi_arguments {
                            phi_args.push((propagate(ctx, a.0), a.1));
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
                        i_lhs = propagate(ctx, ins.lhs);
                        i_rhs = i_lhs;
                        //Similar cast must have same type
                        if let Some(j) =
                            find_similar_cast(ctx, i_lhs, ins.res_type, &anchor[&ins.operator])
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
                            let new_a = propagate(ctx, *a);
                            if !to_update && new_a != *a {
                                to_update = true;
                            }
                            ins_args.push(new_a);
                        }
                        new_list.push(*iter);
                    }
                    node::Operation::Intrinsic(_) => {
                        //n.b this could be the default behovoir for binary operations
                        for a in &ins.ins_arguments {
                            let new_a = propagate(ctx, *a);
                            if !to_update && new_a != *a {
                                to_update = true;
                            }
                            ins_args.push(new_a);
                        }
                        i_lhs = propagate(ctx, ins.lhs);
                        i_rhs = propagate(ctx, ins.rhs);
                        if let Some(j) = find_similar_instruction_with_multiple_arguments(
                            ctx,
                            i_lhs,
                            i_rhs,
                            &ins_args,
                            &anchor[&ins.operator],
                        ) {
                            to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                            i_rhs = j;
                        } else {
                            new_list.push(*iter);
                            anchor.get_mut(&ins.operator).unwrap().push_front(*iter);
                        }
                    }
                    _ => {
                        //TODO: checks we do not need to propagate res arguments
                        new_list.push(*iter);
                    }
                }
            }

            if to_update_phi {
                let update = ctx.get_mut_instruction(*iter);
                update.phi_arguments = phi_args;
            } else if to_delete || ins.lhs != i_lhs || ins.rhs != i_rhs || to_update {
                //update i:
                let ii_l = ins.lhs;
                let ii_r = ins.rhs;
                let update = ctx.get_mut_instruction(*iter);
                update.lhs = i_lhs;
                update.rhs = i_rhs;
                update.is_deleted = to_delete;
                if to_update {
                    update.ins_arguments = ins_args;
                }
                //update instruction name - for debug/pretty print purposes only /////////////////////
                if let Some(Instruction {
                    operator: Operation::Assign,
                    lhs,
                    ..
                }) = ctx.try_get_instruction(ii_l)
                {
                    if let Ok(lv) = ctx.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = ctx.try_get_mut_instruction(i_lhs) {
                            if p_ins.res_name.is_empty() {
                                p_ins.res_name = i_name;
                            }
                        }
                    }
                }
                if let Some(Instruction {
                    operator: Operation::Assign,
                    lhs,
                    ..
                }) = ctx.try_get_instruction(ii_r)
                {
                    if let Ok(lv) = ctx.get_variable(*lhs) {
                        let i_name = lv.name.clone();
                        if let Some(p_ins) = ctx.try_get_mut_instruction(i_rhs) {
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
    let mut last = None;
    for i in new_list.iter().rev() {
        if is_some(ctx, *i) {
            last = Some(*i);
            break;
        }
    }
    ctx[block_id].instructions = new_list;
    last
}

pub fn is_some(ctx: &SsaContext, id: NodeId) -> bool {
    if id == NodeId::dummy() {
        return false;
    }
    if let Some(ins) = ctx.try_get_instruction(id) {
        if ins.operator != node::Operation::Nop {
            return true;
        }
    } else if ctx.try_get_node(id).is_some() {
        return true;
    }
    false
}
