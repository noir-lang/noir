use super::{
    conditional::AssumptionId,
    context::SsaContext,
    node::{self, Instruction, Mark, NodeId, Opcode},
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BlockType {
    Normal,
    ForJoin,
    IfJoin,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub arena::Index);

impl BlockId {
    pub fn dummy() -> BlockId {
        BlockId(SsaContext::dummy_id())
    }
    pub fn is_dummy(&self) -> bool {
        *self == BlockId::dummy()
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub id: BlockId,
    pub kind: BlockType,
    pub dominator: Option<BlockId>, //direct dominator
    pub dominated: Vec<BlockId>,    //dominated sons
    pub predecessor: Vec<BlockId>,  //for computing the dominator tree
    pub left: Option<BlockId>,      //sequential successor
    pub right: Option<BlockId>,     //jump successor
    pub instructions: Vec<NodeId>,
    pub value_map: HashMap<NodeId, NodeId>, //for generating the ssa form
    pub assumption: AssumptionId,
}

impl BasicBlock {
    pub fn new(prev: BlockId, kind: BlockType) -> BasicBlock {
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

    pub fn get_current_value(&self, id: NodeId) -> Option<NodeId> {
        self.value_map.get(&id).copied()
    }

    //When generating a new instance of a variable because of ssa, we update the value array
    //to link the two variables
    pub fn update_variable(&mut self, old_value: NodeId, new_value: NodeId) {
        self.value_map.insert(old_value, new_value);
    }

    pub fn get_first_instruction(&self) -> NodeId {
        self.instructions[0]
    }

    pub fn is_join(&self) -> bool {
        self.kind == BlockType::ForJoin
    }

    //Create the first block for a CFG
    pub fn create_cfg(ctx: &mut SsaContext) -> BlockId {
        let root_block = BasicBlock::new(BlockId::dummy(), BlockType::Normal);
        let root_block = ctx.insert_block(root_block);
        root_block.predecessor = Vec::new();
        let root_id = root_block.id;
        ctx.current_block = root_id;
        ctx.sealed_blocks.insert(root_id);
        ctx.new_instruction(node::Operation::Nop, node::ObjectType::NotAnObject).unwrap();
        root_id
    }

    pub fn written_arrays(&self, ctx: &SsaContext) -> HashSet<super::mem::ArrayId> {
        let mut result = HashSet::new();
        for i in &self.instructions {
            if let Some(node::Instruction {
                operation: node::Operation::Store { array_id: x, .. },
                ..
            }) = ctx.try_get_instruction(*i)
            {
                result.insert(*x);
            }
        }
        result
    }

    //Returns true if the block is a short-circuit
    fn is_short_circuit(&self, ctx: &SsaContext, assumption: Option<NodeId>) -> bool {
        if let Some(ass_value) = assumption {
            if self.instructions.len() >= 3 {
                if let Some(Instruction { operation, .. }) =
                    ctx.try_get_instruction(self.instructions[1])
                {
                    if *operation
                        == (node::Operation::Cond {
                            condition: ass_value,
                            val_true: ctx.zero(),
                            val_false: ctx.one(),
                        })
                    {
                        if let Some(Instruction { operation, .. }) =
                            ctx.try_get_instruction(self.instructions[2])
                        {
                            if *operation == node::Operation::Constrain(self.instructions[1], None)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

pub fn create_first_block(ctx: &mut SsaContext) {
    ctx.first_block = BasicBlock::create_cfg(ctx);
}

//Creates a new sealed block (i.e whose predecessors are known)
//It is not suitable for the first block because it uses the current block.
//if left is true, the new block is left to the current block
pub fn new_sealed_block(ctx: &mut SsaContext, kind: BlockType, left: bool) -> BlockId {
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
pub fn new_unsealed_block(ctx: &mut SsaContext, kind: BlockType, left: bool) -> BlockId {
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
pub fn create_block(ctx: &mut SsaContext, kind: BlockType) -> &mut BasicBlock {
    let new_block = BasicBlock::new(ctx.current_block, kind);
    ctx.insert_block(new_block)
}

//link the current block to the target block so that current block becomes its target
pub fn link_with_target(
    ctx: &mut SsaContext,
    target: BlockId,
    left: Option<BlockId>,
    right: Option<BlockId>,
) {
    if let Some(target_block) = ctx.try_get_block_mut(target) {
        target_block.right = right;
        target_block.left = left;
        //TODO should also update the last instruction rhs to the first instruction of the current block  -- TODOshoud we do it here??
        if let Some(right_uw) = right {
            ctx[right_uw].dominator = Some(target);
        }
        if let Some(left_uw) = left {
            ctx[left_uw].dominator = Some(target);
        }
    }
}

pub fn compute_dom(ctx: &mut SsaContext) {
    let mut dominator_link = HashMap::new();

    for block in ctx.iter_blocks() {
        if let Some(dom) = block.dominator {
            dominator_link.entry(dom).or_insert_with(Vec::new).push(block.id);
        }
    }
    for (master, svec) in dominator_link {
        if let Some(dom_b) = ctx.try_get_block_mut(master) {
            dom_b.dominated.clear();
            for slave in svec {
                dom_b.dominated.push(slave);
            }
        }
    }
}

//breadth-first traversal of the CFG, from start, until we reach stop
pub fn bfs(start: BlockId, stop: Option<BlockId>, ctx: &SsaContext) -> Vec<BlockId> {
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
pub fn find_join(ctx: &SsaContext, block_id: BlockId) -> BlockId {
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
pub fn rewire_block_left(ctx: &mut SsaContext, block_id: BlockId, left: BlockId) {
    let block = &mut ctx[block_id];
    if !block.dominated.contains(&left) {
        block.dominated.push(left);
    }
    if let Some(old_left) = block.left {
        if left == old_left {
            return;
        }
        let i = block.dominated.iter().position(|value| *value == old_left).unwrap();
        block.dominated.swap_remove(i);
    }
    block.left = Some(left);
    assert!(block.right != Some(left));

    ctx[left].predecessor.push(block_id);
    if ctx[left].predecessor.len() == 1 {
        ctx[left].dominator = Some(block_id);
    }
}

//Delete all instructions in the block
pub fn short_circuit_block(ctx: &mut SsaContext, block_id: BlockId) {
    let instructions = ctx[block_id].instructions.clone();
    short_circuit_instructions(ctx, &instructions);
}

pub fn short_circuit_instructions(ctx: &mut SsaContext, instructions: &Vec<NodeId>) {
    let mut zeros = HashMap::new();
    for i in instructions {
        let ins = ctx.get_instruction(*i);
        if ins.res_type != node::ObjectType::NotAnObject {
            zeros.insert(ins.res_type, ctx.zero_with_type(ins.res_type));
        }
    }
    for i in instructions {
        let ins = ctx.get_mut_instruction(*i);
        if ins.res_type != node::ObjectType::NotAnObject {
            ins.mark = Mark::ReplaceWith(zeros[&ins.res_type]);
        } else if ins.operation.opcode() != Opcode::Nop {
            ins.mark = Mark::Deleted;
        }
    }
}

//merge subgraph from start to end in one block, excluding end
pub fn merge_path(
    ctx: &mut SsaContext,
    start: BlockId,
    end: BlockId,
    assumption: Option<NodeId>,
) -> VecDeque<BlockId> {
    let mut removed_blocks = VecDeque::new();
    if start != end {
        let mut next = start;
        let mut instructions = Vec::new();
        let mut block = &ctx[start];
        let mut short_circuit = BlockId::dummy();

        while next != end {
            if block.dominated.len() > 1 || block.right.is_some() {
                dbg!(&block);
                unreachable!("non sequential block sequence");
            }
            block = &ctx[next];
            removed_blocks.push_back(next);

            if short_circuit.is_dummy() {
                instructions.extend(&block.instructions);
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

        //we assign the concatened list of instructions to the start block, using a CSE pass
        let mut modified = false;
        super::optim::cse_block(ctx, start, &mut instructions, &mut modified).unwrap();
        //Wires start to end
        if !end.is_dummy() {
            rewire_block_left(ctx, start, end);
        } else {
            ctx[start].left = None;
        }
        removed_blocks.pop_front();
    }
    //housekeeping for the caller
    removed_blocks
}
