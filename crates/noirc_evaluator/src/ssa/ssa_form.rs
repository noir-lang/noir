use super::{code_gen::IRGenerator, node};
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
    if let Some(phi_ins) = igen.get_as_mut_instruction(phi) {
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
    let block = igen.get_block(block_id).unwrap();
    let pred = block.predecessor.clone();
    let instructions = block.instructions.clone();
    for i in instructions {
        if let Some(ins) = igen.get_as_instruction(i) {
            if ins.operator == node::Operation::phi {
                write_phi(igen, &pred, ins.rhs, i);
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
        let block = igen.get_block(block_id).unwrap();
        if let Some(idx) = block.get_current_value(root) {
            return idx;
        }
        let pred = block.predecessor.clone();
        if pred.is_empty() {
            return root;
        }
        if pred.len() == 1 {
            result = get_block_value(igen, root, block.predecessor[0]);
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
    let block = igen.get_block(block_id).unwrap();
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
