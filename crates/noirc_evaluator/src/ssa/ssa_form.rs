use super::{code_gen::IRGenerator, node};
use crate::{environment::Environment, object::Object};
use arena;

// create phi arguments from the predecessors of the block (containing phi)
pub fn write_phi(
    igen: &mut IRGenerator,
    predecessors: &[arena::Index],
    var: arena::Index,
    phi: arena::Index,
) {
    let mut result: Vec<(arena::Index, arena::Index)> = Vec::new();
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

pub fn seal_block(igen: &mut IRGenerator, block_id: arena::Index) {
    let block = igen.get_block(block_id);
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
pub fn get_block_value(
    igen: &mut IRGenerator,
    root: arena::Index,
    block_id: arena::Index,
) -> arena::Index {
    let result;
    if !igen.sealed_blocks.contains(&block_id) {
        //incomplete CFG
        result = igen.generate_empty_phi(block_id, root);
    } else {
        let block = igen.get_block(block_id);
        if let Some(idx) = block.get_current_value(root) {
            return idx;
        }
        let pred = block.predecessor.clone();
        if pred.is_empty() {
            return root;
        }
        if pred.len() == 1 {
            result = get_block_value(igen, root, pred[0]);
        } else {
            result = igen.generate_empty_phi(block_id, root);
            write_phi(igen, &pred, root, result);
        }
    }
    let block = igen.get_block_mut(block_id).unwrap();
    block.update_variable(root, result);
    result
}

//Returns the current SSA value of a variable in a (filled) block.
pub fn get_current_value_in_block(
    igen: &mut IRGenerator,
    var_id: arena::Index,
    block_id: arena::Index,
) -> arena::Index {
    //1. get root variable
    let mut root = var_id;
    if let Ok(var) = igen.get_variable(var_id) {
        if let Some(var_root) = var.root {
            root = var_root;
        }
    }
    //Local value numbering
    let block = igen.get_block(block_id);
    if let Some(val) = block.get_current_value(root) {
        return val;
    }
    //Global value numbering
    get_block_value(igen, root, block_id)
}

//Returns the current SSA value of a variable, recursively
pub fn get_current_value(igen: &mut IRGenerator, var_id: arena::Index) -> arena::Index {
    get_current_value_in_block(igen, var_id, igen.current_block)
}

pub fn evaluate_identifier(
    igen: &mut IRGenerator,
    env: &mut Environment,
    ident_id: &noirc_frontend::node_interner::IdentId,
) -> arena::Index {
    let ident_name = igen.context.unwrap().def_interner.ident_name(ident_id);
    let ident_def = igen.context.unwrap().def_interner.ident_def(ident_id);
    let o_type = igen.context().def_interner.id_type(ident_def.unwrap());
    //check if the variable is already created:
    if let Some(var) = igen.find_variable(&ident_def) {
        let id = var.id;
        return get_current_value(igen, id);
    }
    let obj = env.get(&ident_name);
    dbg!(&o_type);
    let obj = match obj {
        Object::Array(a) => {
            let obj_type = node::ObjectType::from_type(o_type);
            //We should create an array from 'a' witnesses
            igen.mem
                .create_array(&a, ident_def.unwrap(), obj_type, &ident_name);
            let array_index = (igen.mem.arrays.len() - 1) as u32;
            node::Variable {
                id: igen.id0,
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
                id: igen.id0,
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
    igen.get_block_mut(igen.current_block)
        .unwrap()
        .update_variable(v_id, v_id);
    v_id
}
