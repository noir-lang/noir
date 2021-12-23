use super::{node, code_gen::ParsingContext};
use arena::Index;
use super::code_gen;
use std::collections::HashMap;

// pub fn get_current_value(id: Index, value_array: HashMap<Index, node::NodeObj>) -> Index
// {
//     if value_array.contains_key(&id) {
//         return value_array[&id];
//     }
//     id
// }

pub fn unroll(join_id: Index, eval: &ParsingContext, mut max_id: u32)
{
    //We create a new block, but it must not be linked to the current one
    let join = eval.get_block(join_id).unwrap();
    let mut new_block = node::BasicBlock::new(join.left.unwrap());
    //TODO add block to the CFG (and set its id)

    //evaluate the join  block: 
    // evaluate_phi(predecessor); //for all phi instructions of the block
    //process_join(): we should execute all instructions will should evaluate to a constant (except for the last jump)
    while !evaluate_conditional_jump(*join.instructions.last().unwrap(), eval) {
        //Process body
        let body = eval.get_block(join.left.unwrap()).unwrap();
        for i_id in &body.instructions {
            let ins = eval.get_object(*i_id).unwrap(); 
            match ins {
                node::NodeObj::Instr(i) => {
                    let new_left = new_block.get_current_value(i.lhs);
                    let new_right = new_block.get_current_value(i.rhs);
                    let mut new_ins = node::Instruction::new(i.operator, new_left, new_right, i.res_type, Some(new_block.idx));
                    //instruction name for debugging:
                    new_ins.res_name = format!("%{}", max_id);
                    max_id += 1;
                    new_ins.lhs = new_left;
                    new_ins.rhs = new_right;
                    //TODO new_ins.simplify(new_left, new_right);
                    //TODO add_object(new_ins);
                    //new_block.update_variable(i_id, new_ins);  
                } ,
                _ => todo!(),
            }
        }
        //TODO how to handle nested if/loop inside a loop?
        //we should now indicate to the evaluate_phi that we come from body block.
    }

    //if true return new_list !
    //process body in a SSA fashion
    //loop back
    //That's it
}

fn evaluate_phi(from: Index) {
    //TODO:
    //For all phi instructions of the block:
    //retrieve the value from the phi instruction phi_ins:
    //for a in phi_ins.phi_arguments {
    // (n.b skip the first which gives v)
    // if a.1==from {
     //   update the value array of the block: value_array[v] = a
    //}
    //   else ??
    //} 
    //                                 
}

//returns true if we should jump
fn evaluate_conditional_jump(jump: Index, eval: &ParsingContext) -> bool
{
    //TODO il faut utiliser la VA du block
    let jump_ins = eval.get_as_instruction(jump).unwrap();
    let cond = eval.get_as_constant(jump_ins.lhs);
    if cond.is_some() {
        let result = !cond.unwrap().is_zero();
        match jump_ins.operator {
            node::Operation::jeq => return result,
            node::Operation::jne => return !result,
            node::Operation::jmp => return true,
            _ => panic!("loop without conditional statement!"),     //TODO shouldn't we return false instead?
        }
    }
    unreachable!("Condition should be constant");
    true
}



