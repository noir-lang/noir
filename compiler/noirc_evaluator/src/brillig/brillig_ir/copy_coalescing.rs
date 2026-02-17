use crate::brillig::brillig_ir::artifact::{BrilligArtifact, OpcodeLocation};

use acvm::acir::brillig::{BlackBoxOp, MemoryAddress, Opcode, ValueOrArray};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::VecDeque;

type CopyMap = HashMap<MemoryAddress, MemoryAddress>;

/// A basic block defined by a half-open range [start, end) in the opcode array.
struct Block {
    start: OpcodeLocation,
    end: OpcodeLocation,
}

impl<F: Clone + std::fmt::Debug> BrilligArtifact<F> {
    /// Eliminates redundant MOV instructions through forward copy propagation.
    ///
    /// Uses CFG-aware iterative dataflow analysis to propagate copies across
    /// block boundaries. At join points, only copies common to ALL predecessors
    /// survive. Call instructions kill all copies (callee can modify any register).
    /// Return/Trap/Stop are terminal (no successors).
    pub(crate) fn coalesce_copies(&mut self) {
        let self_moves = self.propagate_copies();
        self.remove_opcodes(&self_moves);
    }

    /// CFG-aware forward copy propagation.
    /// Returns the sorted indices of self-move instructions to remove.
    fn propagate_copies(&mut self) -> Vec<OpcodeLocation> {
        if self.byte_code.is_empty() {
            return Vec::new();
        }

        let label_positions: HashSet<OpcodeLocation> = self.labels.values().copied().collect();

        // Build a map from unresolved jump position → target opcode location.
        let jump_targets: HashMap<OpcodeLocation, OpcodeLocation> = self
            .unresolved_jumps
            .iter()
            .filter_map(|(pos, label)| self.labels.get(label).map(|&target| (*pos, target)))
            .collect();

        // Phase 1: Identify block boundaries.
        let blocks = build_blocks(&self.byte_code, &label_positions);
        let num_blocks = blocks.len();

        if num_blocks == 0 {
            return Vec::new();
        }

        // Phase 2: Build CFG (successor/predecessor lists).
        let (successors, predecessors) =
            build_cfg(&blocks, &self.byte_code, &jump_targets);

        // Phase 3: Iterative dataflow analysis.
        // OUT[b] = transfer(IN[b], instructions in b)
        // IN[b] = intersect(OUT[predecessors of b])
        // None = top (uncomputed), Some({}) = bottom (empty map)
        let mut block_in: Vec<Option<CopyMap>> = vec![None; num_blocks];
        let mut block_out: Vec<Option<CopyMap>> = vec![None; num_blocks];

        // Entry block starts with empty copy map.
        block_in[0] = Some(CopyMap::default());

        let mut worklist: VecDeque<usize> = VecDeque::new();
        worklist.push_back(0);

        while let Some(block_idx) = worklist.pop_front() {
            let in_map = match &block_in[block_idx] {
                Some(m) => m.clone(),
                None => continue,
            };

            let out_map =
                transfer_block(&self.byte_code, &blocks[block_idx], in_map);

            let changed = block_out[block_idx].as_ref() != Some(&out_map);
            block_out[block_idx] = Some(out_map);

            if changed {
                for &succ in &successors[block_idx] {
                    let new_in = merge_copy_maps(&predecessors[succ], &block_out);
                    if block_in[succ] != new_in {
                        block_in[succ] = new_in;
                        if !worklist.contains(&succ) {
                            worklist.push_back(succ);
                        }
                    }
                }
            }
        }

        // Phase 4: Rewrite opcodes using the computed IN maps.
        let mut self_moves = Vec::new();

        for (block_idx, block) in blocks.iter().enumerate() {
            let mut copies = block_in[block_idx].clone().unwrap_or_default();

            for i in block.start..block.end {
                rewrite_opcode_reads(&mut self.byte_code[i], &copies);
                update_copies_after_rewrite(&self.byte_code[i], &mut copies, i, &mut self_moves);
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

/// Identify basic block boundaries from label positions and control flow opcodes.
/// A new block starts at position 0, every label position, and every position
/// immediately after a control flow instruction.
fn build_blocks<F>(
    byte_code: &[Opcode<F>],
    label_positions: &HashSet<OpcodeLocation>,
) -> Vec<Block> {
    let mut block_starts: HashSet<OpcodeLocation> = HashSet::default();
    block_starts.insert(0);

    for &pos in label_positions {
        if pos < byte_code.len() {
            block_starts.insert(pos);
        }
    }

    for (i, opcode) in byte_code.iter().enumerate() {
        if is_control_flow(opcode) && i + 1 < byte_code.len() {
            block_starts.insert(i + 1);
        }
    }

    let mut starts: Vec<OpcodeLocation> = block_starts.into_iter().collect();
    starts.sort_unstable();

    starts
        .windows(2)
        .map(|w| Block { start: w[0], end: w[1] })
        .chain(std::iter::once(Block {
            start: *starts.last().unwrap(),
            end: byte_code.len(),
        }))
        .collect()
}

/// Binary search for the block index that starts at the given position.
fn block_index_at(blocks: &[Block], pos: OpcodeLocation) -> Option<usize> {
    blocks.binary_search_by_key(&pos, |b| b.start).ok()
}

/// Build successor and predecessor lists for the CFG.
fn build_cfg<F>(
    blocks: &[Block],
    byte_code: &[Opcode<F>],
    jump_targets: &HashMap<OpcodeLocation, OpcodeLocation>,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let num_blocks = blocks.len();
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); num_blocks];
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); num_blocks];

    for (block_idx, block) in blocks.iter().enumerate() {
        if block.start >= block.end {
            continue;
        }

        let last_pos = block.end - 1;
        let last_opcode = &byte_code[last_pos];

        match last_opcode {
            Opcode::Jump { location } => {
                // Try unresolved jump target first, then fall back to resolved location.
                let target = jump_targets.get(&last_pos).copied().unwrap_or(*location);
                if let Some(succ) = block_index_at(blocks, target) {
                    successors[block_idx].push(succ);
                }
            }
            Opcode::JumpIf { location, .. } => {
                // Conditional: target + fallthrough.
                let target = jump_targets.get(&last_pos).copied().unwrap_or(*location);
                if let Some(succ) = block_index_at(blocks, target) {
                    successors[block_idx].push(succ);
                }
                // Fallthrough to next block.
                if block_idx + 1 < num_blocks {
                    let next = block_idx + 1;
                    if !successors[block_idx].contains(&next) {
                        successors[block_idx].push(next);
                    }
                }
            }
            Opcode::Call { location } => {
                // Call returns to the next instruction (fallthrough).
                // The target of Call is a function, not a block within this function.
                let _ = location;
                if block_idx + 1 < num_blocks {
                    successors[block_idx].push(block_idx + 1);
                }
            }
            Opcode::Return | Opcode::Trap { .. } | Opcode::Stop { .. } => {
                // Terminal: no successors.
            }
            _ => {
                // Non-control-flow at end of block (block was split by a label
                // at the next position). Fallthrough to next block.
                if block_idx + 1 < num_blocks {
                    successors[block_idx].push(block_idx + 1);
                }
            }
        }
    }

    // Build predecessor lists from successors.
    for (block_idx, succs) in successors.iter().enumerate() {
        for &succ in succs {
            if !predecessors[succ].contains(&block_idx) {
                predecessors[succ].push(block_idx);
            }
        }
    }

    (successors, predecessors)
}

/// Transfer function: simulate instructions in a block, updating the copy map.
/// Unlike the old code, Jump and JumpIf do NOT clear copies (they don't modify registers).
/// Call clears all copies. Return/Trap/Stop are terminal.
fn transfer_block<F: Clone + std::fmt::Debug>(
    byte_code: &[Opcode<F>],
    block: &Block,
    mut copies: CopyMap,
) -> CopyMap {
    for i in block.start..block.end {
        let opcode = &byte_code[i];
        if let Opcode::Mov { destination, source } = opcode {
            let mut src = *source;
            // Simulate read propagation.
            if let Some(&canonical) = copies.get(&src) {
                src = canonical;
            }
            let dst = *destination;
            if dst != src {
                invalidate(&mut copies, dst);
                if dst == MemoryAddress::Direct(0) {
                    invalidate_relative_entries(&mut copies);
                } else {
                    copies.insert(dst, src);
                }
            }
            // If dst == src after propagation, it's a self-move; no map change needed.
        } else if matches!(opcode, Opcode::Call { .. }) {
            copies.clear();
        } else if matches!(
            opcode,
            Opcode::Return | Opcode::Trap { .. } | Opcode::Stop { .. }
        ) {
            // Terminal: copies don't matter after this, but we still
            // keep the map for the OUT of this block (it won't have successors).
        } else if matches!(opcode, Opcode::Jump { .. } | Opcode::JumpIf { .. }) {
            // Jump/JumpIf don't modify registers; copies flow through.
        } else {
            invalidate_opcode_writes(opcode, &mut copies);
            if opcode_writes_stack_pointer(opcode) {
                invalidate_relative_entries(&mut copies);
            }
        }
    }
    copies
}

/// Merge copy maps from predecessors using intersection.
/// None = uncomputed (top), treated as no constraint.
/// Only copies present in ALL computed predecessors survive.
fn merge_copy_maps(
    predecessor_indices: &[usize],
    block_out: &[Option<CopyMap>],
) -> Option<CopyMap> {
    let mut result: Option<CopyMap> = None;

    for &pred_idx in predecessor_indices {
        match &block_out[pred_idx] {
            None => {
                // Uncomputed predecessor = top, no constraint.
                continue;
            }
            Some(pred_map) => match &mut result {
                None => {
                    result = Some(pred_map.clone());
                }
                Some(current) => {
                    // Intersect: keep only entries present in both with same value.
                    current.retain(|k, v| pred_map.get(k) == Some(v));
                }
            },
        }
    }

    result
}

/// Update the copy map after rewriting an opcode's reads (used in the rewrite phase).
fn update_copies_after_rewrite<F>(
    opcode: &Opcode<F>,
    copies: &mut CopyMap,
    pos: OpcodeLocation,
    self_moves: &mut Vec<OpcodeLocation>,
) {
    if let Opcode::Mov { destination, source } = opcode {
        if *destination == *source {
            self_moves.push(pos);
        } else {
            invalidate(copies, *destination);
            if *destination == MemoryAddress::Direct(0) {
                invalidate_relative_entries(copies);
            } else {
                copies.insert(*destination, *source);
            }
        }
    } else if matches!(opcode, Opcode::Call { .. }) {
        copies.clear();
    } else if matches!(opcode, Opcode::Jump { .. } | Opcode::JumpIf { .. }) {
        // Jump/JumpIf don't modify registers.
    } else if matches!(
        opcode,
        Opcode::Return | Opcode::Trap { .. } | Opcode::Stop { .. }
    ) {
        // Terminal: no effect on copies needed for rewrite.
    } else {
        invalidate_opcode_writes(opcode, copies);
        if opcode_writes_stack_pointer(opcode) {
            invalidate_relative_entries(copies);
        }
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

    /// Create a label with a unique section number.
    fn label(section: usize) -> Label {
        Label { label_type: LabelType::Entrypoint, section: Some(section) }
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
    fn cross_block_propagation_through_jump() {
        // Mov r2, r1 in block 0; Jump to label at position 2; Add r3, r2, r1 in block 1
        // The copy SHOULD propagate across the jump since it's the only predecessor.
        let mut artifact = BrilligArtifact::with_opcodes_and_label(
            vec![
                mov(2, 1),
                BrilligOpcode::Jump { location: 2 },
                add_op(3, 2, 1), // at position 2 = label position
            ],
            2,
        );
        artifact.coalesce_copies();
        // Add should have r2 propagated to r1
        assert_eq!(artifact.byte_code[2], add_op(3, 1, 1));
    }

    #[test]
    fn control_flow_clears_copies() {
        // Mov r2, r1; Return; Add r3, r2, r1
        // Return is terminal, so the block after it has no predecessors → no copies.
        let mut artifact =
            BrilligArtifact::with_opcodes(vec![mov(2, 1), BrilligOpcode::Return, add_op(3, 2, 1)]);
        artifact.coalesce_copies();
        assert_eq!(artifact.byte_code[2], add_op(3, 2, 1));
    }

    #[test]
    fn cross_block_propagation_through_fallthrough() {
        // Copies flow through a label-only boundary (fallthrough) when there's
        // only one predecessor.
        let mut artifact = BrilligArtifact::with_opcodes_and_label(
            vec![
                mov(2, 1),       // block 0
                add_op(5, 2, 1), // block 0 - uses copy
                add_op(3, 2, 1), // block 1 (starts at label position 2)
            ],
            2,
        );
        artifact.coalesce_copies();
        // Copy should propagate through the label boundary
        assert_eq!(artifact.byte_code[1], add_op(5, 1, 1));
        assert_eq!(artifact.byte_code[2], add_op(3, 1, 1));
    }

    #[test]
    fn merge_discards_conflicting_copies() {
        // Block 0: Mov r2, r1; JumpIf → label_a (pos 3)
        // Block 1 (fallthrough): Mov r2, r3; Jump → label_a (pos 3)
        // Block 2 (label_a): Add r4, r2, r1
        // Predecessors of block 2: block 0 has {r2→r1}, block 1 has {r2→r3}
        // Intersection: r2 maps differ → discarded.
        let mut artifact = BrilligArtifact::with_opcodes_and_labels(
            vec![
                mov(2, 1),                                                     // 0: block 0
                BrilligOpcode::JumpIf { condition: direct(10), location: 4 },  // 1: block 0 end
                mov(2, 3),                                                     // 2: block 1
                BrilligOpcode::Jump { location: 4 },                           // 3: block 1 end
                add_op(4, 2, 1),                                               // 4: block 2 (label_a)
            ],
            &[(label(1), 4)],
        );
        artifact.coalesce_copies();
        // r2 should NOT be propagated since predecessors disagree
        assert_eq!(artifact.byte_code[4], add_op(4, 2, 1));
    }

    #[test]
    fn merge_preserves_common_copies() {
        // Block 0: Mov r2, r1; JumpIf → label_a (pos 4)
        // Block 1 (fallthrough): Mov r3, r5; Jump → label_a (pos 4)
        // Block 2 (label_a): Add r4, r2, r1
        // Both predecessors have {r2→r1} → preserved at merge.
        let mut artifact = BrilligArtifact::with_opcodes_and_labels(
            vec![
                mov(2, 1),                                                     // 0: block 0
                BrilligOpcode::JumpIf { condition: direct(10), location: 4 },  // 1: block 0 end
                mov(3, 5),                                                     // 2: block 1
                BrilligOpcode::Jump { location: 4 },                           // 3: block 1 end
                add_op(4, 2, 1),                                               // 4: block 2 (label_a)
            ],
            &[(label(1), 4)],
        );
        artifact.coalesce_copies();
        // r2→r1 is common to both predecessors, should propagate
        assert_eq!(artifact.byte_code[4], add_op(4, 1, 1));
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
