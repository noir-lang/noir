use crate::errors::RuntimeError;
use crate::ssa::{
    conditional::AssumptionId,
    context::SsaContext,
    mem::ArrayId,
    node,
    node::{Instruction, Mark, NodeId, Opcode},
};
use std::collections::{HashMap, HashSet, VecDeque};

// A short-circuited block should not have more than 4 instructions (a nop, an optional not, an optional condition and a failing constraint)
// so we do not need to check the whole instruction list when looking for short-circuit instructions.
const MAX_SHORT_CIRCUIT_LEN: usize = 4;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum BlockType {
    Normal,
    ForJoin,
    IfJoin,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BlockId(pub(super) arena::Index);

impl BlockId {
    pub(super) fn dummy() -> BlockId {
        BlockId(SsaContext::dummy_id())
    }
    pub(super) fn is_dummy(&self) -> bool {
        *self == BlockId::dummy()
    }
}

#[derive(Debug)]
pub(crate) struct BasicBlock {
    pub(crate) id: BlockId,
    pub(crate) kind: BlockType,
    pub(crate) dominator: Option<BlockId>, //direct dominator
    pub(crate) dominated: Vec<BlockId>,    //dominated sons
    pub(crate) predecessor: Vec<BlockId>,  //for computing the dominator tree
    pub(crate) left: Option<BlockId>,      //sequential successor
    pub(crate) right: Option<BlockId>,     //jump successor
    pub(crate) instructions: Vec<NodeId>,
    pub(crate) value_map: HashMap<NodeId, NodeId>, //for generating the ssa form
    pub(crate) assumption: AssumptionId,
}

impl BasicBlock {
    pub(crate) fn new(prev: BlockId, kind: BlockType) -> BasicBlock {
        BasicBlock {
            id: BlockId(SsaContext::dummy_id()),
            predecessor: vec![prev],
            left: None,
            right: None,
            instructions: Vec::new(),
            value_map: HashMap::new(),
            dominator: None,
            dominated: Vec::new(),
            kind,
            assumption: AssumptionId::dummy(),
        }
    }

    pub(crate) fn get_current_value(&self, id: NodeId) -> Option<NodeId> {
        self.value_map.get(&id).copied()
    }

    //When generating a new instance of a variable because of ssa, we update the value array
    //to link the two variables
    pub(crate) fn update_variable(&mut self, old_value: NodeId, new_value: NodeId) {
        self.value_map.insert(old_value, new_value);
    }

    pub(crate) fn get_first_instruction(&self) -> NodeId {
        self.instructions[0]
    }

    pub(crate) fn is_join(&self) -> bool {
        self.kind == BlockType::ForJoin
    }

    //Create the first block for a CFG
    pub(crate) fn create_cfg(ctx: &mut SsaContext) -> BlockId {
        let root_block = BasicBlock::new(BlockId::dummy(), BlockType::Normal);
        let root_block = ctx.insert_block(root_block);
        root_block.predecessor = Vec::new();
        let root_id = root_block.id;
        ctx.current_block = root_id;
        ctx.sealed_blocks.insert(root_id);
        ctx.new_instruction(node::Operation::Nop, node::ObjectType::NotAnObject).unwrap();
        root_id
    }

    pub(crate) fn written_arrays(&self, ctx: &SsaContext) -> HashSet<ArrayId> {
        let mut result = HashSet::new();
        for i in &self.instructions {
            if let Some(node::Instruction {
                operation: node::Operation::Store { array_id: x, .. },
                ..
            }) = ctx.try_get_instruction(*i)
            {
                result.insert(*x);
            }

            if let Some(ins) = ctx.try_get_instruction(*i) {
                match &ins.operation {
                    node::Operation::Store { array_id: a, .. } => {
                        result.insert(*a);
                    }
                    node::Operation::Intrinsic(..) => {
                        if let node::ObjectType::ArrayPointer(a) = ins.res_type {
                            result.insert(a);
                        }
                    }
                    node::Operation::Call { func, returned_arrays, .. } => {
                        for a in returned_arrays {
                            result.insert(a.0);
                        }
                        if let Some(f) = ctx.try_get_ssa_func(*func) {
                            for typ in &f.result_types {
                                if let node::ObjectType::ArrayPointer(a) = typ {
                                    result.insert(*a);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        result
    }

    //Returns true if the block is a short-circuit
    fn is_short_circuit(&self, ctx: &SsaContext, assumption: Option<NodeId>) -> bool {
        let mut cond = None;
        for (i, ins_id) in self.instructions.iter().enumerate() {
            if let Some(ins) = ctx.try_get_instruction(*ins_id) {
                if let Some(ass_value) = assumption {
                    if cond.is_none()
                        && ins.operation
                            == (node::Operation::Cond {
                                condition: ass_value,
                                val_true: ctx.zero(),
                                val_false: ctx.one(),
                            })
                    {
                        cond = Some(*ins_id);
                    }

                    if let node::Operation::Constrain(a, _) = ins.operation {
                        if a == ctx.zero() || Some(a) == cond {
                            return true;
                        }
                    }
                }
            }
            if i > MAX_SHORT_CIRCUIT_LEN {
                break;
            }
        }

        false
    }
}

pub(crate) fn create_first_block(ctx: &mut SsaContext) {
    ctx.first_block = BasicBlock::create_cfg(ctx);
}

//Creates a new sealed block (i.e whose predecessors are known)
//It is not suitable for the first block because it uses the current block.
//if left is true, the new block is left to the current block
pub(crate) fn new_sealed_block(ctx: &mut SsaContext, kind: BlockType, left: bool) -> BlockId {
    let current_block = ctx.current_block;
    let new_block = BasicBlock::new(ctx.current_block, kind);
    let new_block = ctx.insert_block(new_block);
    let new_id = new_block.id;

    new_block.dominator = Some(current_block);
    ctx.sealed_blocks.insert(new_id);

    //update current block
    if left {
        let cb = ctx.get_current_block_mut();
        cb.left = Some(new_id);
    }
    ctx.current_block = new_id;
    ctx.new_instruction(node::Operation::Nop, node::ObjectType::NotAnObject).unwrap();
    new_id
}

//if left is true, the new block is left to the current block
pub(crate) fn new_unsealed_block(ctx: &mut SsaContext, kind: BlockType, left: bool) -> BlockId {
    let current_block = ctx.current_block;
    let new_block = create_block(ctx, kind);
    new_block.dominator = Some(current_block);
    let new_idx = new_block.id;

    //update current block
    let cb = ctx.get_current_block_mut();
    if left {
        cb.left = Some(new_idx);
    } else {
        cb.right = Some(new_idx);
    }

    ctx.current_block = new_idx;
    ctx.new_instruction(node::Operation::Nop, node::ObjectType::NotAnObject).unwrap();
    new_idx
}

//create a block and sets its id, but do not update current block, and do not add dummy instruction!
pub(crate) fn create_block(ctx: &mut SsaContext, kind: BlockType) -> &mut BasicBlock {
    let new_block = BasicBlock::new(ctx.current_block, kind);
    ctx.insert_block(new_block)
}

//link the current block to the target block so that current block becomes its target
pub(crate) fn link_with_target(
    ctx: &mut SsaContext,
    target: BlockId,
    left: Option<BlockId>,
    right: Option<BlockId>,
) {
    if let Some(target_block) = ctx.try_get_block_mut(target) {
        target_block.right = right;
        target_block.left = left;
        //TODO should also update the last instruction rhs to the first instruction of the current block  -- TODO should we do it here??
        if let Some(right_uw) = right {
            ctx[right_uw].dominator = Some(target);
        }
        if let Some(left_uw) = left {
            ctx[left_uw].dominator = Some(target);
        }
    }
}

pub(crate) fn compute_dom(ctx: &mut SsaContext) {
    let mut dominator_link = HashMap::new();

    for block in ctx.iter_blocks() {
        if let Some(dom) = block.dominator {
            dominator_link.entry(dom).or_insert_with(Vec::new).push(block.id);
        }
    }
    for (master, slave_vec) in dominator_link {
        if let Some(dom_b) = ctx.try_get_block_mut(master) {
            dom_b.dominated.clear();
            for slave in slave_vec {
                dom_b.dominated.push(slave);
            }
        }
    }
}

pub(crate) fn compute_sub_dom(ctx: &mut SsaContext, blocks: &[BlockId]) {
    let mut dominator_link = HashMap::new();

    for &block_id in blocks {
        let block = &ctx[block_id];
        if let Some(dom) = block.dominator {
            dominator_link.entry(dom).or_insert_with(Vec::new).push(block.id);
        }
    }
    for (master, slave_vec) in dominator_link {
        let dom_b = &mut ctx[master];
        for slave in slave_vec {
            dom_b.dominated.push(slave);
        }
    }
}

//breadth-first traversal of the CFG, from start, until we reach stop
pub(crate) fn bfs(start: BlockId, stop: Option<BlockId>, ctx: &SsaContext) -> Vec<BlockId> {
    let mut result = vec![start]; //list of blocks in the visited subgraph
    let mut queue = VecDeque::new(); //Queue of elements to visit
    queue.push_back(start);

    while !queue.is_empty() {
        let block = &ctx[queue.pop_front().unwrap()];

        let mut test_and_push = |block_opt| {
            if let Some(block_id) = block_opt {
                if stop == Some(block_id) {
                    return;
                }
                if !block_id.is_dummy() && !result.contains(&block_id) {
                    result.push(block_id);
                    queue.push_back(block_id);
                }
            }
        };

        test_and_push(block.left);
        test_and_push(block.right);
    }

    result
}

//Find the exit (i.e join) block from a IF (i.e split) block
pub(crate) fn find_join(ctx: &SsaContext, block_id: BlockId) -> BlockId {
    let mut processed = HashMap::new();
    find_join_helper(ctx, block_id, &mut processed)
}

//We follow down the path from the THEN and ELSE branches until we reach a common descendant
fn find_join_helper(
    ctx: &SsaContext,
    block_id: BlockId,
    processed: &mut HashMap<BlockId, BlockId>,
) -> BlockId {
    let mut left = ctx[block_id].left.unwrap();
    let mut right = ctx[block_id].right.unwrap();
    let mut left_descendants = Vec::new();
    let mut right_descendants = Vec::new();

    while !left.is_dummy() || !right.is_dummy() {
        if let Some(block) = get_only_descendant(ctx, left, processed) {
            left_descendants.push(block);
            left = block;
            if right_descendants.contains(&block) {
                return block;
            }
        }
        if let Some(block) = get_only_descendant(ctx, right, processed) {
            right_descendants.push(block);
            right = block;
            if left_descendants.contains(&block) {
                return block;
            }
        }
    }
    unreachable!("no join");
}

// Find the LCA of x and y
// n.b. this is a naive implementation which assumes there is no cycle in the graph, so it should be used after loop flattening
pub(super) fn lca(ctx: &SsaContext, x: BlockId, y: BlockId) -> BlockId {
    if x == y {
        return x;
    }
    let mut pred_x: HashSet<BlockId> = HashSet::new();
    let mut pred_y: HashSet<BlockId> = HashSet::new();
    let mut to_process_x = ctx[x].predecessor.clone();
    let mut to_process_y = ctx[y].predecessor.clone();
    pred_x.insert(x);
    pred_y.insert(y);
    pred_x.extend(&ctx[x].predecessor);
    pred_y.extend(&ctx[y].predecessor);

    while !to_process_x.is_empty() || !to_process_y.is_empty() {
        if let Some(b) = to_process_x.pop() {
            if pred_y.contains(&b) {
                return b;
            }
            to_process_x.extend(&ctx[b].predecessor);
            pred_x.extend(&ctx[b].predecessor);
        }
        if let Some(b) = to_process_y.pop() {
            if pred_x.contains(&b) {
                return b;
            }
            to_process_y.extend(&ctx[b].predecessor);
            pred_y.extend(&ctx[b].predecessor);
        }
    }
    unreachable!("Blocks {:?} and {:?} are not connected", x, y);
}

//get the most direct descendant which is 'only child'
fn get_only_descendant(
    ctx: &SsaContext,
    block_id: BlockId,
    processed: &mut HashMap<BlockId, BlockId>,
) -> Option<BlockId> {
    if block_id == BlockId::dummy() {
        return None;
    }
    let block = &ctx[block_id];
    if block.right.is_none() || block.kind == BlockType::ForJoin {
        if let Some(left) = block.left {
            processed.insert(block_id, left);
        }
        block.left
    } else {
        if processed.contains_key(&block_id) {
            return Some(processed[&block_id]);
        }
        let descendant = find_join_helper(ctx, block_id, processed);
        processed.insert(block_id, descendant);
        Some(descendant)
    }
}

//Set left as the left block of block_id
//Set block_id as the only parent of left
pub(super) fn rewire_block_left(ctx: &mut SsaContext, block_id: BlockId, left: BlockId) {
    let block = &mut ctx[block_id];
    if let Some(old_left) = block.left {
        if left != old_left {
            let i = block.dominated.iter().position(|value| *value == old_left).unwrap();
            block.dominated.swap_remove(i);
        }
    }
    if !block.dominated.contains(&left) {
        block.dominated.push(left);
    }
    block.left = Some(left);
    assert!(block.right != Some(left));

    ctx[left].predecessor = vec![block_id];
    ctx[left].dominator = Some(block_id);
}

//replace all instructions by a false constraint, except for return instruction which is kept and zeroed
pub(super) fn short_circuit_instructions(
    ctx: &mut SsaContext,
    target: BlockId,
    instructions: &[NodeId],
) -> Vec<NodeId> {
    // short-circuit the return instruction (if it exists)
    zero_instructions(ctx, instructions, None);
    //nop and constrain false
    let unreachable_op = node::Operation::Constrain(ctx.zero(), None);
    let unreachable_ins = ctx.add_instruction(Instruction::new(
        unreachable_op,
        node::ObjectType::NotAnObject,
        Some(target),
    ));
    let nop = instructions[0];
    debug_assert_eq!(ctx.instruction(nop).operation, node::Operation::Nop);
    let mut stack = vec![nop, unreachable_ins];
    //return:
    for &i in instructions.iter() {
        if let Some(ins) = ctx.try_get_instruction(i) {
            if ins.operation.opcode() == Opcode::Return {
                stack.push(i);
                zero_instructions(ctx, &[i], None);
            }
        }
    }

    stack
}

//replace all instructions in the target block by a false constraint, except for return instruction
pub(super) fn short_circuit_inline(ctx: &mut SsaContext, target: BlockId) {
    let instructions = ctx[target].instructions.clone();
    let stack = short_circuit_instructions(ctx, target, &instructions);
    ctx[target].instructions = stack;
}

//Delete all instructions in the block
pub(super) fn short_circuit_block(ctx: &mut SsaContext, block_id: BlockId) {
    let instructions = ctx[block_id].instructions.clone();
    zero_instructions(ctx, &instructions, None);
}

//Delete instructions and replace them with zeros, except for return instruction which is kept with zeroed return values, and the avoid instruction
pub(super) fn zero_instructions(
    ctx: &mut SsaContext,
    instructions: &[NodeId],
    avoid: Option<&NodeId>,
) {
    let mut zeros = HashMap::new();
    let mut zero_keys = Vec::new();
    for i in instructions {
        let ins = ctx.instruction(*i);
        if ins.res_type != node::ObjectType::NotAnObject {
            zeros.insert(ins.res_type, ctx.zero_with_type(ins.res_type));
        } else if let node::Operation::Return(ret) = &ins.operation {
            for i in ret {
                if *i != NodeId::dummy() {
                    let typ = ctx.object_type(*i);
                    assert_ne!(typ, node::ObjectType::NotAnObject);
                    zero_keys.push(typ);
                } else {
                    zero_keys.push(node::ObjectType::NotAnObject);
                }
            }
        }
    }
    for k in zero_keys.iter() {
        let zero_id = if *k != node::ObjectType::NotAnObject {
            ctx.zero_with_type(*k)
        } else {
            NodeId::dummy()
        };
        zeros.insert(*k, zero_id);
    }

    for i in instructions.iter().filter(|x| Some(*x) != avoid) {
        let ins = ctx.instruction_mut(*i);
        if ins.res_type != node::ObjectType::NotAnObject {
            ins.mark = Mark::ReplaceWith(zeros[&ins.res_type]);
        } else if ins.operation.opcode() != Opcode::Nop {
            if ins.operation.opcode() == Opcode::Return {
                let vec = iter_extended::vecmap(&zero_keys, |x| zeros[x]);
                ins.operation = node::Operation::Return(vec);
            } else {
                ins.mark = Mark::Deleted;
            }
        }
    }
}

//merge subgraph from start to end in one block, excluding end
pub(super) fn merge_path(
    ctx: &mut SsaContext,
    start: BlockId,
    end: BlockId,
    assumption: Option<NodeId>,
) -> Result<VecDeque<BlockId>, RuntimeError> {
    let mut removed_blocks = VecDeque::new();
    if start != end {
        let mut next = start;
        let mut instructions = Vec::new();
        let mut block = &ctx[start];
        let mut short_circuit = BlockId::dummy();

        while next != end {
            if block.dominated.len() > 1 || block.right.is_some() {
                unreachable!("non sequential block sequence: {:?}", block);
            }
            block = &ctx[next];
            removed_blocks.push_back(next);

            if short_circuit.is_dummy() {
                if instructions.is_empty() {
                    instructions.extend(&block.instructions);
                } else {
                    let nonop = block.instructions.iter().filter(|&i| {
                        if let Some(ins) = ctx.try_get_instruction(*i) {
                            ins.operation.opcode() != Opcode::Nop
                        } else {
                            true
                        }
                    });
                    instructions.extend(nonop);
                }
            }

            if short_circuit.is_dummy() && block.is_short_circuit(ctx, assumption) {
                instructions.clear();
                instructions.extend(&block.instructions);
                short_circuit = block.id;
            }

            if let Some(left) = block.left {
                next = left;
            } else {
                if !end.is_dummy() {
                    unreachable!("cannot reach block {:?}", end);
                }
                next = BlockId::dummy();
            }
        }
        if !short_circuit.is_dummy() {
            for &b in &removed_blocks {
                if b != short_circuit {
                    short_circuit_block(ctx, b);
                }
            }
        }

        //we assign the concatenated list of instructions to the start block, using a CSE pass
        let mut modified = false;
        super::optimizations::cse_block(ctx, start, &mut instructions, &mut modified)?;
        //Wires start to end
        if !end.is_dummy() {
            rewire_block_left(ctx, start, end);
        } else {
            ctx[start].left = None;
        }
        removed_blocks.pop_front();
    }
    //housekeeping for the caller
    Ok(removed_blocks)
}

// retrieve written arrays along the CFG until we reach stop
pub(super) fn written_along(
    ctx: &SsaContext,
    block_id: BlockId,
    stop: BlockId,
    modified: &mut HashSet<ArrayId>,
) {
    if block_id == stop {
        return;
    }
    //process block
    modified.extend(ctx[block_id].written_arrays(ctx));

    //process next block
    if ctx[block_id].is_join() {
        written_along(ctx, ctx[block_id].left.unwrap(), stop, modified);
    } else if ctx[block_id].right.is_some() {
        let join = find_join(ctx, block_id);
        written_along(ctx, join, stop, modified);
    } else if let Some(left) = ctx[block_id].left {
        written_along(ctx, left, stop, modified);
    } else {
        unreachable!("could not reach stop block");
    }
}

// compute the exit block of a graph
pub(super) fn exit(ctx: &SsaContext, block_id: BlockId) -> BlockId {
    let block = &ctx[block_id];
    if let Some(left) = block.left {
        if left != BlockId::dummy() {
            return exit(ctx, left);
        }
    }
    block_id
}
