use acvm::acir::brillig_vm::Opcode as BrilligOpcode;
use std::collections::{HashMap, HashSet};

use crate::{
    brillig::Brillig,
    ssa_refactor::ir::{basic_block::BasicBlockId, function::FunctionId},
};

#[derive(Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code.
/// Currently it is just the brillig bytecode of the function.
pub(crate) struct BrilligArtifact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
    /// The set of jumps that need to have their locations
    /// resolved.
    unresolved_jumps_or_calls: Vec<(JumpInstructionPosition, UnresolvedLocation)>,
    /// A map of labels to their position in byte code.
    labels: HashMap<Label, OpcodeLocation>,
    /// functions called that need to be resolved
    functions_to_process: HashSet<FunctionId>,
    ///function id
    function_id: FunctionId,
}

/// A pointer to a location in the opcode.
pub(crate) type OpcodeLocation = usize;
/// An identifier for a location in the code.
///
/// It is assumed that an entity will keep a map
/// of labels to Opcode locations.
pub(crate) type Label = String;
/// Pointer to a unresolved Jump instruction in
/// the bytecode.
pub(crate) type JumpInstructionPosition = OpcodeLocation;

/// When constructing the bytecode, there may be instructions
/// which require one to jump to a specific region of code (function)
/// or a position relative to the current instruction.
///
/// The position of a function cannot always be known
/// at this point in time, so Jumps are unresolved
/// until all functions/all of the bytecode has been processed.
/// `Label` is used as the jump location and once all of the bytecode
/// has been processed, the jumps are resolved using a map from Labels
/// to their position in the bytecode.
///
/// Sometimes the jump destination may be relative to the jump instruction.
/// Since the absolute position in the bytecode cannot be known until
/// all internal and external functions have been linked, jumps of this
/// nature cannot be fully resolved while building the bytecode either.
/// We add relative jumps into the `Relative` variant of this enum.
#[derive(Debug, Clone)]
pub(crate) enum UnresolvedLocation {
    Label(String),
    Relative(i32),
}

impl BrilligArtifact {
    pub(crate) fn new(func_id: FunctionId) -> BrilligArtifact {
        BrilligArtifact {
            byte_code: Vec::new(),
            unresolved_jumps_or_calls: Vec::new(),
            labels: HashMap::new(),
            functions_to_process: HashSet::new(),
            function_id: func_id,
        }
    }

    /// Link two Brillig artifacts together and resolve all unresolved jump instructions.
    pub(crate) fn link(&mut self, id: FunctionId, brillig: &Brillig) -> Vec<BrilligOpcode> {
        let obj = &brillig[id];
        self.append_artifact(obj);
        self.push_opcode(BrilligOpcode::Stop);
        let mut queue: Vec<FunctionId> = obj.functions_to_process.clone().into_iter().collect();
        while let Some(func) = queue.pop() {
            dbg!(&brillig.function_label(func));
            if !self.labels.contains_key(&brillig.function_label(func)) {
                let obj = &brillig[func];
                self.append_artifact(obj);
                self.byte_code.pop();
                self.push_opcode(BrilligOpcode::Return);
                let mut functions: Vec<FunctionId> =
                    obj.functions_to_process.clone().into_iter().collect();
                queue.append(&mut functions);
            }
        }

        self.resolve_jumps();
        self.byte_code.clone()
    }

    pub(crate) fn block_label(&self, block_id: BasicBlockId) -> String {
        self.function_id.to_string() + "-" + &block_id.to_string()
    }
    /// Link with an external brillig artifact.
    ///
    /// This method will offset the positions in the Brillig artifact to
    /// account for the fact that it is being appended to the end of this
    /// Brillig artifact (self).
    fn append_artifact(&mut self, obj: &BrilligArtifact) {
        let offset = self.index_of_next_opcode();
        for (jump_label, jump_location) in &obj.unresolved_jumps_or_calls {
            self.unresolved_jumps_or_calls.push((jump_label + offset, jump_location.clone()));
        }

        for (label_id, position_in_bytecode) in &obj.labels {
            self.labels.insert(label_id.clone(), position_in_bytecode + offset);
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.byte_code.push(opcode);
    }

    /// Adds a unresolved jump to be fixed at the end of bytecode processing.
    pub(crate) fn add_unresolved_call(
        &mut self,
        call_instruction: BrilligOpcode,
        destination: UnresolvedLocation,
        func_id: FunctionId,
    ) {
        assert!(
            Self::is_call_instruction(&call_instruction),
            "expected a call instruction, but found {call_instruction:?}"
        );

        self.unresolved_jumps_or_calls.push((self.index_of_next_opcode(), destination));
        self.push_opcode(call_instruction);
        self.functions_to_process.insert(func_id);
    }

    /// Adds a unresolved jump to be fixed at the end of bytecode processing.
    pub(crate) fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedLocation,
    ) {
        assert!(
            Self::is_jmp_instruction(&jmp_instruction),
            "expected a jump instruction, but found {jmp_instruction:?}"
        );

        self.unresolved_jumps_or_calls.push((self.index_of_next_opcode(), destination));
        self.push_opcode(jmp_instruction);
    }

    /// Returns true if the opcode is a jump instruction
    fn is_jmp_instruction(instruction: &BrilligOpcode) -> bool {
        matches!(
            instruction,
            BrilligOpcode::JumpIfNot { .. }
                | BrilligOpcode::JumpIf { .. }
                | BrilligOpcode::Jump { .. }
        )
    }

    /// Returns true if the opcode is a call instruction
    fn is_call_instruction(instruction: &BrilligOpcode) -> bool {
        matches!(instruction, BrilligOpcode::Call { .. })
    }

    /// Adds a label in the bytecode to specify where this block's
    /// opcodes will start.
    pub(crate) fn add_label_at_position(&mut self, label: String, position: OpcodeLocation) {
        let old_value = self.labels.insert(label.clone(), position);
        assert!(
            old_value.is_none(),
            "overwriting label {label}. old_value = {old_value:?}, new_value = {position}"
        );
    }

    /// Returns the index of the next opcode.
    ///
    /// This is useful for labelling regions of code
    /// before you have generated the opcodes for the region.
    pub(crate) fn index_of_next_opcode(&self) -> OpcodeLocation {
        self.byte_code.len()
    }

    /// Resolves all of the unresolved jumps in the program.
    ///
    /// Note: This should only be called once all blocks are processed and
    /// linkage with other bytecode has happened.
    fn resolve_jumps(&mut self) {
        for (location_of_jump, unresolved_location) in &self.unresolved_jumps_or_calls {
            let resolved_location = match unresolved_location {
                UnresolvedLocation::Label(label) => self.labels[label],
                UnresolvedLocation::Relative(offset) => {
                    (offset + *location_of_jump as i32) as usize
                }
            };

            let jump_instruction = self.byte_code[*location_of_jump].clone();
            match jump_instruction {
                BrilligOpcode::Jump { location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::Jump { location: resolved_location };
                }
                BrilligOpcode::JumpIfNot { condition, location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::JumpIfNot { condition, location: resolved_location };
                }
                BrilligOpcode::JumpIf { condition, location } => {
                    assert_eq!(location, 0, "location is not zero, which means that the jump label does not need resolving");

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::JumpIf { condition, location: resolved_location };
                }
                BrilligOpcode::Call { location } => {
                    assert_eq!(
                        location, 0,
                        "location is not zero, which means that the label does not need resolving"
                    );
                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::Call { location: resolved_location };
                }
                _ => unreachable!(
                    "all labels should point to a jump or a call instruction in the bytecode"
                ),
            }
        }
    }
}
