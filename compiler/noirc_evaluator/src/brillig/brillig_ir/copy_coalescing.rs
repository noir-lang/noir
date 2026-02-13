use crate::brillig::brillig_ir::artifact::{BrilligArtifact, OpcodeLocation};

use acvm::acir::brillig::{BlackBoxOp, MemoryAddress, Opcode, ValueOrArray};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

impl<F: Clone + std::fmt::Debug> BrilligArtifact<F> {
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
        let label_positions: HashSet<OpcodeLocation> = self.labels.values().copied().collect();

        let mut copies: HashMap<MemoryAddress, MemoryAddress> = HashMap::default();
        let mut self_moves = Vec::new();

        for i in 0..self.byte_code.len() {
            if label_positions.contains(&i) {
                copies.clear();
            }

            rewrite_opcode_reads(&mut self.byte_code[i], &copies);

            if let Opcode::Mov { destination, source } = self.byte_code[i] {
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

        // TODO: is this necessary?
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
}

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
fn opcode_writes_stack_pointer<F>(opcode: &Opcode<F>) -> bool {
    let sp = MemoryAddress::Direct(0);
    match opcode {
        Opcode::BinaryFieldOp { destination, .. }
        | Opcode::BinaryIntOp { destination, .. }
        | Opcode::Not { destination, .. }
        | Opcode::Cast { destination, .. }
        | Opcode::Const { destination, .. }
        | Opcode::Load { destination, .. }
        | Opcode::ConditionalMov { destination, .. } => *destination == sp,
        Opcode::CalldataCopy { destination_address, .. } => *destination_address == sp,
        Opcode::ForeignCall { destinations, .. } => destinations.iter().any(|d| match d {
            ValueOrArray::MemoryAddress(a) => *a == sp,
            ValueOrArray::HeapVector(vec) => vec.size == sp,
            ValueOrArray::HeapArray(_) => false,
        }),
        Opcode::BlackBox(
            BlackBoxOp::EcdsaSecp256k1 { result, .. } | BlackBoxOp::EcdsaSecp256r1 { result, .. },
        ) => *result == sp,
        _ => false,
    }
}

fn is_control_flow<F>(opcode: &Opcode<F>) -> bool {
    matches!(
        opcode,
        Opcode::Jump { .. }
            | Opcode::JumpIf { .. }
            | Opcode::Call { .. }
            | Opcode::Return
            | Opcode::Trap { .. }
            | Opcode::Stop { .. }
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
fn rewrite_opcode_reads<F>(opcode: &mut Opcode<F>, copies: &HashMap<MemoryAddress, MemoryAddress>) {
    if copies.is_empty() {
        return;
    }
    match opcode {
        Opcode::BinaryFieldOp { lhs, rhs, .. } | Opcode::BinaryIntOp { lhs, rhs, .. } => {
            propagate(lhs, copies);
            propagate(rhs, copies);
        }
        Opcode::Cast { source, .. } | Opcode::Not { source, .. } | Opcode::Mov { source, .. } => {
            propagate(source, copies);
        }
        Opcode::Load { source_pointer, .. } => {
            propagate(source_pointer, copies);
        }
        Opcode::Store { destination_pointer, source } => {
            propagate(destination_pointer, copies);
            propagate(source, copies);
        }
        Opcode::JumpIf { condition, .. } => {
            propagate(condition, copies);
        }
        Opcode::CalldataCopy { size_address, offset_address, .. } => {
            propagate(size_address, copies);
            propagate(offset_address, copies);
        }
        Opcode::IndirectConst { destination_pointer, .. } => {
            propagate(destination_pointer, copies);
        }
        Opcode::ConditionalMov { source_a, source_b, condition, .. } => {
            propagate(condition, copies);
            propagate(source_a, copies);
            propagate(source_b, copies);
        }
        Opcode::ForeignCall { inputs, destinations, .. } => {
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
        Opcode::Trap { revert_data } => {
            propagate(&mut revert_data.pointer, copies);
            propagate(&mut revert_data.size, copies);
        }
        Opcode::Stop { return_data } => {
            propagate(&mut return_data.pointer, copies);
            propagate(&mut return_data.size, copies);
        }
        Opcode::BlackBox(op) => rewrite_black_box_reads(op, copies),
        Opcode::Const { .. } | Opcode::Jump { .. } | Opcode::Call { .. } | Opcode::Return => {}
    }
}

/// Invalidate copy entries for all registers written by an opcode.
/// Mov is handled separately in the main loop.
fn invalidate_opcode_writes<F>(
    opcode: &Opcode<F>,
    copies: &mut HashMap<MemoryAddress, MemoryAddress>,
) {
    if copies.is_empty() {
        return;
    }
    match opcode {
        Opcode::BinaryFieldOp { destination, .. }
        | Opcode::BinaryIntOp { destination, .. }
        | Opcode::Not { destination, .. }
        | Opcode::Cast { destination, .. }
        | Opcode::Const { destination, .. }
        | Opcode::Load { destination, .. }
        | Opcode::ConditionalMov { destination, .. } => {
            invalidate(copies, *destination);
        }
        Opcode::CalldataCopy { destination_address, .. } => {
            invalidate(copies, *destination_address);
        }
        Opcode::ForeignCall { destinations, .. } => {
            for dest in destinations {
                match dest {
                    ValueOrArray::MemoryAddress(addr) => invalidate(copies, *addr),
                    // The VM writes the result size to HeapVector::size
                    ValueOrArray::HeapVector(vec) => invalidate(copies, vec.size),
                    ValueOrArray::HeapArray(_) => {}
                }
            }
        }
        Opcode::BlackBox(op) => invalidate_black_box_writes(op, copies),
        // Mov handled in main loop; Store/IndirectConst write to heap, not registers
        Opcode::Mov { .. }
        | Opcode::Store { .. }
        | Opcode::IndirectConst { .. }
        | Opcode::JumpIf { .. }
        | Opcode::Jump { .. }
        | Opcode::Call { .. }
        | Opcode::Return
        | Opcode::Trap { .. }
        | Opcode::Stop { .. } => {}
    }
}

fn rewrite_black_box_reads(op: &mut BlackBoxOp, copies: &HashMap<MemoryAddress, MemoryAddress>) {
    match op {
        BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
            propagate(&mut inputs.pointer, copies);
            propagate(&mut iv.pointer, copies);
            propagate(&mut key.pointer, copies);
            propagate(&mut outputs.pointer, copies);
        }
        BlackBoxOp::Blake2s { message, output } | BlackBoxOp::Blake3 { message, output } => {
            propagate(&mut message.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::Keccakf1600 { input, output } => {
            propagate(&mut input.pointer, copies);
            propagate(&mut output.pointer, copies);
        }
        BlackBoxOp::EcdsaSecp256k1 {
            hashed_msg, public_key_x, public_key_y, signature, ..
        }
        | BlackBoxOp::EcdsaSecp256r1 {
            hashed_msg, public_key_x, public_key_y, signature, ..
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
        BlackBoxOp::EcdsaSecp256k1 { result, .. } | BlackBoxOp::EcdsaSecp256r1 { result, .. } => {
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
mod tests {
    use acvm::acir::brillig::{
        BinaryFieldOp, BitSize, HeapVector, MemoryAddress, Opcode as BrilligOpcode,
    };

    use super::BrilligArtifact;
    use crate::brillig::{Label, LabelType};

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

    #[test]
    fn self_move_elimination() {
        let mut artifact = BrilligArtifact::with_opcodes(vec![
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
        let mut artifact = BrilligArtifact::with_opcodes(vec![
            mov(2, 1),
            mov(3, 2),       // should become Mov r3, r1
            add_op(4, 3, 1), // should become Add r4, r1, r1
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
        let mut artifact =
            BrilligArtifact::with_opcodes(vec![mov(2, 1), const_op(1, 99), add_op(3, 2, 1)]);
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
        let mut artifact =
            BrilligArtifact::with_opcodes(vec![mov(2, 1), const_op(2, 99), add_op(3, 2, 1)]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code.len(), 3);
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn block_boundary_clears_copies() {
        // Mov r2, r1 in block 0; label at position 2; Add r3, r2, r1 in block 1
        // The copy should NOT propagate across the label.
        let mut artifact = BrilligArtifact::with_opcodes_and_label(
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
        let mut artifact =
            BrilligArtifact::with_opcodes(vec![mov(2, 1), BrilligOpcode::Return, add_op(3, 2, 1)]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn propagation_creates_self_move() {
        // Mov r2, r1; Mov r1, r2 → Mov r1, r1 → self-move, removed
        let mut artifact = BrilligArtifact::with_opcodes(vec![
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
        let mut artifact = BrilligArtifact::with_opcodes_and_label(
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
        let mut artifact = BrilligArtifact::with_opcodes(vec![
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
        let mut artifact = BrilligArtifact::with_opcodes(vec![
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
        let mut artifact = BrilligArtifact::with_opcodes(vec![
            mov(2, 1),
            mov(4, 3),
            BrilligOpcode::Trap { revert_data: HeapVector { pointer: direct(2), size: direct(4) } },
        ]);
        artifact.coalesce_copies();
        assert_eq!(
            artifact.byte_code[2],
            BrilligOpcode::Trap { revert_data: HeapVector { pointer: direct(1), size: direct(3) } }
        );
    }
}
