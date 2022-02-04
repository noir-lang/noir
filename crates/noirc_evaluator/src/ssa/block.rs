use super::{code_gen::IRGenerator, node};
use arena::Index;
use std::collections::{HashMap, VecDeque};

#[derive(PartialEq, Debug)]
pub enum BlockType {
    Normal,
    ForJoin,
}
#[derive(Debug)]
pub struct BasicBlock {
    pub idx: arena::Index,
    pub kind: BlockType,
    pub dominator: Option<arena::Index>, //direct dominator
    pub dominated: Vec<arena::Index>,    //dominated sons
    pub predecessor: Vec<arena::Index>,  //for computing the dominator tree
    pub left: Option<arena::Index>,      //sequential successor
    pub right: Option<arena::Index>,     //jump successor
    pub instructions: Vec<arena::Index>,
    pub value_array: HashMap<arena::Index, arena::Index>, //for generating the ssa form
}

impl BasicBlock {
    pub fn new(prev: arena::Index, kind: BlockType) -> BasicBlock {
        BasicBlock {
            idx: crate::ssa::code_gen::IRGenerator::dummy_id(),
            predecessor: vec![prev],
            left: None,
            right: None,
            instructions: Vec::new(),
            value_array: HashMap::new(),
            dominator: None,
            dominated: Vec::new(),
            kind,
        }
    }

    pub fn get_current_value(&self, idx: arena::Index) -> Option<arena::Index> {
        match self.value_array.get(&idx) {
            Some(cur_idx) => Some(*cur_idx),
            None => None,
        }
    }

    //When generating a new instance of a variable because of ssa, we update the value array
    //to link the two variables
    pub fn update_variable(&mut self, old_value: arena::Index, new_value: arena::Index) {
        self.value_array.insert(old_value, new_value);
    }

    pub fn get_first_instruction(&self) -> arena::Index {
        self.instructions[0]
    }

    pub fn is_join(&self) -> bool {
        self.kind == BlockType::ForJoin
    }
}

///////////

pub fn create_first_block(igen: &mut IRGenerator) {
    let mut first_block = BasicBlock::new(igen.dummy(), BlockType::Normal);
    let new_idx = igen.blocks.insert(first_block);
    let block2 = igen.blocks.get_mut(new_idx).unwrap(); //RIA..
    block2.idx = new_idx;
    igen.first_block = new_idx;
    igen.current_block = new_idx;
    igen.new_instruction(
        igen.dummy(),
        igen.dummy(),
        node::Operation::nop,
        node::ObjectType::none,
    );
}

//Creates a new sealed block (i.e whose predecessors are known)
//It is not suitable for the first block because it uses the current block.
pub fn new_sealed_block(igen: &mut IRGenerator, kind: BlockType) -> arena::Index {
    let new_block = BasicBlock::new(igen.current_block, kind);
    let new_idx = igen.blocks.insert(new_block);
    let block2 = igen.blocks.get_mut(new_idx).unwrap(); //RIA..
    block2.idx = new_idx;
    block2.dominator = Some(igen.current_block);
    igen.sealed_blocks.insert(new_idx);
    //update current block
    let cb = igen.get_block_mut(igen.current_block).unwrap();
    cb.left = Some(new_idx);
    igen.current_block = new_idx;
    igen.new_instruction(
        igen.dummy(),
        igen.dummy(),
        node::Operation::nop,
        node::ObjectType::none,
    );
    new_idx
}

//if left is true, the new block is left to the current block
pub fn new_unsealed_block(igen: &mut IRGenerator, kind: BlockType, left: bool) -> arena::Index {
    let new_idx = create_block(igen, kind);
    let block2 = igen.blocks.get_mut(new_idx).unwrap(); //RIA..
    block2.dominator = Some(igen.current_block);
    //update current block
    let cb = igen.get_block_mut(igen.current_block).unwrap();
    if left {
        cb.left = Some(new_idx);
    } else {
        cb.right = Some(new_idx);
    }
    igen.current_block = new_idx;
    igen.new_instruction(
        igen.dummy(),
        igen.dummy(),
        node::Operation::nop,
        node::ObjectType::none,
    );
    new_idx
}

//create a block and sets its id, but do not update current block, and do not add dummy instruction!
pub fn create_block(igen: &mut IRGenerator, kind: BlockType) -> arena::Index {
    let new_block = BasicBlock::new(igen.current_block, kind);
    let new_idx = igen.blocks.insert(new_block);
    let block2 = igen.blocks.get_mut(new_idx).unwrap(); //RIA..
    block2.idx = new_idx;
    new_idx
}

//link the current block to the target block so that current block becomes its target
pub fn link_with_target(
    igen: &mut IRGenerator,
    target: arena::Index,
    left: Option<arena::Index>,
    right: Option<arena::Index>,
) {
    if let Some(target_block) = igen.get_block_mut(target) {
        target_block.right = right;
        target_block.left = left;
        //TODO should also update the last instruction rhs to the first instruction of the current block  -- TODOshoud we do it here??
        if let Some(right_uw) = right {
            let rb = igen.get_block_mut(right_uw);
            rb.unwrap().dominator = Some(target);
        }
        if let Some(left_uw) = left {
            let lb = igen.get_block_mut(left_uw);
            lb.unwrap().dominator = Some(target);
        }
    }
}

pub fn compute_dom(igen: &mut IRGenerator) {
    let mut dominator_link: HashMap<arena::Index, Vec<arena::Index>> = HashMap::new();
    for (idx, block) in &igen.blocks {
        if let Some(dom) = block.dominator {
            if dominator_link.contains_key(&dom) {
                let mut v = dominator_link[&dom].clone(); //TODO can we avoid it?
                v.push(idx);
                dominator_link.insert(dom, v);
            } else {
                dominator_link.insert(dom, [idx].to_vec());
            }
            // dom_block.dominated.push(idx);
        }
    }
    //RIA
    for (master, svec) in dominator_link {
        let dom_b = igen.get_block_mut(master).unwrap();
        for slave in svec {
            dom_b.dominated.push(slave);
        }
    }
}

//breadth-first traversal of the CFG, from start, until we reach stop
pub fn bfs(start: Index, stop: Index, eval: &IRGenerator) -> Vec<Index> {
    let mut result = vec![start]; //list of blocks in the visited subgraph
    let mut queue: VecDeque<Index> = VecDeque::new(); //Queue of elements to visit
    queue.push_back(start);
    while !queue.is_empty() {
        let b = queue.pop_front().unwrap();
        if let Some(block) = eval.try_get_block(b) {
            if let Some(left) = block.left {
                if left != stop && !result.contains(&left) {
                    result.push(left);
                    queue.push_back(left);
                }
            }
            if let Some(right) = block.right {
                if right != stop && !result.contains(&right) {
                    result.push(right);
                    queue.push_back(right);
                }
            }
        }
    }
    result
}
