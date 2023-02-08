use crate::ssa::{
    block::BlockId,
    context::SsaContext,
    node::{Mark, NodeId, Operation},
    {block, node},
};
use std::collections::HashSet;

// create phi arguments from the predecessors of the block (containing phi)
pub fn write_phi(ctx: &mut SsaContext, predecessors: &[BlockId], var: NodeId, phi: NodeId) {
    let mut result = Vec::new();
    for b in predecessors {
        let v = get_current_value_in_block(ctx, var, *b);
        result.push((v, *b));
    }
    let s2 = node::Instruction::simplify_phi(phi, &result);
    if let Some(phi_ins) = ctx.try_get_mut_instruction(phi) {
        let phi_args = match &mut phi_ins.operation {
            Operation::Phi { block_args, .. } => block_args,
            _ => unreachable!(),
        };

        assert_eq!(phi_args.len(), 0);
        if let Some(s_phi) = s2 {
            if s_phi != phi {
                phi_ins.mark = Mark::ReplaceWith(s_phi);
                //eventually simplify recursively: if a phi instruction is in phi use list, call simplify_phi() on it
                //but cse should deal with most of it.
            } else {
                //s2 == phi
                *phi_args = result;
            }
        } else {
            //s2 is None
            phi_ins.mark = Mark::Deleted;
        }
    }
}

pub fn seal_block(ctx: &mut SsaContext, block_id: BlockId, entry_block: BlockId) {
    let block = &ctx[block_id];
    let pred = block.predecessor.clone();
    let instructions = block.instructions.clone();
    for i in instructions {
        if let Some(ins) = ctx.try_get_instruction(i) {
            if let Operation::Phi { root, .. } = &ins.operation {
                let root = *root;
                write_phi(ctx, &pred, root, i);
            }
        }
    }
    add_dummy_store(ctx, entry_block, block_id);
    ctx.sealed_blocks.insert(block_id);
}

// write dummy store for join block
pub fn add_dummy_store(ctx: &mut SsaContext, entry: BlockId, join: BlockId) {
    //retrieve modified arrays
    let mut modified = HashSet::new();
    if entry == join {
        block::written_along(ctx, ctx[entry].right.unwrap(), join, &mut modified);
    } else {
        block::written_along(ctx, ctx[entry].left.unwrap(), join, &mut modified);
        block::written_along(ctx, ctx[entry].right.unwrap(), join, &mut modified);
    }

    //add dummy store
    for a in modified {
        let store =
            node::Operation::Store { array_id: a, index: NodeId::dummy(), value: NodeId::dummy() };
        let i = node::Instruction::new(store, node::ObjectType::NotAnObject, Some(join));
        ctx.insert_instruction_after_phi(i, join);
    }
}

//look-up recursively into predecessors
pub fn get_block_value(ctx: &mut SsaContext, root: NodeId, block_id: BlockId) -> NodeId {
    let result = if !ctx.sealed_blocks.contains(&block_id) {
        //incomplete CFG
        ctx.generate_empty_phi(block_id, root)
    } else {
        let block = &ctx[block_id];
        if let Some(idx) = block.get_current_value(root) {
            return idx;
        }
        let pred = block.predecessor.clone();
        if pred.is_empty() {
            return root;
        }
        if pred.len() == 1 {
            get_block_value(ctx, root, pred[0])
        } else {
            let result = ctx.generate_empty_phi(block_id, root);
            ctx[block_id].update_variable(root, result);
            write_phi(ctx, &pred, root, result);
            result
        }
    };

    ctx[block_id].update_variable(root, result);
    result
}

//Returns the current SSA value of a variable in a (filled) block.
pub fn get_current_value_in_block(
    ctx: &mut SsaContext,
    var_id: NodeId,
    block_id: BlockId,
) -> NodeId {
    let root = ctx.get_root_value(var_id);

    ctx[block_id]
        .get_current_value(root) //Local value numbering
        .unwrap_or_else(|| get_block_value(ctx, root, block_id)) //Global value numbering
}

//Returns the current SSA value of a variable, recursively
pub fn get_current_value(ctx: &mut SsaContext, var_id: NodeId) -> NodeId {
    get_current_value_in_block(ctx, var_id, ctx.current_block)
}
