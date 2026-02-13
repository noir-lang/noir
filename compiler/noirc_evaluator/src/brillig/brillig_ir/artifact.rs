use acvm::acir::brillig::{BlackBoxOp, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray};
use acvm::acir::brillig::lengths::SemanticLength;
use acvm::acir::circuit::ErrorSelector;
use noirc_errors::call_stack::CallStackId;
use std::collections::{BTreeMap, HashMap, HashSet};

use super::procedures::ProcedureId;
use crate::ErrorType;
use crate::brillig::assert_usize;
use crate::ssa::ir::{basic_block::BasicBlockId, function::FunctionId};

/// Represents a parameter or a return value of an entry point function.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) enum BrilligParameter {
    /// A single address parameter or return value. Holds the bit size of the parameter.
    SingleAddr(u32),
    /// An array parameter or return value. Holds the type of an array item and its size.
    Array(Vec<BrilligParameter>, SemanticLength),
    /// A vector parameter or return value. Holds the type of a vector item.
    /// Only known-length vectors can be passed to brillig entry points, so the size is available as well.
    Vector(Vec<BrilligParameter>, SemanticLength),
}

impl BrilligParameter {
    /// Computes the size of a parameter if it was flattened
    pub(crate) fn flattened_size(&self) -> usize {
        match self {
            BrilligParameter::SingleAddr(_) => 1,
            BrilligParameter::Array(item_types, item_count)
            | BrilligParameter::Vector(item_types, item_count) => {
                let size_of_item: usize =
                    item_types.iter().map(|param| param.flattened_size()).sum();
                assert_usize(item_count.0) * size_of_item
            }
        }
    }
}

/// The result of compiling and linking brillig artifacts.
/// This is ready to run bytecode with attached metadata.
#[derive(Debug, Default, Clone)]
pub(crate) struct GeneratedBrillig<F> {
    pub(crate) byte_code: Vec<BrilligOpcode<F>>,
    pub(crate) locations: BTreeMap<OpcodeLocation, CallStackId>,
    pub(crate) error_types: BTreeMap<ErrorSelector, ErrorType>,
    pub(crate) name: String,
    pub(crate) procedure_locations: BTreeMap<ProcedureId, (OpcodeLocation, OpcodeLocation)>,
}

impl<F: std::fmt::Display> std::fmt::Display for GeneratedBrillig<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn {}", self.name)?;
        let width = self.byte_code.len().to_string().len();
        for (index, opcode) in self.byte_code.iter().enumerate() {
            writeln!(f, "{index:>width$}: {opcode}")?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
/// Artifacts resulting from the compilation of a function into brillig byte code.
/// It includes the bytecode of the function and all the metadata that allows linking with other functions.
pub struct BrilligArtifact<F> {
    pub(crate) byte_code: Vec<BrilligOpcode<F>>,
    pub(crate) error_types: BTreeMap<ErrorSelector, ErrorType>,
    /// The set of jumps that need to have their locations
    /// resolved.
    unresolved_jumps: Vec<(JumpInstructionPosition, UnresolvedJumpLocation)>,
    /// A map of labels to their position in byte code.
    pub(crate) labels: HashMap<Label, OpcodeLocation>,
    /// Set of labels which are external to the bytecode.
    ///
    /// This will most commonly contain the labels of functions
    /// which are defined in other bytecode, that this bytecode has called.
    /// TODO: perhaps we should combine this with the `unresolved_jumps` field
    /// TODO: and have an enum which indicates whether the jump is internal or external
    unresolved_external_call_labels: Vec<(JumpInstructionPosition, Label)>,
    /// Maps the opcodes that are associated with a callstack to it.
    locations: BTreeMap<OpcodeLocation, CallStackId>,
    /// The current call stack. All opcodes that are pushed will be associated with this call stack.
    call_stack_id: CallStackId,
    /// Name of the function, only used for debugging purposes.
    pub(crate) name: String,

    /// This field contains the given procedure id if this artifact originates from as procedure
    pub(crate) procedure: Option<ProcedureId>,
    /// Procedure ID mapped to the range of its opcode locations
    /// This is created as artifacts are linked together and allows us to determine
    /// which opcodes originate from reusable procedures.s
    /// The range is inclusive for both start and end opcode locations.
    pub(crate) procedure_locations: BTreeMap<ProcedureId, (OpcodeLocation, OpcodeLocation)>,
}

impl<F: std::fmt::Display> std::fmt::Display for BrilligArtifact<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn {}", self.name)?;
        let width = self.byte_code.len().to_string().len();
        for (index, opcode) in self.byte_code.iter().enumerate() {
            writeln!(f, "{index:>width$}: {opcode}")?;
        }
        Ok(())
    }
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
    /// Label for initialization of globals
    /// Stores a function ID referencing the entry point
    GlobalInit(FunctionId),
}

impl std::fmt::Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LabelType::Function(function_id, block_id) => {
                if let Some(block_id) = block_id {
                    write!(f, "Function({function_id:?}, {block_id:?})")
                } else {
                    write!(f, "Function({function_id:?})")
                }
            }
            LabelType::Entrypoint => write!(f, "Entrypoint"),
            LabelType::Procedure(procedure_id) => write!(f, "Procedure({procedure_id:?})"),
            LabelType::GlobalInit(function_id) => {
                write!(f, "Globals Initialization({function_id:?})")
            }
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
    pub(crate) fn with_section(&self, section: usize) -> Self {
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

    pub(crate) fn globals_init(function_id: FunctionId) -> Self {
        Label { label_type: LabelType::GlobalInit(function_id), section: None }
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
            self.locations.insert(position_in_bytecode + offset, *call_stack);
        }
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode<F>) {
        if !self.call_stack_id.is_root() {
            self.locations.insert(self.index_of_next_opcode(), self.call_stack_id);
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
        matches!(instruction, BrilligOpcode::JumpIf { .. } | BrilligOpcode::Jump { .. })
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
    /// before we start generating the opcodes for the region.
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
                    assert_eq!(
                        location, 0,
                        "location is not zero, which means that the jump label does not need resolving"
                    );

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::Jump { location: resolved_location };
                }
                BrilligOpcode::JumpIf { condition, location } => {
                    assert_eq!(
                        location, 0,
                        "location is not zero, which means that the jump label does not need resolving"
                    );

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::JumpIf { condition, location: resolved_location };
                }
                BrilligOpcode::Call { location } => {
                    assert_eq!(
                        location, 0,
                        "location is not zero, which means that the call label does not need resolving"
                    );

                    self.byte_code[*location_of_jump] =
                        BrilligOpcode::Call { location: resolved_location };
                }
                _ => unreachable!(
                    "all jump labels should point to a jump instruction in the bytecode"
                ),
            }
        }
    }

    /// Eliminates redundant MOV instructions through forward copy propagation.
    ///
    /// Within each basic block, tracks register-to-register copies and propagates
    /// them into subsequent read operands. MOVs that become self-moves (dst == src
    /// after propagation) are removed, and all internal position references are
    /// remapped to account for the removed opcodes.
    pub(crate) fn coalesce_copies(&mut self) {
        let self_moves = self.propagate_copies();
        self.remove_opcodes(&self_moves);
    }

    /// Phase 1: Forward copy propagation within basic blocks.
    /// Returns the sorted indices of self-move instructions to remove.
    fn propagate_copies(&mut self) -> Vec<OpcodeLocation> {
        let label_positions: HashSet<OpcodeLocation> =
            self.labels.values().copied().collect();
        let mut copies: HashMap<MemoryAddress, MemoryAddress> = HashMap::new();
        let mut self_moves = Vec::new();

        for i in 0..self.byte_code.len() {
            if label_positions.contains(&i) {
                copies.clear();
            }

            rewrite_opcode_reads(&mut self.byte_code[i], &copies);

            if let BrilligOpcode::Mov { destination, source } = self.byte_code[i] {
                if destination == source {
                    self_moves.push(i);
                } else {
                    invalidate(&mut copies, destination);
                    if destination == MemoryAddress::Direct(0) {
                        // Writing to the stack pointer changes the meaning of
                        // all Relative addresses, invalidating those copies.
                        invalidate_relative_entries(&mut copies);
                    } else {
                        copies.insert(destination, source);
                    }
                }
            } else if is_control_flow(&self.byte_code[i]) {
                copies.clear();
            } else {
                invalidate_opcode_writes(&self.byte_code[i], &mut copies);
                if opcode_writes_stack_pointer(&self.byte_code[i]) {
                    invalidate_relative_entries(&mut copies);
                }
            }
        }

        self_moves
    }

    /// Phase 2: Remove opcodes at the given sorted indices and remap all
    /// internal position-based references.
    fn remove_opcodes(&mut self, to_remove: &[OpcodeLocation]) {
        if to_remove.is_empty() {
            return;
        }

        let remap = |pos: OpcodeLocation| -> OpcodeLocation {
            pos - to_remove.partition_point(|&r| r < pos)
        };

        for pos in self.labels.values_mut() {
            *pos = remap(*pos);
        }

        for (pos, _) in &mut self.unresolved_jumps {
            *pos = remap(*pos);
        }

        for (pos, _) in &mut self.unresolved_external_call_labels {
            *pos = remap(*pos);
        }

        self.locations = self
            .locations
            .iter()
            .filter(|(pos, _)| to_remove.binary_search(pos).is_err())
            .map(|(pos, cs)| (remap(*pos), *cs))
            .collect();

        for (start, end) in self.procedure_locations.values_mut() {
            *start = remap(*start);
            *end = remap(*end);
        }

        let mut remove_idx = 0;
        self.byte_code = std::mem::take(&mut self.byte_code)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| {
                if remove_idx < to_remove.len() && to_remove[remove_idx] == *i {
                    remove_idx += 1;
                    false
                } else {
                    true
                }
            })
            .map(|(_, op)| op)
            .collect();
    }

    pub(crate) fn set_call_stack(&mut self, call_stack: CallStackId) {
        self.call_stack_id = call_stack;
    }

    #[cfg(test)]
    pub(crate) fn take_labels(&mut self) -> HashMap<Label, usize> {
        std::mem::take(&mut self.labels)
    }
}

// --- Copy coalescing helpers ---

/// Replace `addr` with its canonical source if one exists in the copy map.
fn propagate(addr: &mut MemoryAddress, copies: &HashMap<MemoryAddress, MemoryAddress>) {
    if let Some(&src) = copies.get(addr) {
        *addr = src;
    }
}

/// Remove all copy entries where `addr` appears as key (destination overwritten)
/// or value (source overwritten).
fn invalidate(copies: &mut HashMap<MemoryAddress, MemoryAddress>, addr: MemoryAddress) {
    copies.remove(&addr);
    copies.retain(|_, v| *v != addr);
}

/// Remove all copy entries involving Relative addresses. Called when the
/// stack pointer (Direct(0)) is written, since Relative addresses are
/// resolved relative to it and their meaning changes.
fn invalidate_relative_entries(copies: &mut HashMap<MemoryAddress, MemoryAddress>) {
    copies.retain(|k, v| !k.is_relative() && !v.is_relative());
}

/// Check whether an opcode (other than Mov, which is handled separately)
/// writes to Direct(0), the stack pointer register.
fn opcode_writes_stack_pointer<F>(opcode: &BrilligOpcode<F>) -> bool {
    let sp = MemoryAddress::Direct(0);
    match opcode {
        BrilligOpcode::BinaryFieldOp { destination, .. }
        | BrilligOpcode::BinaryIntOp { destination, .. }
        | BrilligOpcode::Not { destination, .. }
        | BrilligOpcode::Cast { destination, .. }
        | BrilligOpcode::Const { destination, .. }
        | BrilligOpcode::Load { destination, .. }
        | BrilligOpcode::ConditionalMov { destination, .. } => *destination == sp,
        BrilligOpcode::CalldataCopy { destination_address, .. } => *destination_address == sp,
        BrilligOpcode::ForeignCall { destinations, .. } => destinations.iter().any(|d| match d {
            ValueOrArray::MemoryAddress(a) => *a == sp,
            ValueOrArray::HeapVector(vec) => vec.size == sp,
            ValueOrArray::HeapArray(_) => false,
        }),
        BrilligOpcode::BlackBox(
            BlackBoxOp::EcdsaSecp256k1 { result, .. }
            | BlackBoxOp::EcdsaSecp256r1 { result, .. },
        ) => *result == sp,
        _ => false,
    }
}

fn is_control_flow<F>(opcode: &BrilligOpcode<F>) -> bool {
    matches!(
        opcode,
        BrilligOpcode::Jump { .. }
            | BrilligOpcode::JumpIf { .. }
            | BrilligOpcode::Call { .. }
            | BrilligOpcode::Return
            | BrilligOpcode::Trap { .. }
            | BrilligOpcode::Stop { .. }
    )
}

fn propagate_value_or_array(
    value: &mut ValueOrArray,
    copies: &HashMap<MemoryAddress, MemoryAddress>,
) {
    match value {
        ValueOrArray::MemoryAddress(addr) => propagate(addr, copies),
        ValueOrArray::HeapArray(arr) => propagate(&mut arr.pointer, copies),
        ValueOrArray::HeapVector(vec) => {
            propagate(&mut vec.pointer, copies);
            propagate(&mut vec.size, copies);
        }
    }
}

/// Propagate copies into all read operands of an opcode.
fn rewrite_opcode_reads<F>(
    opcode: &mut BrilligOpcode<F>,
    copies: &HashMap<MemoryAddress, MemoryAddress>,
) {
    if copies.is_empty() {
        return;
    }
    match opcode {
        BrilligOpcode::BinaryFieldOp { lhs, rhs, .. }
        | BrilligOpcode::BinaryIntOp { lhs, rhs, .. } => {
            propagate(lhs, copies);
            propagate(rhs, copies);
        }
        BrilligOpcode::Cast { source, .. }
        | BrilligOpcode::Not { source, .. }
        | BrilligOpcode::Mov { source, .. } => {
            propagate(source, copies);
        }
        BrilligOpcode::Load { source_pointer, .. } => {
            propagate(source_pointer, copies);
        }
        BrilligOpcode::Store { destination_pointer, source } => {
            propagate(destination_pointer, copies);
            propagate(source, copies);
        }
        BrilligOpcode::JumpIf { condition, .. } => {
            propagate(condition, copies);
        }
        BrilligOpcode::CalldataCopy { size_address, offset_address, .. } => {
            propagate(size_address, copies);
            propagate(offset_address, copies);
        }
        BrilligOpcode::IndirectConst { destination_pointer, .. } => {
            propagate(destination_pointer, copies);
        }
        BrilligOpcode::ConditionalMov { source_a, source_b, condition, .. } => {
            propagate(condition, copies);
            propagate(source_a, copies);
            propagate(source_b, copies);
        }
        BrilligOpcode::ForeignCall { inputs, destinations, .. } => {
            for input in inputs.iter_mut() {
                propagate_value_or_array(input, copies);
            }
            // HeapArray/HeapVector destination pointers are read by the VM
            // to know where to write results on the heap.
            // HeapVector size is NOT propagated because the VM writes to it.
            for dest in destinations.iter_mut() {
                match dest {
                    ValueOrArray::HeapArray(arr) => propagate(&mut arr.pointer, copies),
                    ValueOrArray::HeapVector(vec) => {
                        propagate(&mut vec.pointer, copies);
                        // vec.size is written by the VM, not read — don't propagate
                    }
                    ValueOrArray::MemoryAddress(_) => {}
                }
            }
        }
        BrilligOpcode::Trap { revert_data } => {
            propagate(&mut revert_data.pointer, copies);
            propagate(&mut revert_data.size, copies);
        }
        BrilligOpcode::Stop { return_data } => {
            propagate(&mut return_data.pointer, copies);
            propagate(&mut return_data.size, copies);
        }
        BrilligOpcode::BlackBox(op) => rewrite_black_box_reads(op, copies),
        BrilligOpcode::Const { .. }
        | BrilligOpcode::Jump { .. }
        | BrilligOpcode::Call { .. }
        | BrilligOpcode::Return => {}
    }
}

/// Invalidate copy entries for all registers written by an opcode.
/// Mov is handled separately in the main loop.
fn invalidate_opcode_writes<F>(
    opcode: &BrilligOpcode<F>,
    copies: &mut HashMap<MemoryAddress, MemoryAddress>,
) {
    if copies.is_empty() {
        return;
    }
    match opcode {
        BrilligOpcode::BinaryFieldOp { destination, .. }
        | BrilligOpcode::BinaryIntOp { destination, .. }
        | BrilligOpcode::Not { destination, .. }
        | BrilligOpcode::Cast { destination, .. }
        | BrilligOpcode::Const { destination, .. }
        | BrilligOpcode::Load { destination, .. }
        | BrilligOpcode::ConditionalMov { destination, .. } => {
            invalidate(copies, *destination);
        }
        BrilligOpcode::CalldataCopy { destination_address, .. } => {
            invalidate(copies, *destination_address);
        }
        BrilligOpcode::ForeignCall { destinations, .. } => {
            for dest in destinations {
                match dest {
                    ValueOrArray::MemoryAddress(addr) => invalidate(copies, *addr),
                    // The VM writes the result size to HeapVector::size
                    ValueOrArray::HeapVector(vec) => invalidate(copies, vec.size),
                    ValueOrArray::HeapArray(_) => {}
                }
            }
        }
        BrilligOpcode::BlackBox(op) => invalidate_black_box_writes(op, copies),
        // Mov handled in main loop; Store/IndirectConst write to heap, not registers
        BrilligOpcode::Mov { .. }
        | BrilligOpcode::Store { .. }
        | BrilligOpcode::IndirectConst { .. }
        | BrilligOpcode::JumpIf { .. }
        | BrilligOpcode::Jump { .. }
        | BrilligOpcode::Call { .. }
        | BrilligOpcode::Return
        | BrilligOpcode::Trap { .. }
        | BrilligOpcode::Stop { .. } => {}
    }
}

fn rewrite_black_box_reads(
    op: &mut BlackBoxOp,
    copies: &HashMap<MemoryAddress, MemoryAddress>,
) {
    match op {
        BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
            propagate(&mut inputs.pointer, copies);
            propagate(&mut iv.pointer, copies);
            propagate(&mut key.pointer, copies);
            propagate(&mut outputs.pointer, copies);
        }
        BlackBoxOp::Blake2s { message, output }
        | BlackBoxOp::Blake3 { message, output } => {
            propagate(&mut message.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::Keccakf1600 { input, output } => {
            propagate(&mut input.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::EcdsaSecp256k1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            ..
        }
        | BlackBoxOp::EcdsaSecp256r1 {
            hashed_msg,
            public_key_x,
            public_key_y,
            signature,
            ..
        } => {
            propagate(&mut hashed_msg.pointer, copies);
            propagate(&mut public_key_x.pointer, copies);
            propagate(&mut public_key_y.pointer, copies);
            propagate(&mut signature.pointer, copies);
        }
        BlackBoxOp::MultiScalarMul { points, scalars, outputs } => {
            propagate(&mut points.pointer, copies);
            propagate(&mut scalars.pointer, copies);
            propagate(&mut outputs.pointer, copies);
        }
        BlackBoxOp::EmbeddedCurveAdd {
            input1_x,
            input1_y,
            input1_infinite,
            input2_x,
            input2_y,
            input2_infinite,
            result,
        } => {
            propagate(input1_x, copies);
            propagate(input1_y, copies);
            propagate(input1_infinite, copies);
            propagate(input2_x, copies);
            propagate(input2_y, copies);
            propagate(input2_infinite, copies);
            propagate(&mut result.pointer, copies);
        }
        BlackBoxOp::Poseidon2Permutation { message, output } => {
            propagate(&mut message.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::Sha256Compression { input, hash_values, output } => {
            propagate(&mut input.pointer, copies);
            propagate(&mut hash_values.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::ToRadix { input, radix, output_pointer, num_limbs, output_bits } => {
            propagate(input, copies);
            propagate(radix, copies);
            propagate(output_pointer, copies);
            propagate(num_limbs, copies);
            propagate(output_bits, copies);
        }
    }
}

fn invalidate_black_box_writes(
    op: &BlackBoxOp,
    copies: &mut HashMap<MemoryAddress, MemoryAddress>,
) {
    match op {
        // ECDSA result is a scalar register write
        BlackBoxOp::EcdsaSecp256k1 { result, .. }
        | BlackBoxOp::EcdsaSecp256r1 { result, .. } => {
            invalidate(copies, *result);
        }
        // All other BlackBox ops write through heap pointers, not to registers
        BlackBoxOp::AES128Encrypt { .. }
        | BlackBoxOp::Blake2s { .. }
        | BlackBoxOp::Blake3 { .. }
        | BlackBoxOp::Keccakf1600 { .. }
        | BlackBoxOp::MultiScalarMul { .. }
        | BlackBoxOp::EmbeddedCurveAdd { .. }
        | BlackBoxOp::Poseidon2Permutation { .. }
        | BlackBoxOp::Sha256Compression { .. }
        | BlackBoxOp::ToRadix { .. } => {}
    }
}

#[cfg(test)]
mod copy_coalescing_tests {
    use acvm::acir::brillig::{
        BinaryFieldOp, BitSize, HeapVector, MemoryAddress, Opcode as BrilligOpcode,
    };

    use super::{BrilligArtifact, Label, LabelType, OpcodeLocation};

    fn direct(n: u32) -> MemoryAddress {
        MemoryAddress::Direct(n)
    }

    fn mov<F>(dst: u32, src: u32) -> BrilligOpcode<F> {
        BrilligOpcode::Mov { destination: direct(dst), source: direct(src) }
    }

    fn const_op(dst: u32, val: u128) -> BrilligOpcode<acvm::FieldElement> {
        BrilligOpcode::Const {
            destination: direct(dst),
            bit_size: BitSize::Integer(acvm::acir::brillig::IntegerBitSize::U64),
            value: acvm::FieldElement::from(val),
        }
    }

    fn add_op(dst: u32, lhs: u32, rhs: u32) -> BrilligOpcode<acvm::FieldElement> {
        BrilligOpcode::BinaryFieldOp {
            destination: direct(dst),
            op: BinaryFieldOp::Add,
            lhs: direct(lhs),
            rhs: direct(rhs),
        }
    }

    fn make_artifact(
        opcodes: Vec<BrilligOpcode<acvm::FieldElement>>,
    ) -> BrilligArtifact<acvm::FieldElement> {
        BrilligArtifact { byte_code: opcodes, ..Default::default() }
    }

    fn make_artifact_with_label(
        opcodes: Vec<BrilligOpcode<acvm::FieldElement>>,
        label_pos: OpcodeLocation,
    ) -> BrilligArtifact<acvm::FieldElement> {
        let label = Label { label_type: LabelType::Entrypoint, section: Some(99) };
        let mut labels = std::collections::HashMap::new();
        labels.insert(label, label_pos);
        BrilligArtifact { byte_code: opcodes, labels, ..Default::default() }
    }

    #[test]
    fn self_move_elimination() {
        let mut artifact = make_artifact(vec![
            mov(1, 1), // self-move
            mov(2, 2), // self-move
            const_op(3, 42),
        ]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 1);
        assert_eq!(artifact.byte_code[0], const_op(3, 42));
    }

    #[test]
    fn copy_chain_propagation() {
        // Mov r2, r1; Mov r3, r2 → Mov r3, r1 (chain propagation)
        // Then Add r4, r3, r1 → Add r4, r1, r1
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            mov(3, 2),        // should become Mov r3, r1
            add_op(4, 3, 1),  // should become Add r4, r1, r1
        ]);
        artifact.coalesce_copies();
        // Mov r2, r1 stays (not self-move)
        // Mov r3, r1 stays (not self-move, different registers)
        // Add r4, r1, r1
        assert_eq!(artifact.byte_code.len(), 3);
        assert_eq!(artifact.byte_code[1], mov(3, 1));
        assert_eq!(artifact.byte_code[2], add_op(4, 1, 1));
    }

    #[test]
    fn source_invalidation_on_write() {
        // Mov r2, r1; Const r1, 99; Add r3, r2, r1
        // After Const writes r1, {r2→r1} is invalidated.
        // So Add should NOT propagate r2 → r1.
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            const_op(1, 99),
            add_op(3, 2, 1),
        ]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 3);
        // r2 should NOT be propagated since r1 was overwritten
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn destination_invalidation_on_write() {
        // Mov r2, r1; Const r2, 99; Add r3, r2, r1
        // After Const writes r2, {r2→r1} is invalidated (key removed).
        // So Add should NOT propagate r2 → r1.
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            const_op(2, 99),
            add_op(3, 2, 1),
        ]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 3);
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn block_boundary_clears_copies() {
        // Mov r2, r1 in block 0; label at position 2; Add r3, r2, r1 in block 1
        // The copy should NOT propagate across the label.
        let mut artifact = make_artifact_with_label(
            vec![
                mov(2, 1),
                BrilligOpcode::Jump { location: 2 },
                add_op(3, 2, 1), // at position 2 = label position
            ],
            2,
        );
        artifact.coalesce_copies();
        // Add should be unchanged since copies cleared at label
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn control_flow_clears_copies() {
        // Mov r2, r1; Return; Add r3, r2, r1
        // (unrealistic but tests that Return clears copies)
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            BrilligOpcode::Return,
            add_op(3, 2, 1),
        ]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn propagation_creates_self_move() {
        // Mov r2, r1; Mov r1, r2 → Mov r1, r1 → self-move, removed
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            mov(1, 2), // becomes Mov r1, r1 after propagation
        ]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 1);
        assert_eq!(artifact.byte_code[0], mov(2, 1));
    }

    #[test]
    fn position_remapping_after_removal() {
        // Opcodes: [Const r1, Mov r1 r1 (self-move), Const r2]
        // Label at position 2 → should remap to 1 after removal
        let mut artifact = make_artifact_with_label(
            vec![const_op(1, 10), mov(1, 1), const_op(2, 20)],
            2,
        );
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 2);
        let label = Label { label_type: LabelType::Entrypoint, section: Some(99) };
        assert_eq!(artifact.labels[&label], 1);
    }

    #[test]
    fn location_mapping_after_removal() {
        let mut artifact = make_artifact(vec![
            const_op(1, 10), // pos 0
            mov(1, 1),       // pos 1 (self-move, removed)
            const_op(2, 20), // pos 2 → becomes pos 1
        ]);
        // Manually add location entries
        artifact.locations.insert(0, noirc_errors::call_stack::CallStackId::root());
        artifact.locations.insert(2, noirc_errors::call_stack::CallStackId::root());
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 2);
        // Position 0 stays, position 1 (self-move) removed, position 2 → 1
        assert!(artifact.locations.contains_key(&0));
        assert!(artifact.locations.contains_key(&1));
        assert!(!artifact.locations.contains_key(&2));
    }

    #[test]
    fn store_does_not_invalidate_copies() {
        // Mov r2, r1; Store [r3], r2 → Store [r3], r1 (propagated)
        // The Store does NOT invalidate {r2→r1} since it writes to heap.
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            BrilligOpcode::Store { destination_pointer: direct(3), source: direct(2) },
            add_op(4, 2, 1), // r2 should still propagate to r1
        ]);
        artifact.coalesce_copies();
        assert_eq!(
            artifact.byte_code[1],
            BrilligOpcode::Store { destination_pointer: direct(3), source: direct(1) }
        );
        assert_eq!(artifact.byte_code[2], add_op(4, 1, 1));
    }

    #[test]
    fn trap_reads_are_propagated() {
        let mut artifact = make_artifact(vec![
            mov(2, 1),
            mov(4, 3),
            BrilligOpcode::Trap {
                revert_data: HeapVector { pointer: direct(2), size: direct(4) },
            },
        ]);
        artifact.coalesce_copies();
        assert_eq!(
            artifact.byte_code[2],
            BrilligOpcode::Trap {
                revert_data: HeapVector { pointer: direct(1), size: direct(3) },
            }
        );
    }
}
