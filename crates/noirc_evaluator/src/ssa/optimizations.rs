use crate::errors::{RuntimeError, RuntimeErrorKind};
use crate::ssa::{
    anchor::{Anchor, CseAction},
    block::BlockId,
    builtin,
    context::SsaContext,
    node::{
        Binary, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, ObjectType, Opcode, Operation,
    },
};
use acvm::FieldElement;
use num_bigint::BigUint;

pub(super) fn simplify_id(ctx: &mut SsaContext, ins_id: NodeId) -> Result<(), RuntimeError> {
    let mut ins = ctx.instruction(ins_id).clone();
    simplify(ctx, &mut ins)?;
    ctx[ins_id] = super::node::NodeObject::Instr(ins);
    Ok(())
}

// Performs constant folding, arithmetic simplifications and move to standard form
// Modifies ins.mark with whether the instruction should be deleted, replaced, or neither
pub(super) fn simplify(ctx: &mut SsaContext, ins: &mut Instruction) -> Result<(), RuntimeError> {
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
            if binary.opcode() == Opcode::Div && !r_const.is_zero() {
                binary.rhs = ctx.get_or_create_const(r_const.inverse(), r_type);
                binary.operator = BinaryOp::Mul;
            }
        }
    }
    if let Operation::Binary(binary) = &ins.operation {
        if binary.operator == BinaryOp::Xor && ins.res_type.bits() < 128 {
            let max = FieldElement::from(2_u128.pow(ins.res_type.bits()) - 1);
            if NodeEval::from_id(ctx, binary.rhs).into_const_value() == Some(max) {
                ins.operation = Operation::Not(binary.lhs);
            } else if NodeEval::from_id(ctx, binary.lhs).into_const_value() == Some(max) {
                ins.operation = Operation::Not(binary.rhs);
            }
        }
    }

    Ok(())
}

fn evaluate_intrinsic(
    ctx: &mut SsaContext,
    op: builtin::Opcode,
    args: Vec<FieldElement>,
    res_type: &ObjectType,
    block_id: BlockId,
) -> Result<Vec<NodeId>, RuntimeErrorKind> {
    match op {
        builtin::Opcode::ToBits(endian) => {
            let bit_count = args[1].to_u128() as u32;
            let mut result = Vec::new();
            let mut bits = args[0].bits();
            bits.reverse();
            bits.resize(bit_count as usize, false);
            if endian == builtin::Endian::Big {
                bits.reverse();
            }

            if let ObjectType::ArrayPointer(a) = res_type {
                for i in 0..bit_count {
                    let index = ctx.get_or_create_const(
                        FieldElement::from(i as i128),
                        ObjectType::native_field(),
                    );
                    let op = if i < bits.len() as u32 && bits[i as usize] {
                        Operation::Store {
                            array_id: *a,
                            index,
                            value: ctx.one(),
                            predicate: None,
                            location: None,
                        }
                    } else {
                        Operation::Store {
                            array_id: *a,
                            index,
                            value: ctx.zero(),
                            predicate: None,
                            location: None,
                        }
                    };
                    let i = Instruction::new(op, ObjectType::NotAnObject, Some(block_id));
                    result.push(ctx.add_instruction(i));
                }
                return Ok(result);
            }
            unreachable!(
                "compiler error: to bits should have a Pointer result type and be decomposed."
            );
        }
        builtin::Opcode::ToRadix(endian) => {
            let mut element = BigUint::from_bytes_be(&args[0].to_be_bytes())
                .to_radix_le(args[1].to_u128() as u32);
            let byte_count = args[2].to_u128() as u32;
            let diff = if byte_count >= element.len() as u32 {
                byte_count - element.len() as u32
            } else {
                return Err(RuntimeErrorKind::ArrayOutOfBounds {
                    index: element.len() as u128,
                    bound: byte_count as u128,
                });
            };
            element.extend(vec![0; diff as usize]);
            if endian == builtin::Endian::Big {
                element.reverse();
            }
            let mut result = Vec::new();

            if let ObjectType::ArrayPointer(a) = res_type {
                for (i, item) in element.iter().enumerate() {
                    let index = ctx.get_or_create_const(
                        FieldElement::from(i as i128),
                        ObjectType::native_field(),
                    );
                    let value = ctx.get_or_create_const(
                        FieldElement::from(*item as i128),
                        ObjectType::native_field(),
                    );
                    let op = Operation::Store {
                        array_id: *a,
                        index,
                        value,
                        predicate: None,
                        location: None,
                    };

                    let i = Instruction::new(op, ObjectType::NotAnObject, Some(block_id));
                    result.push(ctx.add_instruction(i));
                }
                return Ok(result);
            }
            unreachable!(
                "compiler error: to radix should have a Pointer result type and be decomposed."
            );
        }
        _ => todo!(),
    }
}

//
// The following code will be concerned with Common Subexpression Elimination (CSE)
//

pub(super) fn propagate(ctx: &SsaContext, id: NodeId, modified: &mut bool) -> NodeId {
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
pub(super) fn cse(
    ir_gen: &mut SsaContext,
    first_block: BlockId,
    stop_on_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut anchor = Anchor::default();
    let mut modified = false;
    cse_tree(ir_gen, first_block, &mut anchor, &mut modified, stop_on_error)
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
fn cse_tree(
    ir_gen: &mut SsaContext,
    block_id: BlockId,
    anchor: &mut Anchor,
    modified: &mut bool,
    stop_on_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut instructions = Vec::new();
    let mut res = cse_block_with_anchor(
        ir_gen,
        block_id,
        &mut instructions,
        anchor,
        modified,
        stop_on_error,
    )?;
    for b in ir_gen[block_id].dominated.clone() {
        let sub_res = cse_tree(ir_gen, b, &mut anchor.clone(), modified, stop_on_error)?;
        if sub_res.is_some() {
            res = sub_res;
        }
    }
    Ok(res)
}

//perform common subexpression elimination until there is no more change
pub(super) fn full_cse(
    ir_gen: &mut SsaContext,
    first_block: BlockId,
    report_error: bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut modified = true;
    let mut result = None;
    while modified {
        modified = false;
        let mut anchor = Anchor::default();
        result = cse_tree(ir_gen, first_block, &mut anchor, &mut modified, report_error)?;
    }
    Ok(result)
}

pub(super) fn simple_cse(
    ctx: &mut SsaContext,
    block_id: BlockId,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut modified = false;
    let mut instructions = Vec::new();
    cse_block(ctx, block_id, &mut instructions, &mut modified)
}

pub(super) fn cse_block(
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
            let mut operator = ins.operation.map_id(|id| propagate(ctx, id, modified));

            let mut new_mark = Mark::None;

            match &operator {
                Operation::Binary(binary) => {
                    if let ObjectType::ArrayPointer(a) = ctx.object_type(binary.lhs) {
                        //No CSE for arrays because they are not in SSA form
                        //We could improve this in future by checking if the arrays are immutable or not modified in-between
                        let id = ctx.get_dummy_load(a);
                        anchor.push_mem_instruction(ctx, id)?;

                        if let ObjectType::ArrayPointer(a) = ctx.object_type(binary.rhs) {
                            let id = ctx.get_dummy_load(a);
                            anchor.push_mem_instruction(ctx, id)?;
                        }

                        new_list.push(*ins_id);
                    } else if let Some(similar) = anchor.find_similar_instruction(&operator) {
                        assert_ne!(similar, ins.id);
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
                Operation::Result { .. } => {
                    if let Some(similar) = anchor.find_similar_instruction(&operator) {
                        assert_ne!(similar, ins.id);
                        *modified = true;
                        new_mark = Mark::ReplaceWith(similar);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&ins.operation, *ins_id);
                    }
                }
                Operation::Load { array_id: x, location, .. }
                | Operation::Store { array_id: x, location, .. } => {
                    if !is_join && ins.operation.is_dummy_store() {
                        continue;
                    }
                    anchor.use_array(*x, ctx.mem[*x].len as usize);
                    let prev_ins = anchor.get_mem_all(*x);
                    let into_runtime_error =
                        |err: RuntimeErrorKind| RuntimeError { location: *location, kind: err };
                    match anchor.find_similar_mem_instruction(ctx, &operator, prev_ins) {
                        Ok(CseAction::Keep) => {
                            anchor
                                .push_mem_instruction(ctx, *ins_id)
                                .map_err(into_runtime_error)?;
                            new_list.push(*ins_id);
                        }
                        Ok(CseAction::ReplaceWith(new_id)) => {
                            *modified = true;
                            new_mark = Mark::ReplaceWith(new_id);
                        }
                        Ok(CseAction::Remove(id_to_remove)) => {
                            anchor
                                .push_mem_instruction(ctx, *ins_id)
                                .map_err(into_runtime_error)?;
                            // TODO if not found, it should be removed from other blocks; we could keep a list of instructions to remove
                            if let Some(id) = new_list.iter().position(|x| *x == id_to_remove) {
                                *modified = true;
                                new_list.remove(id);
                            }
                            // Store with predicate must be merged with the previous store
                            if let Operation::Store {
                                index: idx,
                                value: value2,
                                predicate: Some(predicate2),
                                location: location1,
                                ..
                            } = operator
                            {
                                if let Operation::Store {
                                    value: value1,
                                    predicate: predicate1,
                                    location: location2,
                                    ..
                                } = ctx.instruction(id_to_remove).operation
                                {
                                    let (merge, pred) = if let Some(predicate1) = predicate1 {
                                        if predicate1 != predicate2 {
                                            let or_op = Operation::Binary(Binary {
                                                lhs: predicate1,
                                                rhs: predicate2,
                                                operator: BinaryOp::Or,
                                                predicate: None,
                                            });
                                            let pred_id = ctx.add_instruction(Instruction::new(
                                                or_op,
                                                ObjectType::boolean(),
                                                Some(block_id),
                                            ));
                                            new_list.push(pred_id);
                                            (true, Some(pred_id))
                                        } else {
                                            (false, None)
                                        }
                                    } else {
                                        (true, None)
                                    };
                                    if merge {
                                        *modified = true;
                                        let cond_op = Operation::Cond {
                                            condition: predicate2,
                                            val_true: value2,
                                            val_false: value1,
                                        };
                                        let cond_id = ctx.add_instruction(Instruction::new(
                                            cond_op,
                                            ctx.object_type(value2),
                                            Some(block_id),
                                        ));
                                        new_list.push(cond_id);
                                        operator = Operation::Store {
                                            array_id: *x,
                                            index: idx,
                                            value: cond_id,
                                            predicate: pred,
                                            location: RuntimeError::merge_location(
                                                location1, location2,
                                            ),
                                        };
                                    }
                                } else {
                                    unreachable!("ICE: expected store instruction")
                                }
                            }
                            new_list.push(*ins_id);
                        }
                        Err(err) => {
                            return Err(RuntimeError { location: *location, kind: err });
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
                        anchor.push_mem_instruction(ctx, id)?;
                    }
                    if let Some(f) = ctx.try_get_ssa_func(*func) {
                        for typ in &f.result_types {
                            if let ObjectType::ArrayPointer(a) = typ {
                                let id = ctx.get_dummy_store(*a);
                                anchor.push_mem_instruction(ctx, id)?;
                            }
                        }
                    }
                    //Add dummy load for function arguments:
                    for arg in arguments {
                        if let Some(obj) = ctx.try_get_node(*arg) {
                            if let ObjectType::ArrayPointer(a) = obj.get_type() {
                                let id = ctx.get_dummy_load(a);
                                anchor.push_mem_instruction(ctx, id)?;
                            }
                        }
                    }
                    new_list.push(*ins_id);
                }
                Operation::Return(..) => new_list.push(*ins_id),
                Operation::Intrinsic(opcode, args) => {
                    //Add dummy load for function arguments and enable CSE only if no array in argument
                    let mut activate_cse = true;
                    // We do not want to replace any print intrinsics as we want them to remain in order and unchanged
                    if let builtin::Opcode::Println(_) = opcode {
                        activate_cse = false;
                    }

                    for arg in args {
                        if let Some(obj) = ctx.try_get_node(*arg) {
                            if let ObjectType::ArrayPointer(a) = obj.get_type() {
                                let id = ctx.get_dummy_load(a);
                                anchor.push_mem_instruction(ctx, id)?;
                                activate_cse = false;
                            }
                        }
                    }
                    if let ObjectType::ArrayPointer(a) = ins.res_type {
                        let id = ctx.get_dummy_store(a);
                        anchor.push_mem_instruction(ctx, id)?;
                        activate_cse = false;
                    }

                    if activate_cse {
                        if let Some(similar) = anchor.find_similar_instruction(&operator) {
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
                Operation::Nop => {
                    if new_list.is_empty() {
                        new_list.push(*ins_id);
                    }
                }
                Operation::Constrain(condition, location) => {
                    if let Some(similar) = anchor.find_similar_instruction(&operator) {
                        assert_ne!(similar, ins.id);
                        *modified = true;
                        let similar_ins = ctx
                            .try_get_mut_instruction(similar)
                            .expect("Similar instructions are instructions");
                        if location.is_some() && similar_ins.get_location().is_none() {
                            similar_ins.operation = Operation::Constrain(*condition, *location);
                        }
                        new_mark = Mark::ReplaceWith(similar);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(&ins.operation, *ins_id);
                    }
                }
                _ => {
                    //TODO: checks we do not need to propagate res arguments
                    new_list.push(*ins_id);
                }
            }

            let update = ctx.instruction_mut(*ins_id);

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

            //cannot simplify to_le_bits() in the previous call because it get replaced with multiple instructions
            if let Operation::Intrinsic(opcode, args) = &update2.operation {
                match opcode {
                    // We do not simplify print statements
                    builtin::Opcode::Println(_) => (),
                    _ => {
                        let args =
                            args.iter().map(|arg| NodeEval::from_id(ctx, *arg).into_const_value());

                        if let Some(args) = args.collect() {
                            update2.mark = Mark::Deleted;
                            new_list.extend(evaluate_intrinsic(
                                ctx,
                                *opcode,
                                args,
                                &update2.res_type,
                                block_id,
                            )?);
                        }
                    }
                }
            }
            let update3 = ctx.instruction_mut(*ins_id);
            *update3 = update2;
        }
    }

    let last = new_list.iter().copied().rev().find(|id| is_some(ctx, *id));
    ctx[block_id].instructions = new_list;
    Ok(last)
}

fn is_some(ctx: &SsaContext, id: NodeId) -> bool {
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
