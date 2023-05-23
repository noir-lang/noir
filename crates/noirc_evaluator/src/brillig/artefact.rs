use std::collections::{HashSet, HashMap};

use crate::ssa_refactor::ir::{function::FunctionId, basic_block::BasicBlockId};
use acvm::acir::brillig_bytecode::{
    Opcode as BrilligOpcode
};
#[derive(Default, Debug, Clone)]
pub(crate) struct Artefact {
    functions_to_process: HashSet<FunctionId>,
    pub(crate) byte_code: Vec<BrilligOpcode>,
    to_fix: Vec<(usize, BasicBlockId)>,
    blocks: HashMap<BasicBlockId, usize>, //processed blocks and their entry point
}

impl Artefact {
    pub fn fix_jump(&mut self, destination: BasicBlockId) {
        self.to_fix.push((self.codelen(), destination));
    }

    pub fn start(&mut self, block: BasicBlockId) {
        self.blocks.insert(block, self.codelen());
    }

    pub fn codelen(&self) -> usize {
        self.byte_code.len()
    }
}