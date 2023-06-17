use acvm::acir::brillig_vm::{
    BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, Value,
};
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

    /// Links Brillig artifact and resolve all unresolved jump instructions.
    ///
    /// Current usage of this method, does not link two independent Brillig artifacts.
    /// `Self` at this point in time
    ///
    /// TODO: This method could be renamed to `link_and_resolve_jumps`
    /// TODO: We could make this consume self, so the Clone is explicitly
    /// TODO: done by the caller
    pub(crate) fn link(artifact_to_append: &BrilligArtifact) -> Vec<BrilligOpcode> {
        let mut linked_artifact = BrilligArtifact::default();

        linked_artifact.entry_point_instruction(artifact_to_append.number_of_arguments);
        // First we append the artifact to the end of the current artifact
        // Updating the offsets of the appended artefact, so that the jumps
        // are still correct.
        linked_artifact.append_artifact(artifact_to_append);

        linked_artifact.exit_point_instruction(artifact_to_append.number_of_return_parameters);

        linked_artifact.resolve_jumps();

        linked_artifact.byte_code.clone()
    }

    /// Adds the instructions needed to handle entry point parameters
    ///
    /// And sets the starting value of the reserved registers
    pub(crate) fn entry_point_instruction(&mut self, num_arguments: usize) {
        // Translate the inputs by the reserved registers offset
        for i in (0..num_arguments).rev() {
            self.byte_code.push(BrilligOpcode::Mov {
                destination: ReservedRegisters::user_register_index(i),
                source: RegisterIndex::from(i),
            })
        }

        // Set the initial value of the stack pointer register
        self.byte_code.push(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: Value::from(0_usize),
        });
    }

    /// Adds the instructions needed to handle return parameters
    pub(crate) fn exit_point_instruction(&mut self, num_return_parameters: usize) {
        // We want all functions to follow the calling convention of returning
        // their results in the first `n` registers. So we modify the bytecode of the
        // function to move the return values to the first `n` registers once completed.
        //
        // Remove the ending stop
        // TODO: Shouldn't this be the case when we process a terminator instruction?
        // TODO: If so, then entry_point_instruction and exit_point_instruction should be
        // TODO put in brillig_gen.
        // TODO: entry_point is called when we process a function, and exit_point is called
        // TODO when we process a terminator instruction.
        let expected_stop = self.byte_code.pop().expect("expected at least one opcode");
        assert_eq!(expected_stop, BrilligOpcode::Stop, "expected a stop code");

        // TODO: this _seems_ like an abstraction leak, we need to know about the reserved
        // TODO: registers in order to do this.
        // Move the results to registers 0..n
        for i in 0..num_return_parameters {
            self.push_opcode(BrilligOpcode::Mov {
                destination: i.into(),
                source: ReservedRegisters::user_register_index(i),
            });
        }
        self.push_opcode(BrilligOpcode::Stop);
    }

    /// Link with an external brillig artifact.
    ///
    /// This method will offset the positions in the Brillig artifact to
    /// account for the fact that it is being appended to the end of this
    /// Brillig artifact (self).
    fn append_artifact(&mut self, obj: &BrilligArtifact) {
        let offset = self.index_of_next_opcode();
        for (jump_label, jump_location) in &obj.unresolved_jumps {
            self.unresolved_jumps.push((jump_label + offset, jump_location.clone()));
        }

        for (label_id, position_in_bytecode) in &obj.labels {
            let old_value = self.labels.insert(label_id.clone(), position_in_bytecode + offset);
            assert!(old_value.is_none(), "overwriting label {label_id} {old_value:?}");
        }

        self.byte_code.extend_from_slice(&obj.byte_code);
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
                _ => unreachable!(
                    "all jump labels should point to a jump instruction in the bytecode"
                ),
            }
        }
    }

    fn pretty_print_opcode(opcode: &BrilligOpcode) -> String {
        fn binary_field_op_to_string(op: &BinaryFieldOp) -> &str {
            match op {
                BinaryFieldOp::Add => "+",
                BinaryFieldOp::Sub => "-",
                BinaryFieldOp::Mul => "*",
                BinaryFieldOp::Div => "/",
                BinaryFieldOp::Equals => "==",
            }
        }

        fn binary_int_op_to_string(op: &BinaryIntOp) -> &str {
            match op {
                BinaryIntOp::Add => "+",
                BinaryIntOp::Sub => "-",
                BinaryIntOp::Mul => "*",
                BinaryIntOp::SignedDiv => "SDiv",
                BinaryIntOp::UnsignedDiv => "UDiv",
                BinaryIntOp::Equals => "==",
                BinaryIntOp::LessThan => "<",
                BinaryIntOp::LessThanEquals => "<=",
                BinaryIntOp::And => "&&",
                BinaryIntOp::Or => "||",
                BinaryIntOp::Xor => "^",
                BinaryIntOp::Shl => "<<",
                BinaryIntOp::Shr => ">>",
            }
        }

        match opcode {
            BrilligOpcode::BinaryFieldOp { destination, op, lhs, rhs } => {
                format!(
                    "r{} = r{} {} r{}",
                    destination.to_usize(),
                    lhs.to_usize(),
                    binary_field_op_to_string(op),
                    rhs.to_usize()
                )
            }
            BrilligOpcode::BinaryIntOp { destination, op, bit_size, lhs, rhs } => {
                format!(
                    "r{} = ({}-bit) r{} {} r{}",
                    destination.to_usize(),
                    bit_size,
                    lhs.to_usize(),
                    binary_int_op_to_string(op),
                    rhs.to_usize()
                )
            }
            BrilligOpcode::JumpIfNot { condition, location } => {
                format!("IF NOT r{} GOTO LABEL {}", condition.to_usize(), location)
            }
            BrilligOpcode::JumpIf { condition, location } => {
                format!("IF r{} GOTO LABEL {}", condition.to_usize(), location)
            }
            BrilligOpcode::Jump { location } => format!("GOTO LABEL {}", location),
            BrilligOpcode::Call { location } => format!("CALL LABEL {}", location),
            BrilligOpcode::Const { destination, value } => {
                format!("r{} = CONST {}", destination.to_usize(), value.to_field().to_hex())
            }
            BrilligOpcode::Return => String::from("RETURN"),
            BrilligOpcode::ForeignCall { function, destinations, inputs } => {
                // Assuming RegisterOrMemory also has a 'to_string' method
                // let destinations: Vec<String> =
                //     destinations.iter().map(|d| d.to_string()).collect();
                // let inputs: Vec<String> = inputs.iter().map(|i| i.to_string()).collect();
                // format!(
                //     "FOREIGNCALL {} IN ({}) OUT ({})",
                //     function,
                //     inputs.join(", "),
                //     destinations.join(", ")
                // )
                todo!()
            }
            BrilligOpcode::Mov { destination, source } => {
                format!("r{} = MOV r{}", destination.to_usize(), source.to_usize())
            }
            BrilligOpcode::Load { destination, source_pointer } => {
                format!("r{} = LOAD r{}", destination.to_usize(), source_pointer.to_usize())
            }
            BrilligOpcode::Store { destination_pointer, source } => {
                format!("STORE r{} TO r{}", source.to_usize(), destination_pointer.to_usize())
            }
            BrilligOpcode::Trap => String::from("TRAP"),
            BrilligOpcode::Stop => String::from("STOP"),
        }
    }
}

impl std::fmt::Display for BrilligArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.byte_code
                .iter()
                .map(Self::pretty_print_opcode)
                .collect::<Vec<_>>()
                .join("\n")
                .as_str(),
        )
    }
}
