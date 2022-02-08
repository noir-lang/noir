use super::{
    //block,
    code_gen::IRGenerator,
    node::{self, Node},
    optim,
};
use acvm::FieldElement;
use num_bigint::BigUint;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;

//Gets the maximum value of the instruction result
pub fn get_instruction_max(
    eval: &IRGenerator,
    ins: &node::Instruction,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> BigUint {
    let r_max = get_obj_max_value(eval, None, ins.rhs, max_map, vmap);
    let l_max = get_obj_max_value(eval, None, ins.lhs, max_map, vmap);
    get_instruction_max_operand(eval, ins, l_max,r_max, max_map, vmap)
}

//Gets the maximum value of the instruction result using the provided operand maximum
pub fn get_instruction_max_operand(
    eval: &IRGenerator,
    ins: &node::Instruction,
    left_max: BigUint,
    right_max: BigUint,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> BigUint {
    if ins.operator == node::Operation::load {
        return get_load_max(eval, ins.lhs, max_map, vmap);
    }
    ins.get_max_value(left_max, right_max)
}

// Retrieve max possible value of a node; from the max_map if it was already computed
// or else we compute it.
// we use the value array (get_current_value2) in order to handle truncate instructions
// we need to do it because rust did not allow to modify the instruction in block_overflow..
pub fn get_obj_max_value(
    eval: &IRGenerator,
    obj: Option<&node::NodeObj>,
    idx: arena::Index,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> BigUint {
    let id = get_value_from_map(idx, vmap); //block.get_current_value(idx);
    if max_map.contains_key(&id) {
        return max_map[&id].clone();
    }

    let obj_ = obj.unwrap_or_else(|| eval.get_object(id).unwrap());

    let result: BigUint;
    result = match obj_ {
        node::NodeObj::Obj(v) => {
            dbg!(v.bits());
            if v.bits() > 100 {
                dbg!(&v);
            }
            BigUint::from((1_u128 << v.bits()) - 1)
        } //TODO check for signed type
        node::NodeObj::Instr(i) => get_instruction_max(eval, i, max_map, vmap),
        node::NodeObj::Const(c) => c.value.clone(), //TODO panic for string constants
    };
    max_map.insert(id, result.clone());
    result
}

//Creates a truncate instruction for obj_id
pub fn truncate(
    eval: &mut IRGenerator,
    obj_id: arena::Index,
    bit_size: u32,
    max_map: &mut HashMap<arena::Index, BigUint>,
) -> Option<arena::Index> {
    // get type
    let obj = eval.get_object(obj_id).unwrap();
    let obj_type = obj.get_type();
    let obj_name = obj.print();
    //ensure truncate is needed:
    let v_max = &max_map[&obj_id];
    if *v_max >= BigUint::from(1_u128 << bit_size) {
        let rhs_bitsize = eval.new_constant(FieldElement::from(bit_size as i128)); //TODO is this leaking some info????
                                                                                   //Create a new truncate instruction '(idx): obj trunc bit_size'
                                                                                   //set current value of obj to idx
        let mut i =
            node::Instruction::new(node::Operation::trunc, obj_id, rhs_bitsize, obj_type, None);
        if i.res_name.ends_with("_t") {
            //TODO we should use %t so that we can check for this substring (% is not a valid char for a variable name) in the name and then write name%t[number+1]
        }
        i.res_name = obj_name + "_t";
        i.bit_size = v_max.bits() as u32;
        let i_id = eval.nodes.insert(node::NodeObj::Instr(i));
        max_map.insert(i_id, BigUint::from((1_u128 << bit_size) - 1));
        return Some(i_id);
        //we now need to call fix_truncate(), it is done in a separate function in order to not overwhelm the arguments list.
    }
    None
}

//Set the id and parent block of the truncate instruction
//This is needed because the instruction is inserted into a block and not added in the current block like regular instructions
//We also update the value array
pub fn fix_truncate(
    eval: &mut IRGenerator,
    idx: arena::Index,
    prev_id: arena::Index,
    block_idx: arena::Index,
    vmap: &mut HashMap<arena::Index, arena::Index>,
) {
    if let Some(ins) = eval.try_get_mut_instruction(idx) {
        ins.idx = idx;
        ins.parent_block = block_idx;
        vmap.insert(prev_id, idx);
    }
}

//Adds the variable to the list of variables that need to be truncated
fn add_to_truncate(
    eval: &IRGenerator,
    obj_id: arena::Index,
    bit_size: u32,
    to_truncate: &mut HashMap<arena::Index, u32>,
    max_map: &HashMap<arena::Index, BigUint>,
) -> BigUint {
    let v_max = &max_map[&obj_id];
    if *v_max >= BigUint::from(1_u128 << bit_size) {
        if let Some(node::NodeObj::Const(_)) = eval.get_object(obj_id) {
            return v_max.clone(); //a constant cannot be truncated, so we exit the function gracefully
        }
        let truncate_bits;
        if to_truncate.contains_key(&obj_id) {
            truncate_bits = u32::min(to_truncate[&obj_id], bit_size);
            to_truncate.insert(obj_id, truncate_bits);
        } else {
            to_truncate.insert(obj_id, bit_size);
            truncate_bits = bit_size;
        }
        return BigUint::from(truncate_bits - 1);
    }
    v_max.clone()
}

//Truncate the 'to_truncate' list
fn process_to_truncate(
    eval: &mut IRGenerator,
    new_list: &mut Vec<arena::Index>,
    to_truncate: &mut HashMap<arena::Index, u32>,
    max_map: &mut HashMap<arena::Index, BigUint>,
    block_idx: arena::Index,
    vmap: &mut HashMap<arena::Index, arena::Index>,
) {
    for (id, bit_size) in to_truncate.iter() {
        if let Some(truncate_idx) = truncate(eval, *id, *bit_size, max_map) {
            //TODO properly handle signed arithmetic...
            fix_truncate(eval, truncate_idx, *id, block_idx, vmap);
            new_list.push(truncate_idx);
        }
    }
    to_truncate.clear();
}

//Update right and left operands of the provided instruction
fn update_ins_parameters(
    eval: &mut IRGenerator,
    idx: arena::Index,
    lhs: arena::Index,
    rhs: arena::Index,
    max_value: Option<BigUint>,
) {
    let mut ins = eval.try_get_mut_instruction(idx).unwrap();
    ins.lhs = lhs;
    ins.rhs = rhs;
    if let Some(max_v) = max_value {
        ins.max_value = max_v;
    }
}

//Add required truncate instructions on all blocks
pub fn overflow_strategy(eval: &mut IRGenerator) {
    let mut max_map: HashMap<arena::Index, BigUint> = HashMap::new();
    tree_overflow(eval, eval.first_block, &mut max_map);
}

//implement overflow strategy following the dominator tree
pub fn tree_overflow(
    eval: &mut IRGenerator,
    b_idx: arena::Index,
    max_map: &mut HashMap<arena::Index, BigUint>,
) {
    block_overflow(eval, b_idx, max_map);
    let block = eval.get_block(b_idx);
    let bd = block.dominated.clone();
    //TODO: Handle IF statements in there:
    for b in bd {
        tree_overflow(eval, b, &mut max_map.clone());
    }
}

//overflow strategy for one block
//TODO - check the type; we MUST NOT truncate or overflow field elements!!
pub fn block_overflow(
    eval: &mut IRGenerator,
    b_idx: arena::Index,
    //block: &mut node::BasicBlock,
    max_map: &mut HashMap<arena::Index, BigUint>,
) {
    //for each instruction, we compute the resulting max possible value (in term of the field representation of the operation)
    //when it is over the field charac, or if the instruction requires it, then we insert truncate instructions
    // The instructions are insterted in a duplicate list( because of rust ownership..), which we use for
    // processing another cse round for the block because the truncates may have added duplicate.
    let block = eval.blocks.get(b_idx).unwrap();
    let mut b: Vec<node::Instruction> = Vec::new();
    let mut new_list: Vec<arena::Index> = Vec::new();
    let mut truncate_map: HashMap<arena::Index, u32> = HashMap::new();
    //RIA...
    for iter in &block.instructions {
        b.push((*eval.try_get_instruction(*iter).unwrap()).clone());
    }
    let mut value_map: HashMap<arena::Index, arena::Index> = HashMap::new(); //since we process the block from the start, the block value array is not relevant
                                                                             //block.value_array.clone();     //RIA - we need to modify it and to use it
                                                                             //TODO we should try to make another simplify round here, or at least after copy propagation, we should do it at the best convenient place....TODO
    let mut memory_map: HashMap<u32, arena::Index> = HashMap::new();    //TODO surement a mettre en argument
    let mut delete_ins = false;
    for mut ins in b {
        if ins.operator == node::Operation::nop {
            continue;
        }
        let mut i_lhs = ins.lhs;
        let mut i_rhs = ins.rhs;
        //we propagate optimised loads - todo check if it is needed because there is cse at the end
        if node::is_binary(ins.operator) {
            //binary operation:
            i_lhs = super::optim::propagate(eval, ins.lhs);
            i_rhs = super::optim::propagate(eval, ins.rhs);
        }
        //We retrieve get_current_value() in case a previous truncate has updated the value map
        let r_id = get_value_from_map(i_rhs, &value_map); 
        let mut update_instruction = false;
        if r_id != ins.rhs {
            ins.rhs = r_id;
            update_instruction = true;
        }
        let l_id = get_value_from_map(i_lhs, &value_map);
        if l_id != ins.lhs {
            ins.lhs = l_id;
            update_instruction = true;
        }

        let r_obj = eval.get_object(r_id).unwrap();
        let l_obj = eval.get_object(l_id).unwrap();
        let r_max = get_obj_max_value(eval, Some(r_obj), r_id, max_map, &value_map);
        get_obj_max_value(eval, Some(l_obj), l_id, max_map, &value_map);
        //insert required truncates
        let to_truncate = ins.truncate_required(l_obj.bits(), r_obj.bits());
        if to_truncate.0 {
            //adds a new truncate(lhs) instruction
            add_to_truncate(eval, l_id, l_obj.bits(), &mut truncate_map, max_map);
        }
        if to_truncate.1 {
            //adds a new truncate(rhs) instruction
            add_to_truncate(eval, r_id, r_obj.bits(), &mut truncate_map, max_map);
        }
        
        if ins.operator == node::Operation::load {
            //TODO we use a local memory map for now but it should be used in arguments
            //for instance, the join block of a IF should merge the two memorymaps using the condition value
            if let Some(adr) = super::mem::Memory::to_u32(eval, ins.lhs) {
                if let Some(val) = memory_map.get(&adr) {
                    //optimise static load
                    ins.is_deleted = true;
                    ins.rhs = *val;
                }
            }
        }
        if ins.operator == node::Operation::store {
            if let Some(adr) = super::mem::Memory::to_u32(eval, ins.lhs) {
                //optimise static store
                memory_map.insert(adr, ins.lhs);
                delete_ins = true;
            }
        }
        if ins.operator == node::Operation::cast {
            //TODO for cast, we may need to reduce rhs into the bit size of lhs
            //this can change the max value of the cast so its need to be done here
            //(or we update the get_max_bits() for casts)
            let lhs_bits = l_obj.bits();
            if r_max.bits() as u32 > lhs_bits {
                add_to_truncate(eval, r_id, l_obj.bits(), &mut truncate_map, max_map);
            }
        }
        let mut ins_max = get_instruction_max(eval, &ins, max_map, &value_map);
        if ins_max.bits() >= (FieldElement::max_num_bits() as u64) {
            //let's truncate a and b:
            //- insert truncate(lhs) dans la list des instructions
            //- insert truncate(rhs) dans la list des instructions
            //- update r_max et l_max
            //n.b we could try to truncate only one of them, but then we should check if rhs==lhs.
            let l_trunc_max = add_to_truncate(eval, l_id, l_obj.bits(), &mut truncate_map, max_map);
            let r_trunc_max = add_to_truncate(eval, r_id, r_obj.bits(), &mut truncate_map, max_map);
            ins_max = get_instruction_max_operand(eval, &ins, l_trunc_max.clone(), r_trunc_max.clone(), max_map, &value_map);
            //ins_max = ins.get_max_value(l_trunc_max.clone(), r_trunc_max.clone());
            if ins_max.bits() >= FieldElement::max_num_bits().into() {
                let message = format!(
                    "Require big int implementation, the bit size is too big for the field: {}, {}",
                    l_trunc_max.bits(),
                    r_trunc_max.bits()
                );
                panic!("{}", message);
            }
        }
        process_to_truncate(
            eval,
            &mut new_list,
            &mut truncate_map,
            max_map,
            b_idx,
            &mut value_map,
        );
        if delete_ins {
            delete_ins = false;
        }
        else {
            new_list.push(ins.idx);   
            let l_new = get_value_from_map(l_id, &value_map);
            let r_new = get_value_from_map(r_id, &value_map); 
            if l_new != l_id || r_new != r_id || ins.is_sub() {
                update_instruction = true;
            }
            if update_instruction {
                let mut max_r_value = None;
                if ins.is_sub() {
                    max_r_value = Some(max_map[&r_new].clone()); //for now we pass the max value to the instruction, we could also keep the max_map e.g in the block (or max in each nodeobj)
                                                                //we may do that in future when the max_map becomes more used elsewhere (for other optim)
                }
                update_ins_parameters(eval, ins.idx, l_new, r_new, max_r_value);
            }
        }
    }
    update_value_array(eval, b_idx, &value_map);
    let mut anchor: HashMap<node::Operation, VecDeque<arena::Index>> = HashMap::new();
    //We run another round of CSE for the block in order to remove possible duplicated truncates, this will assign 'new_list' to the block instructions
    optim::block_cse(eval, b_idx, &mut anchor, &mut new_list);
}

fn update_value_array(
    eval: &mut IRGenerator,
    b_id: arena::Index,
    vmap: &HashMap<arena::Index, arena::Index>,
) {
    let block = eval.get_block_mut(b_id).unwrap();
    for (old, new) in vmap {
        block.value_map.insert(*old, *new); //TODO we must merge rather than update
    }
}

//Get current value using the provided vmap
pub fn get_value_from_map(
    idx: arena::Index,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> arena::Index {
    match vmap.get(&idx) {
        Some(cur_idx) => *cur_idx,
        None => idx,
    }
}


pub fn get_load_max(eval: &IRGenerator, address: arena::Index,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> BigUint {
    if let Some(adr_as_const) = eval.get_as_constant(address) {
        let adr : u32 = adr_as_const.to_u128().try_into().unwrap();
        if let Some(&value) = eval.mem.memory_map.get(&adr) {
            return get_obj_max_value(eval, None, value, max_map, vmap);
        }
        return eval.mem.get_array_from_adr(adr).max.clone();
    };
    todo!(); //todo return array max
    
}
