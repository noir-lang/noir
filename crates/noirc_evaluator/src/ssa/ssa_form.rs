use noirc_frontend::ArraySize;

use crate::{environment::Environment, object::Object};

use super::{
    block::BlockId,
    code_gen::IRGenerator,
    node::{self, NodeId},
};

// create phi arguments from the predecessors of the block (containing phi)
pub fn write_phi(igen: &mut IRGenerator, predecessors: &[BlockId], var: NodeId, phi: NodeId) {
    let mut result = Vec::new();
    for b in predecessors {
        let v = get_current_value_in_block(igen, var, *b);
        result.push((v, *b));
    }

    let s2 = node::Instruction::simplify_phi(phi, &result);
    if let Some(phi_ins) = igen.try_get_mut_instruction(phi) {
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

pub fn seal_block(igen: &mut IRGenerator, block_id: BlockId) {
    let block = &igen[block_id];
    let pred = block.predecessor.clone();
    let instructions = block.instructions.clone();
    for i in instructions {
        if let Some(ins) = igen.try_get_instruction(i) {
            let rhs = ins.rhs;
            if ins.operator == node::Operation::Phi {
                write_phi(igen, &pred, rhs, i);
            }
        }
    }
    igen.sealed_blocks.insert(block_id);
}

//look-up recursiverly into predecessors
pub fn get_block_value(igen: &mut IRGenerator, root: NodeId, block_id: BlockId) -> NodeId {
    let result = if !igen.sealed_blocks.contains(&block_id) {
        //incomplete CFG
        igen.generate_empty_phi(block_id, root)
    } else {
        let block = &igen[block_id];
        if let Some(idx) = block.get_current_value(root) {
            return idx;
        }
        let pred = block.predecessor.clone();
        if pred.is_empty() {
            return root;
        }
        if pred.len() == 1 {
            get_block_value(igen, root, pred[0])
        } else {
            let result = igen.generate_empty_phi(block_id, root);
            write_phi(igen, &pred, root, result);
            result
        }
    };

    igen[block_id].update_variable(root, result);
    result
}

//Returns the current SSA value of a variable in a (filled) block.
pub fn get_current_value_in_block(
    igen: &mut IRGenerator,
    var_id: NodeId,
    block_id: BlockId,
) -> NodeId {
    let root = igen.get_root_value(var_id);

    igen[block_id]
        .get_current_value(root) //Local value numbering
        .unwrap_or_else(|| get_block_value(igen, root, block_id)) //Global value numbering
}

//Returns the current SSA value of a variable, recursively
pub fn get_current_value(igen: &mut IRGenerator, var_id: NodeId) -> NodeId {
    get_current_value_in_block(igen, var_id, igen.current_block)
}

pub fn evaluate_identifier(
    igen: &mut IRGenerator,
    env: &mut Environment,
    ident_id: &noirc_frontend::node_interner::IdentId,
) -> NodeId {
    let ident_name = igen.context().def_interner.ident_name(ident_id);
    let ident_def = igen.context().def_interner.ident_def(ident_id);
    let o_type = igen.context().def_interner.id_type(ident_def.unwrap());
    //check if the variable is already created:
    if let Some(var) = igen.find_variable(&ident_def) {
        let id = var.id;
        return get_current_value(igen, id);
    }
    let obj = env.get(&ident_name);
    let obj = match obj {
        Object::Array(a) => {
            let obj_type = node::ObjectType::from_type(&o_type);
            //We should create an array from 'a' witnesses
            igen.mem
                .create_array_from_object(&a, ident_def.unwrap(), obj_type, &ident_name);
            let array_index = (igen.mem.arrays.len() - 1) as u32;
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type: node::ObjectType::Pointer(array_index),
                root: None,
                def: ident_def,
                witness: None,
                parent_block: igen.current_block,
            }
        }
        _ => {
            let obj_type = node::ObjectType::get_type_from_object(&obj);
            //new variable - should be in a let statement? The let statement should set the type
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type,
                root: None,
                def: ident_def,
                witness: node::get_witness_from_object(&obj),
                parent_block: igen.current_block,
            }
        }
    };

    let v_id = igen.add_variable(obj, None);
    igen.get_current_block_mut().update_variable(v_id, v_id);
    v_id
}

fn get_array_size(array_size: &ArraySize) -> u32 {
    match array_size {
        ArraySize::Fixed(l) => *l as u32,
        ArraySize::Variable => todo!(),
    }
}

pub fn create_function_parameter(
    igen: &mut IRGenerator,
    ident_id: &noirc_frontend::node_interner::IdentId,
) -> NodeId {
    let ident_name = igen.context().def_interner.ident_name(ident_id);
    let ident_def = igen.context().def_interner.ident_def(ident_id);
    let o_type = igen.context().def_interner.id_type(ident_def.unwrap());
    //check if the variable is already created:
    if let Some(var) = igen.find_variable(&ident_def) {
        let id = var.id;
        return get_current_value(igen, id);
    }
    let obj_type = node::ObjectType::from_type(&o_type);
    let obj = match o_type {
        noirc_frontend::Type::Array(_, len, _) => {
            let array_idx = igen
                .mem
                .create_new_array(get_array_size(&len), obj_type, &ident_name);
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type: node::ObjectType::Pointer(array_idx),
                root: None,
                def: ident_def,
                witness: None,
                parent_block: igen.current_block,
            }
        }
        _ => {
            //new variable - should be in a let statement? The let statement should set the type
            node::Variable {
                id: NodeId::dummy(),
                name: ident_name.clone(),
                obj_type,
                root: None,
                def: ident_def,
                witness: None,
                parent_block: igen.current_block,
            }
        }
    };
    let v_id = igen.add_variable(obj, None);
    igen.get_current_block_mut().update_variable(v_id, v_id);
    v_id
}
