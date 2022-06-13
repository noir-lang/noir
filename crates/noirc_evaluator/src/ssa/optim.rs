use acvm::FieldElement;

use super::{
    acir_gen::InternalVar,
    block::BlockId,
    context::SsaContext,
    node::{
        self, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, ObjectType, Opcode, Operation,
        Variable,
    },
};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

// Performs constant folding, arithmetic simplifications and move to standard form
// Modifies ins.mark with whether the instruction should be deleted, replaced, or neither
pub fn simplify(ctx: &mut SsaContext, ins: &mut node::Instruction) {
    //1. constant folding
    let new_id = match ins.evaluate(ctx) {
        NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    };

    if new_id != ins.id {
        use Mark::*;
        ins.mark = if new_id == NodeId::dummy() { Deleted } else { ReplaceWith(new_id) };
        return;
    }

    //2. standard form
    ins.standard_form();
    if let Operation::Cast(value_id) = ins.operation {
        if let Some(value) = ctx.try_get_node(value_id) {
            if value.get_type() == ins.res_type {
                ins.mark = Mark::ReplaceWith(value_id);
                return;
            }
        }
    }

    //3. left-overs (it requires &mut ctx)
    if let Operation::Binary(binary) = &mut ins.operation {
        if let NodeEval::Const(r_const, r_type) = NodeEval::from_id(ctx, binary.rhs) {
            match &binary.operator {
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

    if let Operation::Intrinsic(opcode, args) = &ins.operation {
        let args = args
            .iter()
            .map(|arg| NodeEval::from_id(ctx, *arg).into_const_value().map(|f| f.to_u128()));

        if let Some(args) = args.collect() {
            ins.mark = Mark::ReplaceWith(evaluate_intrinsic(ctx, *opcode, args));
        }
    }
}

fn evaluate_intrinsic(ctx: &mut SsaContext, op: acvm::acir::OPCODE, args: Vec<u128>) -> NodeId {
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

            ctx.add_variable(pointer, None)
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
            if &ins.operation == operation {
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
            if &ins.operation == operator && ins.res_type == res_type {
                return Some(*iter);
            }
        }
    }
    None
}

pub enum CseAction {
    ReplaceWith(NodeId),
    Remove(NodeId),
    Keep,
}

fn find_similar_mem_instruction(
    ctx: &SsaContext,
    op: &Operation,
    anchor: &mut Anchor,
) -> CseAction {
    match op {
        Operation::Load { array_id, index } => {
            for iter in anchor.get_all(op.opcode()).iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*iter) {
                    match &ins_iter.operation {
                        Operation::Load { array_id: array_id2, index: _ } => {
                            assert_eq!(array_id, array_id2);
                            return CseAction::ReplaceWith(*iter);
                        }
                        Operation::Store { array_id: array_id2, index: index2, value } => {
                            assert_eq!(array_id, array_id2);
                            if index == index2 {
                                return CseAction::ReplaceWith(*value);
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
            let opcode = Opcode::Load(*array_id);
            for node_id in anchor.get_all(opcode).iter().rev() {
                if let Some(ins_iter) = ctx.try_get_instruction(*node_id) {
                    match ins_iter.operation {
                        Operation::Load { array_id: array_id2, .. } => {
                            assert_eq!(*array_id, array_id2);
                            //TODO: If we know that ins.rhs value cannot be equal to ins_iter.rhs, we could continue instead
                            return CseAction::Keep;
                        }
                        Operation::Store { index: index2, array_id: array_id2, .. } => {
                            assert_eq!(*array_id, array_id2);
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
    if let Some(obj) = ctx.try_get_instruction(id) {
        if let Mark::ReplaceWith(replacement) = obj.mark {
            return replacement;
        } else if let Operation::Binary(node::Binary { operator: BinaryOp::Assign, rhs, .. }) =
            &obj.operation
        {
            return *rhs;
        }
    }
    id
}

//common subexpression elimination, starting from the root
pub fn cse(igen: &mut SsaContext, first_block: BlockId) -> Option<NodeId> {
    let mut anchor = Anchor::default();
    cse_tree(igen, first_block, &mut anchor)
}

/// A list of instructions with the same Operation variant, ordered by the order
/// they appear in their respective blocks.
#[derive(Default, Clone)]
struct Anchor {
    map: HashMap<Opcode, VecDeque<NodeId>>,
}

impl Anchor {
    fn push_front(&mut self, op: &Operation, id: NodeId) {
        let key = match op {
            Operation::Store { array_id, .. } => Opcode::Load(*array_id),
            _ => op.opcode(),
        };
        self.map.entry(key).or_insert_with(VecDeque::new).push_front(id);
    }

    fn get_all(&self, opcode: Opcode) -> Cow<VecDeque<NodeId>> {
        match self.map.get(&opcode) {
            Some(vec) => Cow::Borrowed(vec),
            None => Cow::Owned(VecDeque::new()),
        }
    }
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
fn cse_tree(igen: &mut SsaContext, block_id: BlockId, anchor: &mut Anchor) -> Option<NodeId> {
    let mut instructions = Vec::new();
    let mut res = cse_block_with_anchor(igen, block_id, &mut instructions, anchor);
    for b in igen[block_id].dominated.clone() {
        let sub_res = cse_tree(igen, b, &mut anchor.clone());
        if sub_res.is_some() {
            res = sub_res;
        }
    }
    res
}

pub fn cse_block(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
) -> Option<NodeId> {
    cse_block_with_anchor(ctx, block_id, instructions, &mut Anchor::default())
}

//Performs common subexpression elimination and copy propagation on a block
fn cse_block_with_anchor(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
    anchor: &mut Anchor,
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

            let operator = ins.operation.map_id(|id| propagate(ctx, id));

            let mut new_mark = Mark::None;

            match &operator {
                Operation::Binary(binary) => {
                    let variants = anchor.get_all(binary.opcode());
                    if let Some(similar) = find_similar_instruction(ctx, &operator, &variants) {
                        new_mark = Mark::ReplaceWith(similar);
                    } else if binary.operator == BinaryOp::Assign {
                        new_mark = Mark::ReplaceWith(binary.rhs);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&ins.operation, *ins_id);
                    }
                }
                Operation::Load { .. } | Operation::Store { .. } => {
                    match find_similar_mem_instruction(ctx, &operator, anchor) {
                        CseAction::Keep => new_list.push(*ins_id),
                        CseAction::ReplaceWith(new_id) => {
                            new_mark = Mark::ReplaceWith(new_id);
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
                            new_mark = Mark::ReplaceWith(first);
                        }
                    } else {
                        new_mark = Mark::Deleted;
                    }
                }
                Operation::Cast(_) => {
                    //Similar cast must have same type
                    if let Some(similar) = find_similar_cast(
                        ctx,
                        &operator,
                        ins.res_type,
                        &anchor.get_all(Opcode::Cast),
                    ) {
                        new_mark = Mark::ReplaceWith(similar);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&operator, *ins_id);
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
                        find_similar_instruction(ctx, &operator, &anchor.get_all(operator.opcode()))
                    {
                        new_mark = Mark::ReplaceWith(similar);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&operator, *ins_id);
                    }
                }
                _ => {
                    //TODO: checks we do not need to propagate res arguments
                    new_list.push(*ins_id);
                }
            }

            let update = ctx.get_mut_instruction(*ins_id);
            update.operation = operator;
            update.mark = new_mark;
            if new_mark == Mark::Deleted {
                update.operation = Operation::Nop;
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
        if ins.operation != Operation::Nop {
            return true;
        }
    } else if ctx.try_get_node(id).is_some() {
        return true;
    }
    false
}
