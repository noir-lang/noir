use acvm::acir::brillig_vm::{Opcode as BrilligOpcode, RegisterIndex, Value};
use std::collections::HashMap;

use crate::brillig::brillig_ir::ReservedRegisters;

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code.
/// Currently it is just the brillig bytecode of the function.
pub(crate) struct BrilligArtifact {
    pub(crate) byte_code: Vec<BrilligOpcode>,
    /// The set of jumps that need to have their locations
    /// resolved.
    unresolved_jumps: Vec<(JumpInstructionPosition, UnresolvedJumpLocation)>,
    /// A map of labels to their position in byte code.
    labels: HashMap<Label, OpcodeLocation>,
    /// Set of labels which are external to the bytecode.
    ///
    /// This will most commonly contain the labels of functions
    /// which are defined in other bytecode, that this bytecode has called.
    /// TODO: perhaps we should combine this with the `unresolved_jumps` field
    /// TODO: and have an enum which indicates whether the jump is internal or external
    unresolved_external_call_labels: Vec<(JumpInstructionPosition, UnresolvedJumpLocation)>,
    /// The number of return values that this function will return.
    number_of_return_parameters: usize,

    /// The number of arguments that this function will take.
    number_of_arguments: usize,
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
///
/// The position of a function cannot always be known
/// at this point in time, so Jumps are unresolved
/// until all functions/all of the bytecode has been processed.
/// `Label` is used as the jump location and once all of the bytecode
/// has been processed, the jumps are resolved using a map from Labels
/// to their position in the bytecode.
pub(crate) type UnresolvedJumpLocation = Label;

impl BrilligArtifact {
    /// Initialize an artifact with the number of arguments and return parameters
    pub(crate) fn new(
        number_of_arguments: usize,
        number_of_return_parameters: usize,
    ) -> BrilligArtifact {
        BrilligArtifact {
            byte_code: Vec::new(),
            unresolved_jumps: Vec::new(),
            labels: HashMap::new(),
            unresolved_external_call_labels: Vec::new(),
            number_of_return_parameters,
            number_of_arguments,
        }
    }

    /// Creates an entry point artifact wrapping the bytecode of the function provided.
    pub(crate) fn to_entry_point_artifact(artifact: &BrilligArtifact) -> BrilligArtifact {
        let mut entry_point_artifact = BrilligArtifact::new(
            artifact.number_of_arguments,
            artifact.number_of_return_parameters,
        );
        entry_point_artifact.entry_point_instruction();

        entry_point_artifact.add_unresolved_jumps_and_calls(artifact);
        entry_point_artifact.byte_code.extend_from_slice(&artifact.byte_code);

        entry_point_artifact.exit_point_instruction();
        entry_point_artifact
    }

    /// Resolves all jumps and generates the final bytecode
    pub(crate) fn finish(mut self) -> Vec<BrilligOpcode> {
        self.resolve_jumps();
        self.byte_code
    }

    /// Adds the instructions needed to handle entry point parameters
    ///
    /// And sets the starting value of the reserved registers
    fn entry_point_instruction(&mut self) {
        // Translate the inputs by the reserved registers offset
        for i in (0..self.number_of_arguments).rev() {
            self.byte_code.push(BrilligOpcode::Mov {
                destination: ReservedRegisters::user_register_index(i),
                source: RegisterIndex::from(i),
            });
        }

        // Set the initial value of the stack pointer register
        self.byte_code.push(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: Value::from(0_usize),
        });
    }

    /// Adds the instructions needed to handle return parameters
    fn exit_point_instruction(&mut self) {
        // We want all functions to follow the calling convention of returning
        // their results in the first `n` registers. So we modify the bytecode of the
        // function to move the return values to the first `n` registers once completed.
        //
        // Swap the stop opcode with a jump to the exit point section

        let stop_position =
            self.byte_code.iter().position(|opcode| matches!(opcode, BrilligOpcode::Stop));

        let stop_position = stop_position.expect("expected a stop opcode");

        let exit_section = self.index_of_next_opcode();
        self.byte_code[stop_position] = BrilligOpcode::Jump { location: exit_section };

        // TODO: this _seems_ like an abstraction leak, we need to know about the reserved
        // TODO: registers in order to do this.
        // Move the results to registers 0..n
        for i in 0..self.number_of_return_parameters {
            self.push_opcode(BrilligOpcode::Mov {
                destination: i.into(),
                source: ReservedRegisters::user_register_index(i),
            });
        }
        self.push_opcode(BrilligOpcode::Stop);
    }

    /// Gets the first unresolved function call of this artifact.
    pub(crate) fn first_unresolved_function_call(&self) -> Option<Label> {
        self.unresolved_external_call_labels.first().map(|(_, label)| label.clone())
    }

    /// Link with an external brillig artifact called from this artifact.
    ///
    /// This method will offset the positions in the Brillig artifact to
    /// account for the fact that it is being appended to the end of this
    /// Brillig artifact (self).
    pub(crate) fn link_with(&mut self, func_label: Label, obj: &BrilligArtifact) {
        // First get the unresolved function call we are linking against.
        let unresolved_external_call = self
            .unresolved_external_call_labels
            .iter()
            .position(|(_, label)| label == &func_label)
            .expect("Trying to link with an unknown function");

        // Remove it from the list, since it is now resolved.
        let unresolved_external_call =
            self.unresolved_external_call_labels.remove(unresolved_external_call);
        // Transform the unresolved external call into an unresolved jump to the linked function.
        self.unresolved_jumps.push(unresolved_external_call);

        // Add the unresolved jumps of the linked function to this artifact.
        self.add_unresolved_jumps_and_calls(obj);

        let mut byte_code = obj.byte_code.clone();

        // Replace STOP with RETURN because this is not the end of the program now.
        let stop_position = byte_code
            .iter()
            .position(|opcode| matches!(opcode, BrilligOpcode::Stop))
            .expect("Trying to link with a function that does not have a stop opcode");

        byte_code[stop_position] = BrilligOpcode::Return;

        self.byte_code.append(&mut byte_code);
    }

    /// Adds unresolved jumps & function calls from another artifact offset by the current opcode count in the artifact.
    fn add_unresolved_jumps_and_calls(&mut self, obj: &BrilligArtifact) {
        let offset = self.index_of_next_opcode();
        for (jump_label, jump_location) in &obj.unresolved_jumps {
            self.unresolved_jumps.push((jump_label + offset, jump_location.clone()));
        }

        for (label_id, position_in_bytecode) in &obj.labels {
            let old_value = self.labels.insert(label_id.clone(), position_in_bytecode + offset);
            assert!(old_value.is_none(), "overwriting label {label_id} {old_value:?}");
        }

        for (position_in_bytecode, label_id) in &obj.unresolved_external_call_labels {
            self.unresolved_external_call_labels
                .push((position_in_bytecode + offset, label_id.clone()));
        }
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.byte_code.push(opcode);
    }

    /// Adds a unresolved jump to be fixed at the end of bytecode processing.
    pub(crate) fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        assert!(
            Self::is_jmp_instruction(&jmp_instruction),
            "expected a jump instruction, but found {jmp_instruction:?}"
        );

        self.unresolved_jumps.push((self.index_of_next_opcode(), destination));
        self.push_opcode(jmp_instruction);
    }
    /// Adds a unresolved external call that will be fixed once linking has been done.
    pub(crate) fn add_unresolved_external_call(
        &mut self,
        call_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        // TODO: Add a check to ensure that the opcode is a call instruction

        self.unresolved_external_call_labels.push((self.index_of_next_opcode(), destination));
        self.push_opcode(call_instruction);
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
        for (location_of_jump, unresolved_location) in &self.unresolved_jumps {
            let resolved_location = self.labels[unresolved_location];

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
                    assert_eq!(location, 0, "location is not zero, which means that the call label does not need resolving");

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::Call { location: resolved_location };
                }
                _ => unreachable!(
                    "all jump labels should point to a jump instruction in the bytecode"
                ),
            }
        }
    }
}
