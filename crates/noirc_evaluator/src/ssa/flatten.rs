use super::{
    block::{self, BlockId},
    context::SsaContext,
    function,
    mem::ArrayId,
    node::{
        self, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, NodeObj, ObjectType, Operation,
    },
    optim,
};
use acvm::FieldElement;
use std::collections::{hash_map::Entry, HashMap};

//returns the NodeObj index of a NodeEval object
//if NodeEval is a constant, it may creates a new NodeObj corresponding to the constant value
fn to_index(ctx: &mut SsaContext, obj: NodeEval) -> NodeId {
    match obj {
        NodeEval::Const(c, t) => ctx.get_or_create_const(c, t),
        NodeEval::VarOrInstruction(i) => i,
    }
}
use noirc_frontend::util::vecmap;

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
fn eval_block(block_id: BlockId, eval_map: &HashMap<NodeId, NodeEval>, ctx: &mut SsaContext) {
    for i in &ctx[block_id].instructions.clone() {
        if let Some(ins) = ctx.try_get_mut_instruction(*i) {
            ins.operation = update_operator(&ins.operation, eval_map);
            // TODO: simplify(ctx, ins);
        }
    }
}

fn update_operator(operator: &Operation, eval_map: &HashMap<NodeId, NodeEval>) -> Operation {
    operator.map_id(|id| eval_map.get(&id).and_then(|value| value.into_node_id()).unwrap_or(id))
}

pub fn unroll_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    ctx: &mut SsaContext,
) -> Option<BlockId> {
    if ctx[block_to_unroll].is_join() {
        unroll_join(unrolled_instructions, eval_map, block_to_unroll, ctx)
    } else {
        unroll_std_block(unrolled_instructions, eval_map, block_to_unroll, ctx)
    }
}

//unroll a normal block by generating new instructions into the unroll_ins list, using and updating the eval_map
pub fn unroll_std_block(
    unrolled_instructions: &mut Vec<NodeId>,
    eval_map: &mut HashMap<NodeId, NodeEval>,
    block_to_unroll: BlockId,
    ctx: &mut SsaContext,
) -> Option<BlockId> // The left block
{
    let block = &ctx[block_to_unroll];
    let b_instructions = block.instructions.clone();
    let next = block.left;

    for i_id in &b_instructions {
        match &ctx[*i_id] {
            node::NodeObj::Instr(i) => {
                let new_op = i
                    .operation
                    .map_id(|id| get_current_value(id, eval_map).into_node_id().unwrap());
                let mut new_ins = node::Instruction::new(
                    new_op, i.res_type, None, //TODO to fix later
                );
                match i.operation {
                    Operation::Binary(node::Binary { operator: BinaryOp::Assign, .. }) => {
                        unreachable!("unsupported instruction type when unrolling: assign");
                        //To support assignments, we should create a new variable and updates the eval_map with it
                        //however assignments should have already been removed by copy propagation.
                    }
                    Operation::Jmp(block) => {
                        return Some(block);
                    }
                    Operation::Nop => (),
                    _ => {
                        optim::simplify(ctx, &mut new_ins);

                        match new_ins.mark {
                            Mark::None => {
                                let id = ctx.add_instruction(new_ins);
                                unrolled_instructions.push(id);
                                eval_map.insert(*i_id, NodeEval::VarOrInstruction(id));
                            }
                            Mark::Deleted => (),
                            Mark::ReplaceWith(replacement) => {
                                // TODO: Should we insert into unrolled_instructions as well?
                                // If optim::simplify replaces with a constant then we should not,
                                // otherwise it may make sense if it is not already inserted.
                                eval_map.insert(*i_id, NodeEval::VarOrInstruction(replacement));
                            }
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
    ctx: &mut SsaContext,
) -> Option<BlockId> {
    //Returns the exit block of the loop
    let join = &ctx[block_to_unroll];
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
        evaluate_phi(&join_instructions, from, eval_map, ctx);
        evaluate_conditional_jump(*join_instructions.last().unwrap(), eval_map, ctx)
    } {
        from = block_to_unroll;
        let mut b_id = body_id;
        while let Some(next) = unroll_block(unrolled_instructions, eval_map, b_id, ctx) {
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
    ctx: &mut SsaContext,
) -> Option<BlockId> //next block
{
    assert!(unroll_ins.is_empty());
    let block = &ctx[block_id];
    let b_right = block.right;
    let b_left = block.left;
    let block_instructions = block.instructions.clone();
    if block.is_join() {
        //1. unroll the block into the unroll_ins
        unroll_join(unroll_ins, eval_map, block_id, ctx);
        //2. map the Phis variables to their unrolled values:
        for ins in &block_instructions {
            if let Some(ins_obj) = ctx.try_get_instruction(*ins) {
                if let Operation::Phi { root, .. } = &ins_obj.operation {
                    if let Some(node_eval) = eval_map.get(&ins_obj.id) {
                        let node_eval = *node_eval;
                        eval_map.entry(*root).or_insert(node_eval);
                        //todo test with constants
                    }
                } else if ins_obj.operation != node::Operation::Nop {
                    break; //no more phis
                }
            }
        }
        //3. Merge the unrolled blocks into the join
        for ins in unroll_ins.iter() {
            ctx[*ins].set_id(*ins);
        }
        let join_mut = &mut ctx[block_id];
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
            let sub_graph = block::bfs(body_id, Some(block_id), ctx);
            for b in sub_graph {
                ctx.remove_block(b);
            }
        }

        //5.Finally we clear the unroll_list and go the the next block
        unroll_ins.clear();
    } else {
        //We update block instructions from the eval_map
        eval_block(block_id, eval_map, ctx);
    }
    b_left //returns the next block to process
}

//evaluate phi instruction, coming from 'from' block; retrieve the argument corresponding to the block, evaluates it and update the evaluation map
fn evaluate_phi(
    instructions: &[NodeId],
    from: BlockId,
    to: &mut HashMap<NodeId, NodeEval>,
    ctx: &mut SsaContext,
) {
    for i in instructions {
        let mut to_process = Vec::new();
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if let Operation::Phi { block_args, .. } = &ins.operation {
                for (arg, block) in block_args {
                    if *block == from {
                        //we evaluate the phi instruction value
                        let arg = *arg;
                        let id = ins.id;
                        to_process
                            .push((id, evaluate_one(NodeEval::VarOrInstruction(arg), to, ctx)));
                    }
                }
            } else if ins.operation != node::Operation::Nop {
                break; //phi instructions are placed at the beginning (and after the first dummy instruction)
            }
        }
        //Update the evaluation map.
        for obj in to_process {
            to.insert(obj.0, NodeEval::VarOrInstruction(to_index(ctx, obj.1)));
        }
    }
}

//returns true if we should jump
fn evaluate_conditional_jump(
    jump: NodeId,
    value_array: &mut HashMap<NodeId, NodeEval>,
    ctx: &mut SsaContext,
) -> bool {
    let jump_ins = ctx.try_get_instruction(jump).unwrap();

    let (cond_id, should_jump): (_, fn(FieldElement) -> bool) = match jump_ins.operation {
        Operation::Jeq(cond_id, _) => (cond_id, |field| !field.is_zero()),
        Operation::Jne(cond_id, _) => (cond_id, |field| field.is_zero()),
        Operation::Jmp(_) => return true,
        _ => panic!("loop without conditional statement!"), //TODO shouldn't we return false instead?
    };

    let cond = get_current_value(cond_id, value_array);
    let cond = match evaluate_object(cond, value_array, ctx).into_const_value() {
        Some(c) => c,
        None => unreachable!("Conditional jump argument is non-const: {:?}", evaluate_object(cond, value_array, ctx)),
    };

    should_jump(cond)
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
    ctx: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                return obj;
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    if let Operation::Phi { .. } = i.operation {
                        //n.b phi are handled before, else we should know which block we come from
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    let result = i.evaluate_with(ctx, |_, id| get_current_value(id, value_array));
                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
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
    ctx: &SsaContext,
) -> NodeEval {
    match get_current_value_for_node_eval(obj, value_array) {
        NodeEval::Const(_, _) => obj,
        NodeEval::VarOrInstruction(obj_id) => {
            if ctx.try_get_node(obj_id).is_none() {
                dbg!(obj_id);
                return obj;
            }

            match &ctx[obj_id] {
                NodeObj::Instr(i) => {
                    if let Operation::Phi { .. } = i.operation {
                        dbg!(i.id);
                        return NodeEval::VarOrInstruction(i.id);
                    }

                    //n.b phi are handled before, else we should know which block we come from
                    let result = i.evaluate_with(ctx, |ctx, id| {
                        evaluate_object(get_current_value(id, value_array), value_array, ctx)
                    });

                    if let NodeEval::VarOrInstruction(idx) = result {
                        if ctx.try_get_node(idx).is_none() {
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

//Return false if some inlined function performs a function call
fn inline_block(ctx: &mut SsaContext, block_id: BlockId) -> bool {
    let mut call_ins = vec![];
    for i in &ctx[block_id].instructions {
        if let Some(ins) = ctx.try_get_instruction(*i) {
            if !ins.is_deleted() {
                if let Operation::Call(f, args) = &ins.operation {
                    call_ins.push((ins.id, *f, args.clone(), ins.parent_block));
                }
            }
        }
    }
    let mut result = true;
    for (ins_id, f, args, parent_block) in call_ins {
        if !inline(f, &args, ctx, parent_block, ins_id) {
            result = false;
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
    let mut inline_map = HashMap::<NodeId, NodeId>::new();
    let mut array_map = HashMap::<ArrayId, ArrayId>::new();
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
    array_map: &mut HashMap<ArrayId, ArrayId>,
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
        if let Some(ins) = ctx.try_get_instruction(i_id) {
            if ins.is_deleted() {
                continue;
            }
            let mut array = None;
            let mut array_id = None;
            let mut clone = ins.clone();

            if let node::ObjectType::Pointer(id) = ins.res_type {
                //We need to map arrays to arrays via the array_map, we collect the data here to be mapped below.
                array = Some(ctx.mem[id].clone());
                array_id = Some(id);
            } else if let Operation::Load { array_id: id, .. } = ins.operation {
                array = Some(ctx.mem[id].clone());
                array_id = Some(id);
            } else if let Operation::Store { array_id: id, .. } = ins.operation {
                array = Some(ctx.mem[id].clone());
                array_id = Some(id);
            }

            clone.operation.map_id_mut(|id| {
                function::SSAFunction::get_mapped_value(Some(&id), ctx, inline_map)
            });

            //Arrays are mapped to array. We create the array if not mapped
            if let (Some(array), Some(array_id)) = (array, array_id) {
                if let Entry::Vacant(e) = array_map.entry(array_id) {
                    let new_id =
                        ctx.mem.create_new_array(array.len, array.element_type, &array.name);
                    //We populate the array (if possible) using the inline map
                    for i in &array.values {
                        if let Some(f) = i.to_const() {
                            ctx.mem[new_id].values.push(super::acir_gen::InternalVar::from(f));
                        }
                        //todo: else use inline map.
                    }
                    e.insert(new_id);
                };
            }

            match &clone.operation {
                Operation::Nop => (),
                //Return instruction:
                Operation::Return(values) => {
                    //we need to find the corresponding result instruction in the target block (using ins.rhs) and replace it by ins.lhs
                    for (i, value) in values.iter().enumerate() {
                        ctx.get_result_instruction(target_block_id, call_id, i as u32)
                            .unwrap()
                            .mark = Mark::ReplaceWith(*value);
                    }
                }
                Operation::Call(..) => {
                    *nested_call = true;
                    let new_ins = new_cloned_instruction(clone, target_block_id);
                    push_instruction(ctx, new_ins, &mut new_instructions, inline_map);
                }
                Operation::Load { array_id, index } => {
                    //Compute the new address:
                    //TODO use relative addressing, but that requires a few changes, mainly in acir_gen.rs and integer.rs
                    let b = array_map[array_id];
                    //n.b. this offset is always positive
                    let offset = ctx.mem[b].adr - ctx.mem[*array_id].adr;
                    let index_type = ctx[*index].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);

                    let add = node::Binary { operator: BinaryOp::Add, lhs: offset_id, rhs: *index };
                    let adr_id = ctx.new_instruction(Operation::Binary(add), index_type);
                    let new_ins = Instruction::new(
                        Operation::Load { array_id: array_map[array_id], index: adr_id },
                        clone.res_type,
                        Some(target_block_id),
                    );
                    push_instruction(ctx, new_ins, &mut new_instructions, inline_map);
                }
                Operation::Store { array_id, index, value } => {
                    let b = array_map[array_id];
                    let offset = ctx.mem[*array_id].adr - ctx.mem[b].adr;
                    let index_type = ctx[*index].get_type();
                    let offset_id =
                        ctx.get_or_create_const(FieldElement::from(offset as i128), index_type);

                    let add = node::Binary { operator: BinaryOp::Add, lhs: offset_id, rhs: *index };
                    let adr_id = ctx.new_instruction(Operation::Binary(add), index_type);
                    let new_ins = Instruction::new(
                        Operation::Store {
                            array_id: array_map[array_id],
                            index: adr_id,
                            value: *value,
                        },
                        clone.res_type,
                        Some(target_block_id),
                    );
                    push_instruction(ctx, new_ins, &mut new_instructions, inline_map);
                }
                _ => {
                    let mut new_ins = new_cloned_instruction(clone, target_block_id);

                    if let Some(id) = array_id {
                        if let Some(new_id) = array_map.get(&id) {
                            new_ins.res_type = node::ObjectType::Pointer(*new_id);
                        }
                    }

                    optim::simplify(ctx, &mut new_ins);

                    if let Mark::ReplaceWith(replacement) = new_ins.mark {
                        if let Some(id) = array_id {
                            if let Entry::Occupied(mut entry) = array_map.entry(id) {
                                if let ObjectType::Pointer(new_id) = ctx[replacement].get_type() {
                                    //we now map the array to rhs array
                                    entry.insert(new_id);
                                }
                            }
                        }

                        if replacement != new_ins.id {
                            inline_map.insert(i_id, replacement);
                        }
                    } else {
                        push_instruction(ctx, new_ins, &mut new_instructions, inline_map);
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

fn new_cloned_instruction(original: Instruction, block: BlockId) -> Instruction {
    let mut clone = Instruction::new(original.operation, original.res_type, Some(block));
    // Take the original's ID, it will be used to map it as a replacement in push_instruction later
    clone.id = original.id;
    clone
}

fn push_instruction(
    ctx: &mut SsaContext,
    instruction: Instruction,
    new_instructions: &mut Vec<NodeId>,
    inline_map: &mut HashMap<NodeId, NodeId>,
) {
    let old_id = instruction.id;
    let new_id = ctx.add_instruction(instruction);
    new_instructions.push(new_id);
    inline_map.insert(old_id, new_id);
}
