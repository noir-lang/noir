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

pub fn find_similar_cast(
    igen: &SsaContext,
    operator: &Operation,
    res_type: node::ObjectType,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if &ins.operator == operator && ins.res_type == res_type {
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
    anchor: &HashMap<node::Operation, VecDeque<NodeId>>,
) -> CseAction {
    match op {
        Operation::Load { array, index } => {
            for iter in anchor[&op].iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*iter) {
                    match &ins_iter.operator {
                        Operation::Load {
                            array: array2,
                            index: index2,
                        } => {
                            assert_eq!(array, array2);
                            assert_eq!(index, index2);
                            return CseAction::Replace {
                                original: ins_id,
                                replacement: *iter,
                            };
                        }
                        Operation::Store {
                            array: array2,
                            index: index2,
                            value,
                        } => {
                            assert_eq!(array, array2);
                            assert_eq!(index, index2);
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
            value: _,
        } => {
            let prev_ins = &anchor[&Operation::Load {
                array: *array,
                index: *index,
            }];
            for node_id in prev_ins.iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*node_id) {
                    match ins_iter.operator {
                        Operation::Load {
                            ..
                        } => {
                            //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                            return CseAction::Keep;
                        }
                        Operation::Store {
                            index: index2,
                            array: _,
                            value: _,
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
    let result = id;
    if let Some(obj) = ctx.try_get_instruction(id) {
        if obj.is_deleted || matches!(obj.operator, Operation::Binary(node::Binary { operator: BinaryOp::Assign, .. })) {
            // result = obj.rhs;
            todo!("Refactor deletion")
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
        Operation::Store { array, index, .. } => anchor
            // TODO: review correctness
            .entry(Operation::Load { array, index })
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

    for ins_id in instructions {
        if let Some(ins) = ctx.try_get_instruction(*ins_id) {
            let mut to_delete = false;

            if ins.is_deleted {
                continue;
            }

            anchor_push(ins.operator.clone(), anchor);

            let operator = ins.operator.map_id(|id| propagate(ctx, id));

            if operator.is_binary() {
                //binary operation:
                if let Some(_j) = find_similar_instruction(ctx, &ins.operator, &anchor[&ins.operator]) {
                    // to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                    // i_rhs = j;
                    todo!("Refactor deletion");
                } else if let Operation::Binary(node::Binary { operator: BinaryOp::Assign, .. }) = operator {
                    // i_rhs = propagate(ctx, ins.rhs);
                    // to_delete = true;
                    todo!("Refactor deletion");
                } else {
                    new_list.push(*ins_id);
                    anchor.get_mut(&ins.operator).unwrap().push_front(*ins_id);
                }
            } else {
                match &operator {
                    Operation::Load { .. } | Operation::Store { .. } => {
                        match find_similar_mem_instruction(ctx, &operator, ins.id, anchor) {
                            CseAction::Keep => new_list.push(*ins_id),
                            CseAction::Replace { original: _, replacement: _ } => {
                                // to_delete = true;
                                // i_rhs = replacement;
                                todo!("Refactor deletion")
                            }
                            CseAction::Remove(id_to_remove) => {
                                new_list.push(*ins_id);
                                // TODO if not found, it should be removed from other blocks; we could keep a list of instructions to remove
                                if let Some(id) = new_list.iter().position(|x| *x == id_to_remove) {
                                    new_list.remove(id);
                                }
                            }
                        }
                    }
                    Operation::Phi { block_args, .. } => {
                        // propagate phi arguments
                        if let Some(first) = Instruction::simplify_phi(ins.id, block_args) {
                            if first == ins.id {
                                new_list.push(*ins_id);
                            } else {
                                // to_delete = true;
                                // /i_rhs = first;
                                todo!("Refactor deletion");
                            }
                        } else {
                            to_delete = true;
                        }
                    }
                    Operation::Cast(_) => {
                        //Similar cast must have same type
                        if let Some(_j) =
                            find_similar_cast(ctx, &operator, ins.res_type, &anchor[&operator])
                        {
                            // to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                            // i_rhs = j;
                            todo!("Refactor deletion");
                        } else {
                            new_list.push(*ins_id);
                            anchor.get_mut(&operator).unwrap().push_front(*ins_id);
                        }
                    }
                    Operation::Call(..) | Operation::Return(..) => {
                        //No CSE for function calls because of possible side effect - TODO checks if a function has side effect when parsed and do cse for these.
                        //Propagate arguments:
                        new_list.push(*ins_id);
                    }
                    Operation::Intrinsic(..) => {
                        //n.b this could be the default behavior for binary operations
                        if let Some(_j) = find_similar_instruction(ctx, &operator, &anchor[&operator]) {
                            // to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                            // i_rhs = j;
                            todo!("Refactor deletion")
                        } else {
                            new_list.push(*ins_id);
                            anchor.get_mut(&operator).unwrap().push_front(*ins_id);
                        }
                    }
                    _ => {
                        //TODO: checks we do not need to propagate res arguments
                        new_list.push(*ins_id);
                    }
                }
            }

            let update = ctx.get_mut_instruction(*ins_id);
            update.operator = operator;
            update.is_deleted = to_delete;
        }
    }

    let last = new_list.iter().copied().rev().find(|id| is_some(ctx, *id));
    ctx[block_id].instructions = new_list;
    last
}

pub fn is_some(ctx: &SsaContext, id: NodeId) -> bool {
    if id == NodeId::dummy() {
        return false;
    }
    if let Some(ins) = ctx.try_get_instruction(id) {
        if ins.operator != Operation::Nop {
            return true;
        }
    } else if ctx.try_get_node(id).is_some() {
        return true;
    }
    false
}
