use super::{
    conditional::AssumptionId,
    context::SsaContext,
    node::{self, NodeId},
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(PartialEq, Debug, Clone)]
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
pub fn create_block<'a>(ctx: &'a mut SsaContext, kind: BlockType) -> &'a mut BasicBlock {
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
            // dom_block.dominated.push(idx);
        }
    }
    //RIA
    for (master, svec) in dominator_link {
        if let Some(dom_b) = ctx.try_get_block_mut(master) {
            dom_b.dominated.clear();
            for slave in svec {
                dom_b.dominated.push(slave);
            }
        }
    }
}

pub fn compute_sub_dom(ctx: &mut SsaContext, blocks: &[BlockId]) {
    let mut dominator_link = HashMap::new();

    for &block_id in blocks {
        let block = &ctx[block_id];
        if let Some(dom) = block.dominator {
            dominator_link.entry(dom).or_insert_with(Vec::new).push(block.id);
        }
    }
    //RIA
    for (master, svec) in dominator_link {
        let dom_b = &mut ctx[master];
        for slave in svec {
            dom_b.dominated.push(slave);
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
                if block_id != BlockId::dummy() && !result.contains(&block_id) {
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

pub fn find_join(ctx: &SsaContext, b1: BlockId, b2: BlockId) -> BlockId {
    if b1 == b2 {
        return b1;
    }
    let mut process_left = vec![b1];
    let mut process_right = vec![b2];
    let mut descendants = HashMap::new();
    while !process_left.is_empty() || !process_right.is_empty() {
        if let Some(block) = process_son(ctx, true, &mut descendants, &mut process_left) {
            return block;
        }
        if let Some(block) = process_son(ctx, false, &mut descendants, &mut process_right) {
            return block;
        }
    }
    unreachable!("no join");
}

fn process_son(
    ctx: &SsaContext,
    left: bool,
    descendants: &mut HashMap<BlockId, bool>,
    to_process: &mut Vec<BlockId>,
) -> Option<BlockId> {
    if let Some(block) = to_process.pop() {
        if descendants.contains_key(&block) {
            if descendants[&block] && left {
                return None; //cycle
            }
            if !descendants[&block] && !left {
                return None; //cycle
            }
            return Some(block);
        }
        descendants.insert(block, left);
        if let Some(left) = ctx[block].left {
            to_process.push(left);
        }
        if let Some(right) = ctx[block].right {
            to_process.push(right);
        }
    }
    None
}

//Set left as the left block of block_id
pub fn rewire_block_left(ctx: &mut SsaContext, block_id: BlockId, left: BlockId) {
    let block = &mut ctx[block_id];
    if let Some(old_left) = block.left {
        if left == old_left {
            return;
        }
        let i = block.dominated.iter().position(|value| *value == old_left).unwrap();
        block.dominated.swap_remove(i);
    }
    block.left = Some(left);
    assert!(block.right != Some(left));
    block.dominated.push(left);
    ctx[left].predecessor.push(block_id);
    if ctx[left].predecessor.len() == 1 {
        ctx[left].dominator = Some(block_id);
    }
}

//merge subgraph from start to end in one block, excluding end
pub fn merge_path(ctx: &mut SsaContext, start: BlockId, end: BlockId) -> VecDeque<BlockId> {
    let mut removed_blocks = VecDeque::new();
    if start != end {
        let mut next = start;
        let mut instructions = Vec::new();
        let mut block = &ctx[start];
        while next != end {
            if block.dominated.len() > 1 || block.right.is_some() {
                dbg!(&block);
                unreachable!("non sequential block sequence");
            }
            block = &ctx[next];
            removed_blocks.push_back(next);
            instructions.extend(&block.instructions);
            if let Some(left) = block.left {
                next = left;
            } else {
                unreachable!("cannot reach block {:?}", end);
            }
        }
        //we assign the concatened list of instructions to the start block, using a CSE pass
        let mut modified = false;
        super::optim::cse_block(ctx, start, &mut instructions, &mut modified).unwrap();
        //Wires start to end
        rewire_block_left(ctx, start, end);
        removed_blocks.pop_front();
    }
    //housekeeping for the caller
    removed_blocks
}
