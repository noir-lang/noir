use std::collections::{HashMap, HashSet};

use crate::ssa::node::{Mark, Operation};
use noirc_frontend::node_interner::DefinitionId;

use super::{
    block::BlockId,
    code_gen::IRGenerator,
    context::SsaContext,
    node::{self, NodeId},
};

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

pub fn seal_block(ctx: &mut SsaContext, block_id: BlockId) {
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

    if pred.len() > 1 {
        let mut u: HashSet<super::mem::ArrayId> = HashSet::new();
        for block in pred {
            u.extend(ctx[block].written_arrays(ctx));
        }
        for array in u {
            let store = Operation::Store {
                array_id: array,
                index: NodeId::dummy(),
                value: NodeId::dummy(),
            };
            let i = node::Instruction::new(store, node::ObjectType::NotAnObject, Some(block_id));
            ctx.insert_instruction_after_phi(i, block_id);
        }
    }

    ctx.sealed_blocks.insert(block_id);
}

//look-up recursiverly into predecessors
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

pub fn create_function_parameter(
    igen: &mut IRGenerator,
    ident_id: &DefinitionId,
    arguments: &HashMap<usize, u32>,
    pos: &mut usize,
    template_args: &mut Vec<(usize, u32)>,
) -> Vec<NodeId> {
    let ident_name = igen.def_to_name(*ident_id);
    let o_type = igen.def_interner().id_type(*ident_id);
    //check if the variable is already created:
    let val = if let Some(var) = igen.find_variable(*ident_id) {
        let clone_var = var.clone();
        if let noirc_frontend::Type::Array(len, _) = o_type {
            if len == noirc_frontend::ArraySize::Variable {
                template_args.push((*pos, arguments[pos]));
            }
        }
        *pos += clone_var.len();
        igen.get_current_value(&clone_var)
    } else {
        igen.create_new_value(&o_type, &ident_name, Some(*ident_id), arguments, pos, template_args)
    };
    val.to_node_ids()
}
