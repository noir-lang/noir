use acvm::acir::brillig::Opcode as BrilligOpcode;
use acvm::acir::circuit::ErrorSelector;
use std::collections::{BTreeMap, HashMap};

use crate::ssa::ir::{basic_block::BasicBlockId, call_stack::CallStack, function::FunctionId};
use crate::ErrorType;

use super::procedures::ProcedureId;

/// Represents a parameter or a return value of an entry point function.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) enum BrilligParameter {
    /// A single address parameter or return value. Holds the bit size of the parameter.
    SingleAddr(u32),
    /// An array parameter or return value. Holds the type of an array item and its size.
    Array(Vec<BrilligParameter>, usize),
    /// A slice parameter or return value. Holds the type of a slice item.
    /// Only known-length slices can be passed to brillig entry points, so the size is available as well.
    Slice(Vec<BrilligParameter>, usize),
}

/// The result of compiling and linking brillig artifacts.
/// This is ready to run bytecode with attached metadata.
#[derive(Debug, Default)]
pub(crate) struct GeneratedBrillig<F> {
    pub(crate) byte_code: Vec<BrilligOpcode<F>>,
    pub(crate) locations: BTreeMap<OpcodeLocation, CallStack>,
    pub(crate) error_types: BTreeMap<ErrorSelector, ErrorType>,
    pub(crate) name: String,
    pub(crate) procedure_locations: HashMap<ProcedureId, (OpcodeLocation, OpcodeLocation)>,
}

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code.
/// It includes the bytecode of the function and all the metadata that allows linking with other functions.
pub(crate) struct BrilligArtifact<F> {
    pub(crate) byte_code: Vec<BrilligOpcode<F>>,
    pub(crate) error_types: BTreeMap<ErrorSelector, ErrorType>,
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
    unresolved_external_call_labels: Vec<(JumpInstructionPosition, Label)>,
    /// Maps the opcodes that are associated with a callstack to it.
    locations: BTreeMap<OpcodeLocation, CallStack>,
    /// The current call stack. All opcodes that are pushed will be associated with this call stack.
    call_stack: CallStack,
    /// Name of the function, only used for debugging purposes.
    pub(crate) name: String,

    /// This field contains the given procedure id if this artifact originates from as procedure
    pub(crate) procedure: Option<ProcedureId>,
    /// Procedure ID mapped to the range of its opcode locations
    /// This is created as artifacts are linked together and allows us to determine
    /// which opcodes originate from reusable procedures.s
    /// The range is inclusive for both start and end opcode locations.
    pub(crate) procedure_locations: HashMap<ProcedureId, (OpcodeLocation, OpcodeLocation)>,
}

/// A pointer to a location in the opcode.
pub(crate) type OpcodeLocation = usize;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum LabelType {
    /// Labels for the entry point bytecode
    Entrypoint,
    /// Labels for user defined functions
    Function(FunctionId, Option<BasicBlockId>),
    /// Labels for intrinsic procedures
    Procedure(ProcedureId),
}

impl std::fmt::Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LabelType::Function(function_id, block_id) => {
                if let Some(block_id) = block_id {
                    write!(f, "Function({:?}, {:?})", function_id, block_id)
                } else {
                    write!(f, "Function({:?})", function_id)
                }
            }
            LabelType::Entrypoint => write!(f, "Entrypoint"),
            LabelType::Procedure(procedure_id) => write!(f, "Procedure({:?})", procedure_id),
        }
    }
}

/// An identifier for a location in the code.
///
/// It is assumed that an entity will keep a map
/// of labels to Opcode locations.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Label {
    pub(crate) label_type: LabelType,
    pub(crate) section: Option<usize>,
}

impl Label {
    pub(crate) fn add_section(&self, section: usize) -> Self {
        Label { label_type: self.label_type.clone(), section: Some(section) }
    }

    pub(crate) fn function(func_id: FunctionId) -> Self {
        Label { label_type: LabelType::Function(func_id, None), section: None }
    }

    pub(crate) fn block(func_id: FunctionId, block_id: BasicBlockId) -> Self {
        Label { label_type: LabelType::Function(func_id, Some(block_id)), section: None }
    }

    pub(crate) fn entrypoint() -> Self {
        Label { label_type: LabelType::Entrypoint, section: None }
    }

    pub(crate) fn procedure(procedure_id: ProcedureId) -> Self {
        Label { label_type: LabelType::Procedure(procedure_id), section: None }
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(section) = self.section {
            write!(f, "{:?} - {}", self.label_type, section)
        } else {
            write!(f, "{:?}", self.label_type)
        }
    }
}
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

impl<F: Clone + std::fmt::Debug> BrilligArtifact<F> {
    /// Resolves all jumps and generates the final bytecode
    pub(crate) fn finish(mut self) -> GeneratedBrillig<F> {
        self.resolve_jumps();
        GeneratedBrillig {
            byte_code: self.byte_code,
            locations: self.locations,
            error_types: self.error_types,
            name: self.name,
            procedure_locations: self.procedure_locations,
        }
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
    pub(crate) fn link_with(&mut self, obj: &BrilligArtifact<F>) {
        // Add the unresolved jumps of the linked function to this artifact.
        self.add_unresolved_jumps_and_calls(obj);

        for (error_selector, error_type) in &obj.error_types {
            self.error_types.insert(*error_selector, error_type.clone());
        }

        self.byte_code.append(&mut obj.byte_code.clone());

        // Remove all resolved external calls and transform them to jumps
        let is_resolved = |label: &Label| self.labels.contains_key(label);

        let resolved_external_calls = self
            .unresolved_external_call_labels
            .iter()
            .filter(|(_, label)| is_resolved(label))
            .cloned()
            .collect::<Vec<_>>();

        for resolved_external_call in resolved_external_calls {
            self.unresolved_jumps.push(resolved_external_call);
        }

        self.unresolved_external_call_labels.retain(|(_, label)| !is_resolved(label));
    }

    /// Adds unresolved jumps & function calls from another artifact offset by the current opcode count in the artifact.
    fn add_unresolved_jumps_and_calls(&mut self, obj: &BrilligArtifact<F>) {
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

        for (position_in_bytecode, call_stack) in obj.locations.iter() {
            self.locations.insert(position_in_bytecode + offset, call_stack.clone());
        }
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode<F>) {
        if !self.call_stack.is_empty() {
            self.locations.insert(self.index_of_next_opcode(), self.call_stack.clone());
        }
        self.byte_code.push(opcode);
    }

    /// Adds a unresolved jump to be fixed at the end of bytecode processing.
    pub(crate) fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode<F>,
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
        call_instruction: BrilligOpcode<F>,
        destination: UnresolvedJumpLocation,
    ) {
        // TODO: Add a check to ensure that the opcode is a call instruction

        self.unresolved_external_call_labels.push((self.index_of_next_opcode(), destination));
        self.push_opcode(call_instruction);
    }

    /// Returns true if the opcode is a jump instruction
    fn is_jmp_instruction(instruction: &BrilligOpcode<F>) -> bool {
        matches!(
            instruction,
            BrilligOpcode::JumpIfNot { .. }
                | BrilligOpcode::JumpIf { .. }
                | BrilligOpcode::Jump { .. }
        )
    }

    /// Adds a label in the bytecode to specify where this block's
    /// opcodes will start.
    pub(crate) fn add_label_at_position(&mut self, label: Label, position: OpcodeLocation) {
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

    pub(crate) fn set_call_stack(&mut self, call_stack: CallStack) {
        self.call_stack = call_stack;
    }
}
