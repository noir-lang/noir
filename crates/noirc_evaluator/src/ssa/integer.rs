use super::{
    //block,
    code_gen::IRGenerator,
    node::{self, Instruction, Node, Operation},
    optim,
};
use acvm::FieldElement;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;

//Returns the maximum bit size of short integers
pub fn short_integer_max_bit_size() -> u32 {
    //TODO: it should be FieldElement::max_num_bits()/2, but for now we do not support more than 128 bits as well
    //This allows us to do use u128 to represent integer constant values
    u32::min(FieldElement::max_num_bits() / 2, 128)
}

//Gets the maximum value of the instruction result
pub fn get_instruction_max(
    eval: &IRGenerator,
    ins: &node::Instruction,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> BigUint {
    let r_max = get_obj_max_value(eval, None, ins.rhs, max_map, vmap);
    let l_max = get_obj_max_value(eval, None, ins.lhs, max_map, vmap);
    get_instruction_max_operand(eval, ins, l_max, r_max, max_map, vmap)
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
    match ins.operator {
        node::Operation::Load(array) => get_load_max(eval, ins.lhs, max_map, vmap, array),
        node::Operation::EqGate => {
            //TODO... we should update the max_map AFTER the truncate is processed (else it breaks it)
            // let min = BigUint::min(left_max.clone(), right_max.clone());
            // max_map.insert(ins.lhs, min.clone());
            // max_map.insert(ins.rhs, min);
            get_max_value(ins, left_max, right_max)
        }
        _ => get_max_value(ins, left_max, right_max),
    }
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
    dbg!(&id);
    if max_map.contains_key(&id) {
        return max_map[&id].clone();
    }

    let obj_ = obj.unwrap_or_else(|| eval.get_object(id).unwrap());
    dbg!(&obj_);
    let result: BigUint;
    result = match obj_ {
        node::NodeObj::Obj(v) => {
            if v.bits() > 100 {
                dbg!(&v);
            }
            (BigUint::one() << v.bits()) - BigUint::one()
        } //TODO check for signed type
        node::NodeObj::Instr(i) => get_instruction_max(eval, i, max_map, vmap),
        node::NodeObj::Const(c) => c.value.clone(), //TODO panic for string constants
    };
    max_map.insert(id, result.clone());
    dbg!(&max_map);
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
    let obj_name = format!("{}", obj);
    //ensure truncate is needed:
    let v_max = &max_map[&obj_id];
    if *v_max >= BigUint::one() << bit_size {
        //TODO is this leaking some info????
        let rhs_bitsize = eval.new_constant(FieldElement::from(bit_size as i128));
        //Create a new truncate instruction '(idx): obj trunc bit_size'
        //set current value of obj to idx
        let mut i =
            node::Instruction::new(node::Operation::Trunc, obj_id, rhs_bitsize, obj_type, None);
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
    if *v_max >= BigUint::one() << bit_size {
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

fn update_ins(eval: &mut IRGenerator, idx: arena::Index, copy_from: &node::Instruction) {
    let mut ins = eval.try_get_mut_instruction(idx).unwrap();
    ins.lhs = copy_from.lhs;
    ins.rhs = copy_from.rhs;
    ins.operator = copy_from.operator;
    ins.max_value = copy_from.max_value.clone();
    ins.bit_size = copy_from.bit_size;
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
pub fn block_overflow(
    eval: &mut IRGenerator,
    b_idx: arena::Index,
    //block: &mut node::BasicBlock,
    max_map: &mut HashMap<arena::Index, BigUint>,
) {
    //for each instruction, we compute the resulting max possible value (in term of the field representation of the operation)
    //when it is over the field charac, or if the instruction requires it, then we insert truncate instructions
    // The instructions are insterted in a duplicate list( because of rust ownership..), which we use for
    // processing another cse round for the block because the truncates may be duplicated.
    let block = eval.blocks.get(b_idx).unwrap();
    let mut b: Vec<node::Instruction> = Vec::new();
    let mut new_list: Vec<arena::Index> = Vec::new();
    let mut truncate_map: HashMap<arena::Index, u32> = HashMap::new();
    let mut modify_ins: Option<Instruction> = None;
    let mut trunc_size = FieldElement::zero();
    //RIA...
    for iter in &block.instructions {
        b.push((*eval.try_get_instruction(*iter).unwrap()).clone());
    }
    //since we process the block from the start, the block value array is not relevant
    let mut value_map: HashMap<arena::Index, arena::Index> = HashMap::new();
    let mut memory_map: HashMap<u32, arena::Index> = HashMap::new(); //TODO put in argument
    let mut delete_ins = false;
    for mut ins in b {
        if ins.operator == node::Operation::Nop {
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
        let l_max = get_obj_max_value(eval, Some(l_obj), l_id, max_map, &value_map);
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
        match ins.operator {
            node::Operation::Load(_) => {
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
            node::Operation::Store(_) => {
                if let Some(adr) = super::mem::Memory::to_u32(eval, ins.lhs) {
                    //optimise static store
                    memory_map.insert(adr, ins.lhs);
                    delete_ins = true;
                }
            }
            node::Operation::Cast => {
                //TODO for now the types we support here are only all integer types (field, signed, unsigned, bool)
                //so a cast would normally translate to a truncate.
                //if res_type and lhs have the same bit size (in a large sens, which include field elements)
                //then either they have the same type and should have been simplified
                //or they don't have the same sign so we keep the cast operator
                //if res_type is smaller than lhs bit size, we look if lhs can hold directly into res_type
                // if not, we need to truncate lhs to a res_type. We modify directly the cast instruction into a truncate
                // in other cases we can keep the cast instruction
                // for instance if res_type is greater than lhs bit size, we need to truncate lhs to its bit size and use the truncate
                // result in the cast, but this is handled by the truncate_required
                // after this function, all cast instructions refer to casting lhs into a bigger (or equal) type
                // anyother case has been transformed into the latter using truncates.
                if ins.res_type == l_obj.get_type() {
                    ins.is_deleted = true;
                    ins.rhs = ins.lhs;
                }
                if ins.res_type.bits() < l_obj.bits() && r_max.bits() as u32 > ins.res_type.bits() {
                    //we need to truncate
                    update_instruction = true;
                    trunc_size = FieldElement::from(ins.res_type.bits() as i128);
                    let mut mod_ins = Instruction::new(
                        node::Operation::Trunc,
                        l_id,
                        l_id,
                        ins.res_type,
                        Some(ins.parent_block),
                    );
                    mod_ins.bit_size = l_max.bits() as u32;
                    modify_ins = Some(mod_ins);
                    //TODO name for the instruction: modify_ins.res_name = l_obj."name"+"_t";
                    //n.b. we do not update value map because we re-use the cast instruction
                }
            }
            _ => (),
        }
        let mut ins_max = get_instruction_max(eval, &ins, max_map, &value_map);
        if ins_max.bits() >= (FieldElement::max_num_bits() as u64)
            && ins.res_type != node::ObjectType::NativeField
        {
            //let's truncate a and b:
            //- insert truncate(lhs) dans la list des instructions
            //- insert truncate(rhs) dans la list des instructions
            //- update r_max et l_max
            //n.b we could try to truncate only one of them, but then we should check if rhs==lhs.
            let l_trunc_max = add_to_truncate(eval, l_id, l_obj.bits(), &mut truncate_map, max_map);
            let r_trunc_max = add_to_truncate(eval, r_id, r_obj.bits(), &mut truncate_map, max_map);
            ins_max = get_instruction_max_operand(
                eval,
                &ins,
                l_trunc_max.clone(),
                r_trunc_max.clone(),
                max_map,
                &value_map,
            );
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
        } else {
            new_list.push(ins.idx);
            let l_new = get_value_from_map(l_id, &value_map);
            let r_new = get_value_from_map(r_id, &value_map);
            if l_new != l_id || r_new != r_id || is_sub(&ins.operator) {
                update_instruction = true;
            }
            if update_instruction {
                let mut max_r_value = None;
                if is_sub(&ins.operator) {
                    //for now we pass the max value to the instruction, we could also keep the max_map e.g in the block (or max in each nodeobj)
                    //sub operations require the max value to ensure it does not underflow
                    max_r_value = Some(max_map[&r_new].clone());
                    //we may do that in future when the max_map becomes more used elsewhere (for other optim)
                }
                if let Some(modified_ins) = &mut modify_ins {
                    modified_ins.rhs = eval.get_const(trunc_size, node::ObjectType::Unsigned(32));
                    modified_ins.lhs = l_new;
                    if let Some(max_v) = max_r_value {
                        modified_ins.max_value = max_v;
                    }
                    update_ins(eval, ins.idx, modified_ins);
                } else {
                    update_ins_parameters(eval, ins.idx, l_new, r_new, max_r_value);
                }
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

pub fn get_load_max(
    eval: &IRGenerator,
    address: arena::Index,
    max_map: &mut HashMap<arena::Index, BigUint>,
    vmap: &HashMap<arena::Index, arena::Index>,
    array: u32,
    // obj_type: node::ObjectType,
) -> BigUint {
    if let Some(adr_as_const) = eval.get_as_constant(address) {
        let adr: u32 = adr_as_const.to_u128().try_into().unwrap();
        if let Some(&value) = eval.mem.memory_map.get(&adr) {
            return get_obj_max_value(eval, None, value, max_map, vmap);
        }
    };
    eval.mem.arrays[array as usize].max.clone() //return array max
                                                //  return obj_type.max_size();
}

//Returns the max value of an operation from an upper bound of left and right hand sides
//Function is used to check for overflows over the field size, this is why we use BigUint.
pub fn get_max_value(ins: &Instruction, lhs_max: BigUint, rhs_max: BigUint) -> BigUint {
    match ins.operator {
        Operation::Add => lhs_max + rhs_max,
        Operation::SafeAdd => todo!(),
        Operation::Sub => {
            let r_mod = BigUint::one() << ins.res_type.bits();
            let mut k = &rhs_max / &r_mod;
            if &rhs_max % &r_mod != BigUint::zero() {
                k += BigUint::one();
            }
            assert!(&k * &r_mod >= rhs_max);
            lhs_max + k * r_mod
        }
        Operation::SafeSub => todo!(),
        Operation::Mul => lhs_max * rhs_max,
        Operation::SafeMul => todo!(),
        Operation::Udiv => lhs_max,
        Operation::Sdiv => todo!(),
        Operation::Urem => rhs_max - BigUint::one(),
        Operation::Srem => todo!(),
        Operation::Div => todo!(),
        Operation::Eq => BigUint::one(),
        Operation::Ne => BigUint::one(),
        Operation::Ugt => BigUint::one(),
        Operation::Uge => BigUint::one(),
        Operation::Ult => BigUint::one(),
        Operation::Ule => BigUint::one(),
        Operation::Sgt => BigUint::one(),
        Operation::Sge => BigUint::one(),
        Operation::Slt => BigUint::one(),
        Operation::Sle => BigUint::one(),
        Operation::Lt => BigUint::one(),
        Operation::Gt => BigUint::one(),
        Operation::Lte => BigUint::one(),
        Operation::Gte => BigUint::one(),
        Operation::And => ins.res_type.max_size(),
        Operation::Not => ins.res_type.max_size(),
        Operation::Or => ins.res_type.max_size(),
        Operation::Xor => ins.res_type.max_size(),
        //'a cast a' means we cast a into res_type of the instruction
        Operation::Cast => {
            let type_max = ins.res_type.max_size();
            BigUint::min(lhs_max, type_max)
        }
        Operation::Trunc => BigUint::min(
            lhs_max,
            BigUint::from(2_u32).pow(rhs_max.try_into().unwrap()) - BigUint::from(1_u32),
        ),
        //'a = b': a and b must be of same type.
        Operation::Ass => rhs_max,
        Operation::Nop | Operation::Jne | Operation::Jeq | Operation::Jmp => todo!(),
        Operation::Phi => BigUint::max(lhs_max, rhs_max), //TODO operands are in phi_arguments, not lhs/rhs!!
        Operation::EqGate => BigUint::zero(),             //min(lhs_max, rhs_max),
        Operation::Load(_) => {
            unreachable!();
        }
        Operation::Store(_) => BigUint::from(0_u32),
        Operation::StdLib(opcode) => todo!(),
    }
}

//indicates if the operation is a substraction, we need to check them for underflow
pub fn is_sub(operator: &Operation) -> bool {
    matches!(operator, Operation::Sub | Operation::SafeSub)
}
