use noirc_frontend::{node_interner::DefinitionId, ArraySize};

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
        assert!(phi_ins.phi_arguments.is_empty());
        if let Some(s_phi) = s2 {
            if s_phi != phi {
                //s2 != phi
                phi_ins.is_deleted = true;
                phi_ins.rhs = s_phi;
                //eventually simplify recursively: if a phi instruction is in phi use list, call simplify_phi() on it
                //but cse should deal with most of it.
            } else {
                //s2 == phi
                phi_ins.phi_arguments = result;
            }
        } else {
            //s2 is None
            phi_ins.is_deleted = true;
        }
    }
}

pub fn seal_block(ctx: &mut SsaContext, block_id: BlockId) {
    let block = &ctx[block_id];
    let pred = block.predecessor.clone();
    let instructions = block.instructions.clone();
    for i in instructions {
        if let Some(ins) = ctx.try_get_instruction(i) {
            let rhs = ins.rhs;
            if ins.operator == node::Operation::Phi {
                write_phi(ctx, &pred, rhs, i);
            }
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

fn get_array_size(array_size: &ArraySize) -> u32 {
    match array_size {
        ArraySize::Fixed(l) => *l as u32,
        ArraySize::Variable => todo!(),
    }
}

pub fn create_function_parameter(igen: &mut IRGenerator, ident_id: &DefinitionId) -> NodeId {
    let ident_name = igen.def_to_name(*ident_id);
    let o_type = igen.def_interner().id_type(*ident_id);
    //check if the variable is already created:
    if let Some(var) = igen.find_variable(*ident_id) {
        let id = var.unwrap_id(); //TODO handle multiple values
        return get_current_value(&mut igen.context, id);
    }
    let obj_type = node::ObjectType::from(&o_type);
    let obj = match o_type {
        noirc_frontend::Type::Array(_, len, _) => {
            let array_idx =
                igen.context
                    .mem
                    .create_new_array(get_array_size(&len), obj_type, &ident_name);
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type: node::ObjectType::Pointer(array_idx),
                root: None,
                def: Some(*ident_id),
                witness: None,
                parent_block: igen.context.current_block,
            }
        }
        _ => {
            //new variable - should be in a let statement? The let statement should set the type
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type,
                root: None,
                def: Some(*ident_id),
                witness: None,
                parent_block: igen.context.current_block,
            }
        }
    };
    let v_id = igen.context.add_variable(obj, None);
    igen.context
        .get_current_block_mut()
        .update_variable(v_id, v_id);
    v_id
}
