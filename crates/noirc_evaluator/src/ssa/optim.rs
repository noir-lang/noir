use acvm::FieldElement;

use super::{
    acir_gen::InternalVar,
    block::BlockId,
    context::SsaContext,
    mem::Memory,
    node::{
        self, BinaryOp, ConstrainOp, Instruction, Node, NodeEval, NodeId, ObjectType, Operation,
        Variable,
    },
};
use std::collections::{HashMap, VecDeque};

// Performs constant folding, arithmetic simplifications and move to standard form
// Returns a new NodeId to replace with the current if the current is deleted.
pub fn simplify(ctx: &mut SsaContext, ins: &mut node::Instruction) -> Option<NodeId> {
    //1. constant folding
    let new_id = match ins.evaluate(ctx) {
        NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    };

    if new_id != ins.id {
        ins.replacement = Some(new_id);
        if new_id == NodeId::dummy() {
            ins.delete();
        }
        return Some(new_id);
    }

    //2. standard form
    ins.standard_form();
    match ins.operator {
        Operation::Cast(value_id) => {
            if let Some(value) = ctx.try_get_node(value_id) {
                if value.get_type() == ins.res_type {
                    ins.delete();
                    return Some(NodeId::dummy());
                }
            }
        }
        Operation::Binary(node::Binary { operator: BinaryOp::Constrain(op), lhs, rhs }) => {
            match (op, Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
                (ConstrainOp::Eq, Some(lhs), Some(rhs)) if lhs == rhs => {
                    ins.delete();
                }
                _ => (),
            }
        }
        _ => (),
    }

    //3. left-overs (it requires &mut ctx)
    if let Operation::Binary(binary) = &mut ins.operator {
        if let NodeEval::Const(r_const, r_type) = NodeEval::from_id(ctx, binary.rhs) {
            match &binary.operator {
                BinaryOp::Udiv => {
                    //TODO handle other bitsize, not only u128!!
                    let inverse = FieldElement::from(1 / r_const.to_u128());
                    binary.rhs = ctx.get_or_create_const(inverse, r_type);
                    binary.operator = BinaryOp::Mul;
                }
                BinaryOp::Sdiv => {
                    //TODO handle other bitsize, not only u128!!
                    let inverse = FieldElement::from(1 / r_const.to_u128());
                    binary.rhs = ctx.get_or_create_const(inverse, r_type);
                    binary.operator = BinaryOp::Mul;
                }
                BinaryOp::Div => {
                    binary.rhs = ctx.get_or_create_const(r_const.inverse(), r_type);
                    binary.operator = BinaryOp::Mul;
                }
                BinaryOp::Shl => {
                    binary.operator = BinaryOp::Mul;
                    //todo checks that 2^rhs does not overflow
                    binary.rhs =
                        ctx.get_or_create_const(FieldElement::from(2_i128).pow(&r_const), r_type);
                }
                BinaryOp::Shr => {
                    if !matches!(ins.res_type, node::ObjectType::Unsigned(_)) {
                        todo!("Right shift is only implemented for unsigned integers");
                    }
                    binary.operator = BinaryOp::Udiv;
                    //todo checks that 2^rhs does not overflow
                    binary.rhs =
                        ctx.get_or_create_const(FieldElement::from(2_i128).pow(&r_const), r_type);
                }
                _ => (),
            }
        }
    }

    if let Operation::Intrinsic(opcode, args) = &ins.operator {
        let args = args
            .iter()
            .map(|arg| NodeEval::from_id(ctx, *arg).into_const_value().map(|f| f.to_u128()));

        if let Some(args) = args.collect() {
            evaluate_intrinsic(ctx, *opcode, args);
        }
    }

    None
}

fn evaluate_intrinsic(ctx: &mut SsaContext, op: acvm::acir::OPCODE, args: Vec<u128>) -> NodeEval {
    match op {
        acvm::acir::OPCODE::ToBits => {
            let bit_count = args[1] as u32;
            let array_id = ctx.mem.create_new_array(bit_count, ObjectType::Unsigned(1), "");
            let pointer = Variable {
                id: NodeId::dummy(),
                obj_type: ObjectType::Pointer(array_id),
                root: None,
                name: String::new(),
                def: None,
                witness: None,
                parent_block: ctx.current_block,
            };

            for i in 0..bit_count {
                if args[0] & (1 << i) != 0 {
                    ctx.mem[array_id].values.push(InternalVar::from(FieldElement::one()));
                } else {
                    ctx.mem[array_id].values.push(InternalVar::from(FieldElement::zero()));
                }
            }

            let new_var = ctx.add_variable(pointer, None);
            NodeEval::VarOrInstruction(new_var)
        }
        _ => todo!(),
    }
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
    Replace { original: NodeId, replacement: NodeId },
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
        Operation::Load { array_id, index } => {
            for iter in anchor[op].iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*iter) {
                    match &ins_iter.operator {
                        Operation::Load { array_id: array_id2, index: _ } => {
                            assert_eq!(array_id, array_id2);
                            return CseAction::Replace { original: ins_id, replacement: *iter };
                        }
                        Operation::Store { array_id: array_id2, index: index2, value } => {
                            assert_eq!(array_id, array_id2);
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
        Operation::Store { array_id, index, value: _ } => {
            let prev_ins = &anchor[&Operation::Load { array_id: *array_id, index: *index }];
            for node_id in prev_ins.iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*node_id) {
                    match ins_iter.operator {
                        Operation::Load { .. } => {
                            //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                            return CseAction::Keep;
                        }
                        Operation::Store { index: index2, array_id: _, value: _ } => {
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
        if let Operation::Binary(node::Binary { operator: BinaryOp::Assign, rhs, .. }) =
            &obj.operator
        {
            result = *rhs;
        }

        if let Some(replacement) = obj.replacement {
            result = replacement;
        }
    }
    result
}

//common subexpression elimination, starting from the root
pub fn cse(igen: &mut SsaContext, first_block: BlockId) -> Option<NodeId> {
    let mut anchor = HashMap::new();
    cse_tree(igen, first_block, &mut anchor)
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
        Operation::Store { array_id, index, .. } => anchor
            // TODO: review correctness
            .entry(Operation::Load { array_id, index })
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
            if ins.is_deleted() {
                continue;
            }
            anchor_push(ins.operator.clone(), anchor);

            let operator = ins.operator.map_id(|id| propagate(ctx, id));

            let mut to_delete = false;
            let mut replacement = None;

            if operator.is_binary() {
                //binary operation:
                if let Some(similar) =
                    find_similar_instruction(ctx, &ins.operator, &anchor[&ins.operator])
                {
                    replacement = Some(similar);
                } else if let Operation::Binary(node::Binary {
                    operator: BinaryOp::Assign,
                    rhs,
                    ..
                }) = operator
                {
                    replacement = Some(propagate(ctx, rhs));
                } else {
                    new_list.push(*ins_id);
                    anchor.get_mut(&ins.operator).unwrap().push_front(*ins_id);
                }
            } else {
                match &operator {
                    Operation::Load { .. } | Operation::Store { .. } => {
                        match find_similar_mem_instruction(ctx, &operator, ins.id, anchor) {
                            CseAction::Keep => new_list.push(*ins_id),
                            CseAction::Replace { original, replacement: replace_with } => {
                                assert_eq!(original, *ins_id);
                                replacement = Some(replace_with);
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
                                replacement = Some(first);
                            }
                        } else {
                            to_delete = true;
                        }
                    }
                    Operation::Cast(_) => {
                        //Similar cast must have same type
                        if let Some(similar) =
                            find_similar_cast(ctx, &operator, ins.res_type, &anchor[&operator])
                        {
                            replacement = Some(similar);
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
                        if let Some(similar) =
                            find_similar_instruction(ctx, &operator, &anchor[&operator])
                        {
                            replacement = Some(similar);
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
            update.replacement = replacement;
            if to_delete {
                update.delete();
            }
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
