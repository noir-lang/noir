use std::collections::HashMap;

use acvm::acir::brillig_vm::Opcode as BrilligOpcode;

use crate::ssa_refactor::ir::basic_block::BasicBlockId;

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code
/// Currently it is just the brillig bytecode of the function
pub(crate) struct BrilligArtifact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
    to_fix: Vec<(usize, BasicBlockId)>,
    blocks: HashMap<BasicBlockId, usize>, //processed blocks and their entry point
}

impl BrilligArtifact {
    // Link some compiled brillig bytecode with its referenced artifacts
    pub(crate) fn link(&mut self, obj: &BrilligArtifact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        self.fix_jumps();
        self.byte_code.clone()
    }

    // Link with a brillig artifact
    fn link_with(&mut self, obj: &BrilligArtifact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }
        let offset = self.code_len();
        for i in &obj.to_fix {
            self.to_fix.push((i.0 + offset, i.1));
        }
        for i in &obj.blocks {
            self.blocks.insert(*i.0, i.1 + offset);
        }
        self.byte_code.extend_from_slice(&obj.byte_code);
    }
    
    pub(crate) fn fix_jump(&mut self, destination: BasicBlockId) {
        self.to_fix.push((self.code_len(), destination));
     }

    pub(crate) fn start(&mut self, block: BasicBlockId) {
        self.blocks.insert(block, self.code_len());
     }

     pub(crate) fn code_len(&self) -> usize {
        self.byte_code.len()
    }

    fn fix_jumps(&mut self) {
        for (jump, block) in &self.to_fix {
            match self.byte_code[*jump] {
                BrilligOpcode::Jump { location } => {
                    assert_eq!(location, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] = BrilligOpcode::Jump { location: current };
                }
                BrilligOpcode::JumpIfNot { condition, location } => {
                    let current = if location == 0 {
                        self.blocks[block]
                    } else {
                        location + self.byte_code.len()
                    };
                    self.byte_code[*jump] =
                        BrilligOpcode::JumpIfNot { condition, location: current };
                }
                BrilligOpcode::JumpIf { condition, location } => {
                    assert_eq!(location, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] =
                        BrilligOpcode::JumpIf { condition, location: current };
                }
                _ => unreachable!(),
            }
        }
    }
}
