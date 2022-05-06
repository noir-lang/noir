use super::{
    block::{self, BlockId},
    context::SsaContext,
    function,
    node::{self, Node, NodeEval, NodeId, NodeObj, Operation},
    optim,
};
use acvm::FieldElement;
use noirc_frontend::util::vecmap;
use std::collections::HashMap;

// Number of allowed times for inlining function calls inside a code block.
// If a function calls another function, the inlining of the first function will leave the second function call that needs to be inlined as well.
// In case of recursive calls, this iterative inlining does not end so we arbitraty limit it. 100 nested calls should already support very complex programs.
const MAX_INLINE_TRIES: u32 = 100;

//Unroll the CFG
pub fn unroll_tree(ctx: &mut SsaContext, mut block_id: BlockId) {
    //Calls outer_unroll() from the root node
    let mut unroll_ins = Vec::new();
    let mut eval_map = HashMap::new();
    while let Some(next) = outer_unroll(&mut unroll_ins, &mut eval_map, block_id, ctx) {
        block_id = next;
    }
}

//Update the block instruction list using the eval_map
fn eval_block(block_id: BlockId, eval_map: &HashMap<NodeId, NodeEval>, igen: &mut SsaContext) {
    for i in &igen[block_id].instructions.clone() {
        //RIA
        if let Some(ins) = igen.try_get_mut_instruction(*i) {
            if let Some(value) = eval_map.get(&ins.rhs) {
                ins.rhs = value.into_node_id().unwrap();
            }
            if let Some(value) = eval_map.get(&ins.lhs) {
                ins.lhs = value.into_node_id().unwrap();
            }
            //TODO simplify(ctx, ins);
        }
    }
}

pub fn unroll_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut SsaContext,
) -> Option<BlockId> {
    if igen[block_to_unroll].is_join() {
        unroll_join(unrolled_instructions, eval_map, block_to_unroll, igen)
    } else if let Some(i) = unroll_std_block(unrolled_instructions, eval_map, block_to_unroll, igen)
    {
        igen.try_get_instruction(i).map(|ins| ins.parent_block)
    } else {
        None
    }
}

//unroll a normal block by generating new instructions into the unroll_ins list, using and updating the eval_map
pub fn unroll_std_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut SsaContext,
) -> Option<NodeId> //first instruction of the left block
{
    let block = &igen[block_to_unroll];
    let b_instructions = block.instructions.clone();
    let mut next = None;
    if let Some(left) = block.left {
        if let Some(f) = igen[left].instructions.first() {
            next = Some(*f);
        }
    }
    for i_id in &b_instructions {
        match &igen[*i_id] {
            node::NodeObj::Instr(i) => {
                let new_left = get_current_value(i.lhs, eval_map).into_node_id().unwrap();
                let new_right = get_current_value(i.rhs, eval_map).into_node_id().unwrap();
                let mut new_ins = node::Instruction::new(
                    i.operator, new_left, new_right, i.res_type, None, //TODO to fix later
                );
                match i.operator {
                    Operation::Ass => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    Operation::Jmp => {
                        return Some(i.rhs);
                    }
                    Operation::Nop => (),
                    _ => {
                        optim::simplify(igen, &mut new_ins);
                        let result_id;
                        let mut to_delete = false;
                        if new_ins.is_deleted {
                            result_id = new_ins.rhs;
                            if new_ins.rhs == new_ins.id {
                                to_delete = true;
                            }
                        } else {
                            result_id = igen.add_instruction(new_ins);
                            unrolled_instructions.push(result_id);
                        }
                        //ignore self-deleted instructions
                        if !to_delete {
                            eval_map.insert(*i_id, NodeEval::VarOrInstruction(result_id));
                        }
                    }
                }
            }
            _ => todo!(), //ERROR
        }
    }
    next
}

//Unroll a for loop: exit <- join <--> body
//join block is given in argumemt, it will evaluate the join condition, starting from 'start' until it reaches 'end'
//and write the unrolled instructions of the body block into the unroll_ins list
//If the body does not ends with a jump back to the join block, we continue to unroll the next block, until we reach the join.
//If there is a nested loop, unroll_block will call recursively unroll_join, keeping unroll list and eval map from the previous one
pub fn unroll_join(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    igen: &mut SsaContext,
) -> Option<BlockId> {
    //Returns the exit block of the loop
    let join = &igen[block_to_unroll];
    let join_instructions = join.instructions.clone();
    let join_left = join.left; //XXX.clone();
    let prev = *join.predecessor.first().unwrap(); //todo predecessor.first or .last?

    let mut from = prev; //todo caller?
    assert!(join.is_join());
    let body_id = join.right.unwrap();
    if unrolled_instructions.is_empty() {
        unrolled_instructions.push(*join_instructions.first().unwrap()); //TODO is it needed? we also should assert it is a nop instruction.
    }
    while {
        //evaluate the join  block:
        evaluate_phi(&join_instructions, from, eval_map, igen);
        evaluate_conditional_jump(*join_instructions.last().unwrap(), eval_map, igen)
    } {
        from = block_to_unroll;
        let mut b_id = body_id;
        while let Some(next) = unroll_block(unrolled_instructions, eval_map, b_id, igen) {
            //process next block:
            from = b_id;
            b_id = next;
            if b_id == block_to_unroll {
                //looping back to the join block; we are done
                break;
            }
        }
    }
    join_left
}

// Unrolling outer loops, i.e non-nested loops
pub fn outer_unroll(
    unroll_ins: &mut Vec<NodeId>, //unrolled instructions
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_id: BlockId, //block to unroll
    igen: &mut SsaContext,
) -> Option<BlockId> //next block
{
    assert!(unroll_ins.is_empty());
    let block = &igen[block_id];
    let b_right = block.right;
    let b_left = block.left;
    let block_instructions = block.instructions.clone();
    if block.is_join() {
        //1. unroll the block into the unroll_ins
        unroll_join(unroll_ins, eval_map, block_id, igen);
        //2. map the Phis variables to their unrolled values:
        for ins in &block_instructions {
            if let Some(ins_obj) = igen.try_get_instruction(*ins) {
                if ins_obj.operator == node::Operation::Phi {
                    if eval_map.contains_key(&ins_obj.rhs) {
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.rhs]);
                        //todo test with constants
                    } else if eval_map.contains_key(&ins_obj.id) {
                        //   unroll_map.insert(ins_obj.lhs, eval_map[&ins_obj.idx].to_index().unwrap());
                        eval_map.insert(ins_obj.lhs, eval_map[&ins_obj.id]);
                        //todo test with constants
                    }
                } else if ins_obj.operator != node::Operation::Nop {
                    break; //no more phis
                }
            }
        }
        //3. Merge the unrolled blocks into the join
        for ins in unroll_ins.iter() {
            igen[*ins].set_id(*ins);
        }
        let join_mut = &mut igen[block_id];
        join_mut.instructions = unroll_ins.clone();
        join_mut.right = None;
        join_mut.kind = block::BlockType::Normal;

        //4. Remove the right sub-graph of the join block
        //Note that this is not done in unroll_join because in case of nested loops we need to unroll_join a block several times.
        if b_left.is_some() {
            join_mut.dominated = vec![b_left.unwrap()];
        } else {
            join_mut.dominated.clear();
        }
        //we get the subgraph, however we could retrieve the list of processed blocks directly in unroll_join (cf. processed)
        if let Some(body_id) = b_right {
            let sub_graph = block::bfs(body_id, Some(block_id), igen);
            for b in sub_graph {
                igen.remove_block(b);
            }
        }

        //5.Finally we clear the unroll_list and go the the next block
        unroll_ins.clear();
    } else {
        //We update block instructions from the eval_map
        eval_block(block_id, eval_map, igen);
    }
    b_left //returns the next block to process
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[NodeId],
    from: BlockId,
    to: &mut HashMap<NodeId, NodeEval>,
    igen: &mut SsaContext,
) {
    for i in instructions {
        let mut to_process = Vec::new();
        if let Some(ins) = igen.try_get_instruction(*i) {
            if ins.operator == node::Operation::Phi {
                for phi in &ins.phi_arguments {
                    if phi.1 == from {
                        //we evaluate the phi instruction value
                        to_process.push((
                            ins.id,
                            evaluate_one(NodeEval::VarOrInstruction(phi.0), to, igen),
                        ));
                    }
                }
            } else if ins.operator != node::Operation::Nop {
                break; //phi instructions are placed at the beginning (and after the first dummy instruction)
            }
        }
        //Update the evaluation map.
        for obj in to_process {
            to.insert(obj.0, NodeEval::VarOrInstruction(optim::to_index(igen, obj.1)));
        }
    }
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: NodeId,
    value_array: &mut HashMap<NodeId, NodeEval>,
    ctx: &SsaContext,
) -> bool {
    let jump_ins = ctx.try_get_instruction(jump).unwrap();
    let lhs = get_current_value(jump_ins.lhs, value_array);
    let cond = evaluate_object(lhs, value_array, ctx);
    if let Some(cond_const) = cond.into_const_value() {
        let result = !cond_const.is_zero();
        match jump_ins.operator {
            node::Operation::Jeq => return result,
            node::Operation::Jne => return !result,
            node::Operation::Jmp => return true,
            _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
        }
    }

    unreachable!("Condition should be constant");
}

//Retrieve the NodeEval value of the index in the evaluation map
fn get_current_value(id: NodeId, value_array: &HashMap<NodeId, NodeEval>) -> NodeEval {
    *value_array.get(&id).unwrap_or(&NodeEval::VarOrInstruction(id))
}

//Same as get_current_value but for a NodeEval object instead of a NodeObj
fn get_current_value_for_node_eval(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
) -> NodeEval {
    match obj {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => get_current_value(obj_id, value_array),
    }
}

//evaluate the object without recursion, doing only one step of evaluation
fn evaluate_one(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    igen: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if igen.try_get_node(obj_id).is_none() {
                return obj;
            }

            match &igen[obj_id] {
                NodeObj::Instr(i) => {
                    if i.operator == node::Operation::Phi {
                        //n.b phi are handled before, else we should know which block we come from
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    let lhs = get_current_value(i.lhs, value_array);
                    let lhr = get_current_value(i.rhs, value_array);
                    let result = i.evaluate(&lhs, &lhr);
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if igen.try_get_node(idx).is_none() {
                            return NodeEval::VarOrInstruction(obj_id);
                        }
                    }
                    result
                }
                NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    NodeEval::Const(value, c.get_type())
                }
                NodeObj::Obj(_) => NodeEval::VarOrInstruction(obj_id),
            }
        }
    }
}

//Evaluate an object recursively
fn evaluate_object(
    obj: NodeEval,
    value_array: &HashMap<NodeId, NodeEval>,
    igen: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if igen.try_get_node(obj_id).is_none() {
                dbg!(obj_id);
                return obj;
            }

            match &igen[obj_id] {
                NodeObj::Instr(i) => {
                    if i.operator == Operation::Phi {
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }
                    //n.b phi are handled before, else we should know which block we come from
                    let lhs =
                        evaluate_object(get_current_value(i.lhs, value_array), value_array, igen);
                    let lhr =
                        evaluate_object(get_current_value(i.rhs, value_array), value_array, igen);
                    let result = i.evaluate(&lhs, &lhr);
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if igen.try_get_node(idx).is_none() {
                            return NodeEval::VarOrInstruction(obj_id);
                        }
                    }
                    result
                }
                NodeObj::Const(c) => {
                    let value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                    NodeEval::Const(value, c.get_type())
                }
                NodeObj::Obj(_) => NodeEval::VarOrInstruction(obj_id),
            }
        }
    }
}

//inline main
pub fn inline_tree(ctx: &mut SsaContext, block_id: BlockId) {
    //inline all function calls
    let mut retry = MAX_INLINE_TRIES;
    while retry > 0 && !inline_block(ctx, ctx.first_block) {
        retry -= 1;
    }
    assert!(retry > 0, "Error - too many nested calls");
    for b in ctx[block_id].dominated.clone() {
        inline_tree(ctx, b);
    }
}

fn inline_cfg(ctx: &mut SsaContext, entry_block: BlockId) -> bool {
    let mut result = true;
    let func_cfg = block::bfs(entry_block, None, ctx);
    for block_id in func_cfg {
        if !inline_block(ctx, block_id) {
            result = false;
        }
    }
    result
}

//inline each function so that a function does not call anyy other function
//There is probably an optimal order to use when inlining the functions
//for now we inline all functions one time, and then recurse until no more function call
pub fn inline_all_functions(ctx: &mut SsaContext) {
    let mut nested_call = true;
    let mut retry = MAX_INLINE_TRIES;
    let func_cfg: Vec<BlockId> = vecmap(ctx.functions.values(), |f| f.entry_block);
    while retry > 0 && nested_call {
        retry -= 1;
        nested_call = false;
        for k in &func_cfg {
            if !inline_cfg(ctx, *k) {
                nested_call = true;
            }
        }
    }
}

//inline all function calls of the block
//Return false if some inlined function performs a function call
fn inline_block(ctx: &mut SsaContext, block_id: BlockId) -> bool {
    let mut call_ins = Vec::<NodeId>::new();
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted && matches!(ins.operator, node::Operation::Call(_)) {
                call_ins.push(*i);
            }
        }
    }
    let mut result = true;
    for ins_id in call_ins {
        let ins = ctx.try_get_instruction(ins_id).unwrap().clone();
        if let node::Instruction { operator: node::Operation::Call(f), .. } = ins {
            if !inline(f, &ins.ins_arguments, ctx, ins.parent_block, ins.id) {
                result = false;
            }
        }
    }
    optim::cse(ctx, block_id); //handles the deleted call instructions
    result
}

//inline a function call
//Return false if the inlined function performs a function call
pub fn inline(
    func_id: noirc_frontend::node_interner::FuncId,
    args: &[NodeId],
    ctx: &mut SsaContext,
    block: BlockId,
    call_id: NodeId,
) -> bool {
    let ssa_func = ctx.get_ssafunc(func_id).unwrap();

    //map nodes from the function cfg to the caller cfg
    let mut inline_map: HashMap<NodeId, NodeId> = HashMap::new();
    let mut array_map: HashMap<u32, u32> = HashMap::new();
    //1. map function parameters
    for (arg_caller, arg_function) in args.iter().zip(&ssa_func.arguments) {
        inline_map.insert(*arg_function, *arg_caller);
    }
    let mut result = true;
    //2. inline in the block: we assume the function cfg is already flatened.
    let mut next_block = Some(ssa_func.entry_block);
    while let Some(next_b) = next_block {
        let mut nested_call = false;
        next_block = inline_in_block(
            next_b,
            block,
            &mut inline_map,
            &mut array_map,
            call_id,
            &mut nested_call,
            ctx,
        );
        if result && nested_call {
            result = false
        }
    }
    result
}

//inline the given block of the function body into the target_block
pub fn inline_in_block(
    block_id: BlockId,
    target_block_id: BlockId,
    inline_map: &mut HashMap<NodeId, NodeId>,
    array_map: &mut HashMap<u32, u32>,
    call_id: NodeId,
    nested_call: &mut bool,
    ctx: &mut SsaContext,
) -> Option<BlockId> {
    let mut new_instructions: Vec<NodeId> = Vec::new();
    let block_func = &ctx[block_id];
    let next_block = block_func.left;
    let block_func_instructions = &block_func.instructions.clone();
    *nested_call = false;
    for &i_id in block_func_instructions {
        let mut array_func = None;
        let mut array_func_idx = u32::MAX;
        if let Some(ins) = ctx.try_get_instruction(i_id) {
            if ins.is_deleted {
                continue;
            }
            let clone = ins.clone();
            if let node::ObjectType::Pointer(a) = ins.res_type {
                //We need to map arrays to arrays via the array_map, we collect the data here to be mapped below.
                array_func = Some(ctx.mem.arrays[a as usize].clone());
                array_func_idx = a;
            } else if let Operation::Load(a) = ins.operator {
                array_func = Some(ctx.mem.arrays[a as usize].clone());
                array_func_idx = a;
            } else if let Operation::Store(a) = ins.operator {
                array_func = Some(ctx.mem.arrays[a as usize].clone());
                array_func_idx = a;
            }

            let new_left =
                function::SSAFunction::get_mapped_value(Some(&clone.lhs), ctx, inline_map);
            let new_right =
                function::SSAFunction::get_mapped_value(Some(&clone.rhs), ctx, inline_map);
            let new_arg = function::SSAFunction::get_mapped_value(
                clone.ins_arguments.first(),
                ctx,
                inline_map,
            );
            //Arrays are mapped to array. We create the array if not mapped
            if let Some(a) = array_func {
                if let std::collections::hash_map::Entry::Vacant(e) =
                    array_map.entry(array_func_idx)
                {
                    let i_pointer = ctx.mem.create_new_array(a.len, a.element_type, &a.name);
                    //We populate the array (if possible) using the inline map
                    for i in &a.values {
                        if let Some(f) = i.to_const() {
                            ctx.mem.arrays[i_pointer as usize]
                                .values
                                .push(super::acir_gen::InternalVar::from(f));
                        }
                        //todo: else use inline map.
                    }
                    e.insert(i_pointer);
                };
            }

            match clone.operator {
                Operation::Nop => (),
                //Return instruction:
                Operation::Ret => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    if let Some(ret_id) = ctx[target_block_id].get_result_instruction(call_id, ctx)
                    {
                        //we support only one result for now, should use 'ins.lhs.get_value()'
                        if let node::NodeObj::Instr(i) = &mut ctx[ret_id] {
                            i.is_deleted = true;
                            i.rhs = new_left; //Then we need to ensure there is a CSE.
                        }
                    } else {
                        //we use the call instruction instead
                        //we could use the ins_arguments to get the results here, and since we have the input arguments (in the ssafunction) we know how many there are.
                        //for now the call instruction is replaced by the (one) result
                        let call_ins = ctx.get_mut_instruction(call_id);
                        call_ins.is_deleted = true;
                        call_ins.rhs = new_arg;
                        if array_map.contains_key(&array_func_idx) {
                            let i_pointer = array_map[&array_func_idx];
                            call_ins.res_type = node::ObjectType::Pointer(i_pointer);
                        }
                    }
                }
                Operation::Call(_) => {
                    *nested_call = true;

                    let mut new_ins = node::Instruction::new(
                        clone.operator,
                        new_left,
                        new_right,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    new_ins.ins_arguments = Vec::new();
                    for i in clone.ins_arguments {
                        new_ins.ins_arguments.push(function::SSAFunction::get_mapped_value(
                            Some(&i),
                            ctx,
                            inline_map,
                        ));
                    }
                    let result_id = ctx.add_instruction(new_ins);
                    new_instructions.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                Operation::Load(a) => {
                    //Compute the new address:
                    //TODO use relative addressing, but that requires a few changes, mainly in acir_gen.rs and integer.rs
                    let b = array_map[&a];
                    //n.b. this offset is always positive
                    let offset = ctx.mem.arrays[b as usize].adr - ctx.mem.arrays[a as usize].adr;
                    let index_type = ctx[new_left].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);
                    let adr_id =
                        ctx.new_instruction(offset_id, new_left, node::Operation::Add, index_type);
                    let new_ins = node::Instruction::new(
                        node::Operation::Load(array_map[&a]),
                        adr_id,
                        adr_id,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    let result_id = ctx.add_instruction(new_ins);
                    new_instructions.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                Operation::Store(a) => {
                    let b = array_map[&a];
                    let offset = ctx.mem.arrays[a as usize].adr - ctx.mem.arrays[b as usize].adr;
                    let index_type = ctx[new_left].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);
                    let adr_id =
                        ctx.new_instruction(offset_id, new_left, node::Operation::Add, index_type);
                    let new_ins = node::Instruction::new(
                        node::Operation::Store(array_map[&a]),
                        new_left,
                        adr_id,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    let result_id = ctx.add_instruction(new_ins);
                    new_instructions.push(result_id);
                    inline_map.insert(i_id, result_id);
                }
                _ => {
                    let mut new_ins = node::Instruction::new(
                        clone.operator,
                        new_left,
                        new_right,
                        clone.res_type,
                        Some(target_block_id),
                    );
                    if array_map.contains_key(&array_func_idx) {
                        let i_pointer = array_map[&array_func_idx];
                        new_ins.res_type = node::ObjectType::Pointer(i_pointer);
                    }
                    optim::simplify(ctx, &mut new_ins);
                    let result_id;
                    let mut to_delete = false;
                    if new_ins.is_deleted {
                        result_id = new_ins.rhs;
                        if let std::collections::hash_map::Entry::Occupied(mut e) =
                            array_map.entry(array_func_idx)
                        {
                            if let node::ObjectType::Pointer(a) = ctx[result_id].get_type() {
                                //we now map the array to rhs array
                                e.insert(a);
                            }
                        }
                        if new_ins.rhs == new_ins.id {
                            to_delete = true;
                        }
                    } else {
                        result_id = ctx.add_instruction(new_ins);
                        new_instructions.push(result_id);
                    }
                    //ignore self-deleted instructions
                    if !to_delete {
                        inline_map.insert(i_id, result_id);
                    }
                }
            }
        }
    }

    // add instruction to target_block, at proper location (really need a linked list!)
    let mut pos = ctx[target_block_id].instructions.iter().position(|x| *x == call_id).unwrap();
    for &new_id in &new_instructions {
        ctx[target_block_id].instructions.insert(pos, new_id);
        pos += 1;
    }

    next_block
}
