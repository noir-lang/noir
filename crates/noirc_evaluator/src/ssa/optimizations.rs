use crate::errors::RuntimeError;
use crate::ssa::{
    context::SsaContext,
    node::{Binary, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, Operation},
};
use acvm::FieldElement;

pub fn simplify_id(ctx: &mut SsaContext, ins_id: NodeId) -> Result<(), RuntimeError> {
    let mut ins = ctx.get_instruction(ins_id).clone();
    simplify(ctx, &mut ins)?;
    ctx[ins_id] = super::node::NodeObject::Instr(ins);
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
    if let Operation::Binary(binary) = &ins.operation {
        if binary.operator == BinaryOp::Xor {
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
