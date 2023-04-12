use crate::errors::RuntimeError;
use crate::ssa::{
    block::BlockId,
    context::SsaContext,
    mem::{ArrayId, Memory},
    node::{BinaryOp, Instruction, Mark, Node, NodeId, NodeObject, ObjectType, Operation},
    {node, optimizations},
};
use acvm::FieldElement;
use iter_extended::vecmap;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Neg,
};

//Returns the maximum bit size of short integers
pub(super) fn short_integer_max_bit_size() -> u32 {
    //TODO: it should be FieldElement::max_num_bits()/2, but for now we do not support more than 128 bits as well
    //This allows us to do use u128 to represent integer constant values
    u32::min(FieldElement::max_num_bits() / 2, 128)
}

//Gets the maximum value of the instruction result
fn get_instruction_max(
    ctx: &SsaContext,
    ins: &Instruction,
    max_map: &mut HashMap<NodeId, BigUint>,
    value_map: &HashMap<NodeId, NodeId>,
) -> BigUint {
    assert_ne!(
        ins.operation.opcode(),
        node::Opcode::Phi,
        "Phi instructions must have been simplified"
    );
    ins.operation.for_each_id(|id| {
        get_obj_max_value(ctx, id, max_map, value_map);
    });
    get_instruction_max_operand(ctx, ins, max_map, value_map)
}

//Gets the maximum value of the instruction result using the provided operand maximum
fn get_instruction_max_operand(
    ctx: &SsaContext,
    ins: &Instruction,
    max_map: &mut HashMap<NodeId, BigUint>,
    value_map: &HashMap<NodeId, NodeId>,
) -> BigUint {
    match &ins.operation {
        Operation::Load { array_id, index, .. } => {
            get_load_max(ctx, *index, max_map, value_map, *array_id)
        }
        Operation::Binary(node::Binary { operator, lhs, rhs, .. }) => {
            if let BinaryOp::Sub { .. } = operator {
                //TODO uses interval analysis instead
                // Note that a boolean is also handled as an unsigned integer
                if ins.res_type.is_unsigned_integer() {
                    if let Some(lhs_const) = ctx.get_as_constant(*lhs) {
                        let lhs_big = BigUint::from_bytes_be(&lhs_const.to_be_bytes());
                        if max_map[rhs] <= lhs_big {
                            //TODO unsigned
                            return lhs_big;
                        }
                    }
                }
            }
            get_max_value(ins, max_map)
        }
        // Operation::Constrain(_) => {
        //ConstrainOp::Eq :
        //TODO... we should update the max_map AFTER the truncate is processed (else it breaks it)
        // let min = BigUint::min(left_max.clone(), right_max.clone());
        // max_map.insert(ins.lhs, min.clone());
        // max_map.insert(ins.rhs, min);
        _ => get_max_value(ins, max_map),
    }
}

// Retrieve max possible value of a node; from the max_map if it was already computed
// or else we compute it.
// we use the value array (get_current_value2) in order to handle truncate instructions
// we need to do it because rust did not allow to modify the instruction in block_overflow..
fn get_obj_max_value(
    ctx: &SsaContext,
    id: NodeId,
    max_map: &mut HashMap<NodeId, BigUint>,
    value_map: &HashMap<NodeId, NodeId>,
) -> BigUint {
    let id = get_value_from_map(id, value_map);
    if max_map.contains_key(&id) {
        return max_map[&id].clone();
    }
    if id == NodeId::dummy() {
        max_map.insert(id, BigUint::zero());
        return BigUint::zero(); //a non-argument has no max
    }
    let obj = &ctx[id];

    let result = match obj {
        NodeObject::Variable(v) => (BigUint::one() << v.size_in_bits()) - BigUint::one(), //TODO check for signed type
        NodeObject::Instr(i) => get_instruction_max(ctx, i, max_map, value_map),
        NodeObject::Const(c) => c.value.clone(), //TODO panic for string constants
        NodeObject::Function(..) => BigUint::zero(),
    };
    max_map.insert(id, result.clone());
    result
}

//Creates a truncate instruction for obj_id
fn truncate(
    ctx: &mut SsaContext,
    obj_id: NodeId,
    bit_size: u32,
    max_map: &mut HashMap<NodeId, BigUint>,
) -> Option<NodeId> {
    // get type
    let obj = &ctx[obj_id];
    let obj_type = obj.get_type();
    let obj_name = format!("{obj}");
    //ensure truncate is needed:
    let v_max = &max_map[&obj_id];

    if *v_max >= BigUint::one() << bit_size {
        //TODO is max_bit_size leaking some info????
        //Create a new truncate instruction '(idx): obj truncate bit_size'
        //set current value of obj to idx
        let max_bit_size = v_max.bits() as u32;

        let mut i = Instruction::new(
            Operation::Truncate { value: obj_id, bit_size, max_bit_size },
            obj_type,
            None,
        );

        if i.res_name.ends_with("_t") {
            //TODO we should use %t so that we can check for this substring (% is not a valid char for a variable name) in the name and then write name%t[number+1]
        }
        i.res_name = obj_name + "_t";
        let i_id = ctx.add_instruction(i);
        max_map.insert(i_id, BigUint::from((1_u128 << bit_size) - 1));
        Some(i_id)
        //we now need to call fix_truncate(), it is done in a separate function in order to not overwhelm the arguments list.
    } else {
        None
    }
}

//Set the id and parent block of the truncate instruction
//This is needed because the instruction is inserted into a block and not added in the current block like regular instructions
//We also update the value array
fn fix_truncate(
    eval: &mut SsaContext,
    id: NodeId,
    prev_id: NodeId,
    block_idx: BlockId,
    value_map: &mut HashMap<NodeId, NodeId>,
) {
    if let Some(ins) = eval.try_get_mut_instruction(id) {
        ins.parent_block = block_idx;
        value_map.insert(prev_id, id);
    }
}

//Adds the variable to the list of variables that need to be truncated
fn add_to_truncate(
    ctx: &SsaContext,
    obj_id: NodeId,
    bit_size: u32,
    to_truncate: &mut BTreeMap<NodeId, u32>,
    max_map: &HashMap<NodeId, BigUint>,
) {
    let v_max = &max_map[&obj_id];
    if *v_max >= BigUint::one() << bit_size {
        if let Some(NodeObject::Const(_)) = &ctx.try_get_node(obj_id) {
            return; //a constant cannot be truncated, so we exit the function gracefully
        }
        let truncate_bits = match to_truncate.get(&obj_id) {
            Some(value) => u32::min(*value, bit_size),
            None => bit_size,
        };
        to_truncate.insert(obj_id, truncate_bits);
    }
}

//Truncate the 'to_truncate' list
fn process_to_truncate(
    ctx: &mut SsaContext,
    new_list: &mut Vec<NodeId>,
    to_truncate: &mut BTreeMap<NodeId, u32>,
    max_map: &mut HashMap<NodeId, BigUint>,
    block_idx: BlockId,
    value_map: &mut HashMap<NodeId, NodeId>,
) {
    for (id, bit_size) in to_truncate.iter() {
        if let Some(truncate_idx) = truncate(ctx, *id, *bit_size, max_map) {
            //TODO properly handle signed arithmetic...
            fix_truncate(ctx, truncate_idx, *id, block_idx, value_map);
            new_list.push(truncate_idx);
        }
    }
    to_truncate.clear();
}

//Add required truncate instructions on all blocks
pub(super) fn overflow_strategy(ctx: &mut SsaContext) -> Result<(), RuntimeError> {
    let mut max_map: HashMap<NodeId, BigUint> = HashMap::new();
    let mut memory_map = HashMap::new();
    tree_overflow(ctx, ctx.first_block, &mut max_map, &mut memory_map)
}

//implement overflow strategy following the dominator tree
fn tree_overflow(
    ctx: &mut SsaContext,
    b_idx: BlockId,
    max_map: &mut HashMap<NodeId, BigUint>,
    memory_map: &mut HashMap<u32, NodeId>,
) -> Result<(), RuntimeError> {
    block_overflow(ctx, b_idx, max_map, memory_map)?;
    for b in ctx[b_idx].dominated.clone() {
        tree_overflow(ctx, b, &mut max_map.clone(), &mut memory_map.clone())?;
    }
    Ok(())
}

//overflow strategy for one block
fn block_overflow(
    ctx: &mut SsaContext,
    block_id: BlockId,
    max_map: &mut HashMap<NodeId, BigUint>,
    memory_map: &mut HashMap<u32, NodeId>,
) -> Result<(), RuntimeError> {
    //for each instruction, we compute the resulting max possible value (in term of the field representation of the operation)
    //when it is over the field characteristic, or if the instruction requires it, then we insert truncate instructions
    // The instructions are inserted in a duplicate list( because of rust ownership..), which we use for
    // processing another cse round for the block because the truncates may be duplicated.
    let mut new_list = Vec::new();

    // This needs to be a BTreeMap and not a HashMap so that it can have a deterministic order
    // when we collect it into a Vec later on
    let mut truncate_map = BTreeMap::new();

    let mut modified = false;
    let instructions =
        vecmap(&ctx[block_id].instructions, |id| ctx.try_get_instruction(*id).unwrap().clone());

    //since we process the block from the start, the block value map is not relevant
    let mut value_map = HashMap::new();
    for mut ins in instructions {
        if matches!(
            ins.operation,
            Operation::Nop | Operation::Call { .. } | Operation::Result { .. }
        ) {
            //For now we skip completely functions from overflow; that means arguments are NOT truncated.
            //The reasoning is that this is handled by doing the overflow strategy after the function has been inlined
            continue;
        }

        ins.operation.map_id_mut(|id| {
            let id = optimizations::propagate(ctx, id, &mut modified);
            get_value_from_map(id, &value_map)
        });

        //we propagate optimized loads - todo check if it is needed because there is cse at the end
        //We retrieve get_current_value() in case a previous truncate has updated the value map
        let should_truncate_ins = ins.truncate_required(ctx);
        let ins_max_bits = get_instruction_max(ctx, &ins, max_map, &value_map).bits();
        let res_type = ins.res_type;

        let too_many_bits =
            ins_max_bits > FieldElement::max_num_bits() as u64 && !res_type.is_native_field();

        ins.operation.for_each_id(|id| {
            get_obj_max_value(ctx, id, max_map, &value_map);
            let arg = ctx.try_get_node(id);
            let should_truncate_arg =
                should_truncate_ins && arg.is_some() && !get_type(arg).is_native_field();

            if should_truncate_arg || too_many_bits {
                add_to_truncate(ctx, id, get_size_in_bits(arg), &mut truncate_map, max_map);
            }
        });

        match ins.operation {
            Operation::Load { array_id, index, .. } => {
                if let Some(val) = ctx.get_indexed_value(array_id, index) {
                    //optimize static load
                    ins.mark = Mark::ReplaceWith(*val);
                }
            }
            Operation::Store { array_id, index, value, predicate, .. } => {
                if let Some(idx) = Memory::to_u32(ctx, index) {
                    if ctx.is_one(crate::ssa::conditional::DecisionTree::unwrap_predicate(
                        ctx, &predicate,
                    )) {
                        let absolute_adr = ctx.mem[array_id].absolute_adr(idx);
                        //optimize static store
                        memory_map.insert(absolute_adr, value);
                    }
                }
            }
            Operation::Binary(node::Binary { operator: BinaryOp::Shl, lhs, rhs, .. }) => {
                if let Some(r_const) = ctx.get_as_constant(rhs) {
                    let r_type = ctx[rhs].get_type();
                    let rhs =
                        ctx.get_or_create_const(FieldElement::from(2_i128).pow(&r_const), r_type);
                    ins.operation = Operation::Binary(node::Binary {
                        lhs,
                        rhs,
                        operator: BinaryOp::Mul,
                        predicate: None,
                    });
                }
            }
            Operation::Binary(node::Binary { operator: BinaryOp::Shr(loc), lhs, rhs, .. }) => {
                if !ins.res_type.is_unsigned_integer() {
                    todo!("Right shift is only implemented for unsigned integers");
                }
                if let Some(r_const) = ctx.get_as_constant(rhs) {
                    let r_type = ctx[rhs].get_type();
                    if r_const.to_u128() > r_type.bits() as u128 {
                        ins.mark = Mark::ReplaceWith(ctx.zero_with_type(ins.res_type));
                    } else {
                        let rhs = ctx
                            .get_or_create_const(FieldElement::from(2_i128).pow(&r_const), r_type);
                        ins.operation = Operation::Binary(node::Binary {
                            lhs,
                            rhs,
                            operator: BinaryOp::Udiv(loc),
                            predicate: None,
                        });
                    }
                }
            }
            Operation::Cast(value_id) => {
                // For now the types we support here are only all integer types (field, signed, unsigned, bool)
                // so a cast would normally translate to a truncate.
                // if res_type and lhs have the same bit size (in a large sense, which includes field elements)
                // then either they have the same type and should have been simplified
                // or they don't have the same sign so we keep the cast operator
                // if res_type is smaller than lhs bit size, we look if lhs can hold directly into res_type
                // if not, we need to truncate lhs to a res_type. We modify directly the cast instruction into a truncate
                // in other cases we can keep the cast instruction
                // for instance if res_type is greater than lhs bit size, we need to truncate lhs to its bit size and use the truncate
                // result in the cast, but this is handled by the truncate_required
                // after this function, all cast instructions refer to casting lhs into a bigger (or equal) type
                // any other case has been transformed into the latter using truncates.
                let obj = ctx.try_get_node(value_id);

                if ins.res_type == get_type(obj) {
                    ins.mark = Mark::ReplaceWith(value_id);
                } else {
                    let max = get_obj_max_value(ctx, value_id, max_map, &value_map);
                    let max_bits = max.bits() as u32;

                    if ins.res_type.bits() < get_size_in_bits(obj) && max_bits > ins.res_type.bits()
                    {
                        //we need to truncate
                        ins.operation = Operation::Truncate {
                            value: value_id,
                            bit_size: ins.res_type.bits(),
                            max_bit_size: max_bits,
                        };
                    }
                }
            }
            _ => (),
        }

        process_to_truncate(
            ctx,
            &mut new_list,
            &mut truncate_map,
            max_map,
            block_id,
            &mut value_map,
        );

        let id = match ins.mark {
            Mark::None => ins.id,
            Mark::Deleted => continue,
            Mark::ReplaceWith(new_id) => new_id,
        };

        new_list.push(id);
        ins.operation.map_id_mut(|id| get_value_from_map(id, &value_map));

        if let Operation::Binary(node::Binary {
            rhs,
            operator: BinaryOp::Sub { max_rhs_value } | BinaryOp::SafeSub { max_rhs_value },
            ..
        }) = &mut ins.operation
        {
            //for now we pass the max value to the instruction, we could also keep the max_map e.g in the block (or max in each node object)
            //sub operations require the max value to ensure it does not underflow
            *max_rhs_value = max_map[rhs].clone();
            //we may do that in future when the max_map becomes more used elsewhere (for other optimizations)
        }

        let old_ins = ctx.try_get_mut_instruction(ins.id).unwrap();
        *old_ins = ins;
    }

    update_value_array(ctx, block_id, &value_map);

    //We run another round of CSE for the block in order to remove possible duplicated truncates, this will assign 'new_list' to the block instructions
    let mut modified = false;
    optimizations::cse_block(ctx, block_id, &mut new_list, &mut modified)?;
    Ok(())
}

fn update_value_array(
    ctx: &mut SsaContext,
    block_id: BlockId,
    value_map: &HashMap<NodeId, NodeId>,
) {
    let block = &mut ctx[block_id];
    for (old, new) in value_map {
        block.value_map.insert(*old, *new); //TODO we must merge rather than update
    }
}

//Get current value using the provided value map
fn get_value_from_map(id: NodeId, value_map: &HashMap<NodeId, NodeId>) -> NodeId {
    *value_map.get(&id).unwrap_or(&id)
}

fn get_size_in_bits(obj: Option<&NodeObject>) -> u32 {
    if let Some(v) = obj {
        v.size_in_bits()
    } else {
        0
    }
}

fn get_type(obj: Option<&NodeObject>) -> ObjectType {
    if let Some(v) = obj {
        v.get_type()
    } else {
        ObjectType::NotAnObject
    }
}

fn get_load_max(
    ctx: &SsaContext,
    address: NodeId,
    max_map: &mut HashMap<NodeId, BigUint>,
    value_map: &HashMap<NodeId, NodeId>,
    array: ArrayId,
) -> BigUint {
    if let Some(&value) = ctx.get_indexed_value(array, address) {
        return get_obj_max_value(ctx, value, max_map, value_map);
    };
    ctx.mem[array].max.clone() //return array max
}

//Returns the max value of an operation from an upper bound of left and right hand sides
//Function is used to check for overflows over the field size, this is why we use BigUint.
fn get_max_value(ins: &Instruction, max_map: &mut HashMap<NodeId, BigUint>) -> BigUint {
    let max_value = match &ins.operation {
        Operation::Binary(binary) => get_binary_max_value(binary, ins.res_type, max_map),
        Operation::Not(_) => ins.res_type.max_size(),
        Operation::Constrain(..) => BigUint::zero(),
        //'a cast a' means we cast a into res_type of the instruction
        Operation::Cast(value_id) => {
            let type_max = ins.res_type.max_size();
            BigUint::min(max_map[value_id].clone(), type_max)
        }
        Operation::Truncate { value, max_bit_size, .. } => BigUint::min(
            max_map[value].clone(),
            BigUint::from(2_u32).pow(*max_bit_size) - BigUint::from(1_u32),
        ),
        Operation::Nop | Operation::Jne(..) | Operation::Jeq(..) | Operation::Jmp(_) => {
            unreachable!()
        }
        Operation::Phi { root, block_args } => {
            let mut max = max_map[root].clone();
            for (id, _block) in block_args {
                max = BigUint::max(max, max_map[id].clone());
            }
            max
        }
        Operation::Cond { condition: _, val_true: lhs, val_false: rhs } => {
            let lhs_max = &max_map[lhs];
            let rhs_max = &max_map[rhs];
            lhs_max.max(rhs_max).clone()
        }
        Operation::Load { .. } => unreachable!(),
        Operation::Store { .. } => BigUint::zero(),
        Operation::Call { .. } => ins.res_type.max_size(), //n.b. functions should have been inlined
        Operation::Return(_) => ins.res_type.max_size(),
        Operation::Result { .. } => {
            unreachable!("Functions must have been inlined before checking for overflows")
        }
        Operation::Intrinsic(opcode, _) => opcode.get_max_value(),
    };

    if ins.res_type.is_native_field() {
        let field_max = BigUint::from_bytes_be(&FieldElement::one().neg().to_be_bytes());

        //Native Field operations cannot overflow so they will not be truncated
        if max_value >= field_max {
            return field_max;
        }
    }
    max_value
}

fn get_binary_max_value(
    binary: &node::Binary,
    res_type: ObjectType,
    max_map: &mut HashMap<NodeId, BigUint>,
) -> BigUint {
    let lhs_max = &max_map[&binary.lhs];
    let rhs_max = &max_map[&binary.rhs];

    match &binary.operator {
        BinaryOp::Add => lhs_max + rhs_max,
        BinaryOp::SafeAdd => todo!(),
        BinaryOp::Sub { .. } => {
            let r_mod = BigUint::one() << res_type.bits();
            let mut k = rhs_max / &r_mod;
            if rhs_max % &r_mod != BigUint::zero() {
                k += BigUint::one();
            }
            assert!(&k * &r_mod >= *rhs_max);
            lhs_max + k * r_mod
        }
        BinaryOp::SafeSub { .. } => todo!(),
        BinaryOp::Mul => lhs_max * rhs_max,
        BinaryOp::SafeMul => todo!(),
        BinaryOp::Udiv(_) => lhs_max.clone(),
        BinaryOp::Sdiv(_) => todo!(),
        BinaryOp::Urem(_) => rhs_max - BigUint::one(),
        BinaryOp::Srem(_) => todo!(),
        BinaryOp::Div(_) => FieldElement::modulus() - BigUint::one(),
        BinaryOp::Eq => BigUint::one(),
        BinaryOp::Ne => BigUint::one(),
        BinaryOp::Ult => BigUint::one(),
        BinaryOp::Ule => BigUint::one(),
        BinaryOp::Slt => BigUint::one(),
        BinaryOp::Sle => BigUint::one(),
        BinaryOp::Lt => BigUint::one(),
        BinaryOp::Lte => BigUint::one(),
        BinaryOp::And => {
            BigUint::from(2_u32).pow(u64::min(lhs_max.bits(), rhs_max.bits()) as u32)
                - BigUint::one()
        }
        BinaryOp::Or | BinaryOp::Xor => {
            BigUint::from(2_u32).pow(u64::max(lhs_max.bits(), rhs_max.bits()) as u32)
                - BigUint::one()
        }
        BinaryOp::Assign => rhs_max.clone(),
        BinaryOp::Shl => BigUint::min(
            BigUint::from(2_u32).pow((lhs_max.bits() + 1) as u32) - BigUint::one(),
            res_type.max_size(),
        ),
        BinaryOp::Shr(_) => {
            if lhs_max.bits() >= 1 {
                BigUint::from(2_u32).pow((lhs_max.bits() - 1) as u32) - BigUint::one()
            } else {
                BigUint::zero()
            }
        }
    }
}
