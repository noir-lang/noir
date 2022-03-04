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
