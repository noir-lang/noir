use acvm::FieldElement;

use crate::errors::RuntimeError;

use super::{
    anchor::{Anchor, CseAction},
    block::BlockId,
    builtin,
    context::SsaContext,
    node::{Binary, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, ObjectType, Operation},
};

pub fn simplify_id(ctx: &mut SsaContext, ins_id: NodeId) -> Result<(), RuntimeError> {
    let mut ins = ctx.get_instruction(ins_id).clone();
    simplify(ctx, &mut ins)?;
    ctx[ins_id] = super::node::NodeObj::Instr(ins);
    Ok(())
}

// Performs constant folding, arithmetic simplifications and move to standard form
// Modifies ins.mark with whether the instruction should be deleted, replaced, or neither
pub fn simplify(ctx: &mut SsaContext, ins: &mut Instruction) -> Result<(), RuntimeError> {
    if ins.is_deleted() {
        return Ok(());
    }
    //1. constant folding
    let new_id = ins.evaluate(ctx)?.to_index(ctx);

    if new_id != ins.id {
        use Mark::*;
        ins.mark = if new_id == NodeId::dummy() { Deleted } else { ReplaceWith(new_id) };
        return Ok(());
    }

    //2. standard form
    ins.standard_form();
    if let Operation::Cast(value_id) = ins.operation {
        if let Some(value) = ctx.try_get_node(value_id) {
            if value.get_type() == ins.res_type {
                ins.mark = Mark::ReplaceWith(value_id);
                return Ok(());
            }
        }
    }

    //3. left-overs (it requires &mut ctx)
    if ins.is_deleted() {
        return Ok(());
    }

    if let Operation::Binary(binary) = &mut ins.operation {
        if let NodeEval::Const(r_const, r_type) = NodeEval::from_id(ctx, binary.rhs) {
            if binary.operator == BinaryOp::Div && !r_const.is_zero() {
                binary.rhs = ctx.get_or_create_const(r_const.inverse(), r_type);
                binary.operator = BinaryOp::Mul;
            }
        }
    }

    Ok(())
}

fn evaluate_intrinsic(
    ctx: &mut SsaContext,
    op: builtin::Opcode,
    args: Vec<u128>,
    res_type: &ObjectType,
    block_id: BlockId,
) -> Vec<NodeId> {
    match op {
        builtin::Opcode::ToBits => {
            let bit_count = args[1] as u32;
            let mut result = Vec::new();

            if let ObjectType::Pointer(a) = res_type {
                for i in 0..bit_count {
                    let index = ctx.get_or_create_const(
                        FieldElement::from(i as i128),
                        ObjectType::NativeField,
                    );
                    let op = if args[0] & (1 << i) != 0 {
                        Operation::Store { array_id: *a, index, value: ctx.one() }
                    } else {
                        Operation::Store { array_id: *a, index, value: ctx.zero() }
                    };
                    let i = Instruction::new(op, ObjectType::NotAnObject, Some(block_id));
                    result.push(ctx.add_instruction(i));
                }
                return result;
            }
            unreachable!();
        }
        _ => todo!(),
    }
}
////////////////////CSE////////////////////////////////////////

pub fn propagate(ctx: &SsaContext, id: NodeId, modified: &mut bool) -> NodeId {
    if let Some(obj) = ctx.try_get_instruction(id) {
        if let Mark::ReplaceWith(replacement) = obj.mark {
            *modified = true;
            return replacement;
        } else if let Operation::Binary(Binary { operator: BinaryOp::Assign, rhs, .. }) =
            &obj.operation
        {
            *modified = true;
            return *rhs;
        }
    }
    id
}

//common subexpression elimination, starting from the root
pub fn cse(
    igen: &mut SsaContext,
    first_block: BlockId,
    stop_on_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut anchor = Anchor::default();
    let mut modified = false;
    cse_tree(igen, first_block, &mut anchor, &mut modified, stop_on_error)
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
fn cse_tree(
    igen: &mut SsaContext,
    block_id: BlockId,
    anchor: &mut Anchor,
    modified: &mut bool,
    stop_on_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut instructions = Vec::new();
    let mut res =
        cse_block_with_anchor(igen, block_id, &mut instructions, anchor, modified, stop_on_error)?;
    for b in igen[block_id].dominated.clone() {
        let sub_res = cse_tree(igen, b, &mut anchor.clone(), modified, stop_on_error)?;
        if sub_res.is_some() {
            res = sub_res;
        }
    }
    Ok(res)
}

//perform common subexpression elimination until there is no more change
pub fn full_cse(
    igen: &mut SsaContext,
    first_block: BlockId,
    report_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut modified = true;
    let mut result = None;
    while modified {
        modified = false;
        let mut anchor = Anchor::default();
        result = cse_tree(igen, first_block, &mut anchor, &mut modified, report_error)?;
    }
    Ok(result)
}

pub fn simple_cse(ctx: &mut SsaContext, block_id: BlockId) -> Result<Option<NodeId>, RuntimeError> {
    let mut modified = false;
    let mut instructions = Vec::new();
    cse_block(ctx, block_id, &mut instructions, &mut modified)
}

pub fn cse_block(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
    modified: &mut bool,
) -> Result<Option<NodeId>, RuntimeError> {
    cse_block_with_anchor(ctx, block_id, instructions, &mut Anchor::default(), modified, false)
}

//Performs common subexpression elimination and copy propagation on a block
fn cse_block_with_anchor(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
    anchor: &mut Anchor,
    modified: &mut bool,
    stop_on_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut new_list = Vec::new();
    let bb = &ctx[block_id];
    let is_join = bb.predecessor.len() > 1;
    if instructions.is_empty() {
        instructions.append(&mut bb.instructions.clone());
    }

    for ins_id in instructions {
        if let Some(ins) = ctx.try_get_instruction(*ins_id) {
            if ins.is_deleted() {
                continue;
            }

            let operator = ins.operation.map_id(|id| propagate(ctx, id, modified));

            let mut new_mark = Mark::None;

            match &operator {
                Operation::Binary(binary) => {
                    if let ObjectType::Pointer(a) = ctx.get_object_type(binary.lhs) {
                        //No CSE for arrays because they are not in SSA form
                        //We could improve this in future by checking if the arrays are immutable or not modified in-between
                        let id = ctx.get_dummy_load(a);
                        anchor.push_mem_instruction(ctx, id);

                        if let ObjectType::Pointer(a) = ctx.get_object_type(binary.rhs) {
                            let id = ctx.get_dummy_load(a);
                            anchor.push_mem_instruction(ctx, id);
                        }

                        new_list.push(*ins_id);
                    } else if let Some(similar) = anchor.find_similar_instruction(&operator) {
                        debug_assert!(similar != ins.id);
                        *modified = true;
                        new_mark = Mark::ReplaceWith(similar);
                    } else if binary.operator == BinaryOp::Assign {
                        *modified = true;
                        new_mark = Mark::ReplaceWith(binary.rhs);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&ins.operation, *ins_id);
                    }
                }
                Operation::Load { array_id: x, .. } | Operation::Store { array_id: x, .. } => {
                    if !is_join && ins.operation.is_dummy_store() {
                        continue;
                    }
                    anchor.use_array(*x, ctx.mem[*x].len as usize);
                    let prev_ins = anchor.get_mem_all(*x);
                    match anchor.find_similar_mem_instruction(ctx, &operator, prev_ins) {
                        CseAction::Keep => {
                            anchor.push_mem_instruction(ctx, *ins_id);
                            new_list.push(*ins_id)
                        }
                        CseAction::ReplaceWith(new_id) => {
                            *modified = true;
                            new_mark = Mark::ReplaceWith(new_id);
                        }
                        CseAction::Remove(id_to_remove) => {
                            anchor.push_mem_instruction(ctx, *ins_id);
                            new_list.push(*ins_id);
                            // TODO if not found, it should be removed from other blocks; we could keep a list of instructions to remove
                            if let Some(id) = new_list.iter().position(|x| *x == id_to_remove) {
                                *modified = true;
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
                            *modified = true;
                            new_mark = Mark::ReplaceWith(first);
                        }
                    } else {
                        new_mark = Mark::Deleted;
                    }
                }
                Operation::Cast(_) => {
                    //Similar cast must have same type
                    if let Some(similar) = anchor.find_similar_cast(ctx, &operator, ins.res_type) {
                        new_mark = Mark::ReplaceWith(similar);
                        *modified = true;
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_cast_front(&operator, *ins_id, ins.res_type);
                    }
                }
                Operation::Call { func, arguments, returned_arrays, .. } => {
                    //No CSE for function calls because of possible side effect - TODO checks if a function has side effect when parsed and do cse for these.
                    //Add dummy store for functions that modify arrays
                    for a in returned_arrays {
                        let id = ctx.get_dummy_store(a.0);
                        anchor.push_mem_instruction(ctx, id);
                    }
                    if let Some(f) = ctx.try_get_ssafunc(*func) {
                        for typ in &f.result_types {
                            if let ObjectType::Pointer(a) = typ {
                                let id = ctx.get_dummy_store(*a);
                                anchor.push_mem_instruction(ctx, id);
                            }
                        }
                    }
                    //Add dummy load for function arguments:
                    for arg in arguments {
                        if let Some(obj) = ctx.try_get_node(*arg) {
                            if let ObjectType::Pointer(a) = obj.get_type() {
                                let id = ctx.get_dummy_load(a);
                                anchor.push_mem_instruction(ctx, id);
                            }
                        }
                    }
                    new_list.push(*ins_id);
                }
                Operation::Return(..) => new_list.push(*ins_id),
                Operation::Intrinsic(opcode, args) => {
                    // dbg!(args.clone());
                    //Add dunmmy load for function arguments and enable CSE only if no array in argument
                    let mut activate_cse = true;
                    match opcode {
                        // We do not want to replace any print intrinsics as we want them to remain in order and unchanged
                        builtin::Opcode::Println(_) => activate_cse = false,
                        _ => (),
                    }

                    for arg in args {
                        if let Some(obj) = ctx.try_get_node(*arg) {
                            if let ObjectType::Pointer(a) = obj.get_type() {
                                let id = ctx.get_dummy_load(a);
                                anchor.push_mem_instruction(ctx, id);
                                activate_cse = false;
                            }
                        }
                    }
                    if let ObjectType::Pointer(a) = ins.res_type {
                        let id = ctx.get_dummy_store(a);
                        anchor.push_mem_instruction(ctx, id);
                        activate_cse = false;
                    }

                    if activate_cse {
                        if let Some(similar) = anchor.find_similar_instruction(&operator) {
                            dbg!(similar);
                            *modified = true;
                            new_mark = Mark::ReplaceWith(similar);
                        } else {
                            new_list.push(*ins_id);
                            anchor.push_front(&operator, *ins_id);
                        }
                    } else {
                        new_list.push(*ins_id);
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
            update.parent_block = block_id;

            let mut update2 = update.clone();

            let result = simplify(ctx, &mut update2);
            if stop_on_error {
                result?;
            }

            //cannot simplify to_bits() in the previous call because it get replaced with multiple instructions
            if let Operation::Intrinsic(opcode, args) = &update2.operation {
                match opcode {
                    // We do not simplify print statements
                    builtin::Opcode::Println(_) => (),
                    _ => {
                        let args = args.iter().map(|arg| {
                            NodeEval::from_id(ctx, *arg).into_const_value().map(|f| f.to_u128())
                        });

                        if let Some(args) = args.collect() {
                            update2.mark = Mark::Deleted;
                            new_list.extend(evaluate_intrinsic(
                                ctx,
                                *opcode,
                                args,
                                &update2.res_type,
                                block_id,
                            ));
                        }
                    }
                }
            }
            let update3 = ctx.get_mut_instruction(*ins_id);
            *update3 = update2;
        }
    }

    let last = new_list.iter().copied().rev().find(|id| is_some(ctx, *id));
    ctx[block_id].instructions = new_list;
    Ok(last)
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
