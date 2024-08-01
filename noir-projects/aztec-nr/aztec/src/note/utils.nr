use crate::{
    context::PrivateContext, generators::G_slot,
    note::{note_header::NoteHeader, note_interface::NoteInterface}
};

use dep::protocol_types::{
    hash::{
    compute_unique_note_hash, compute_siloed_note_hash as compute_siloed_note_hash,
    compute_siloed_nullifier as compute_siloed_nullifier_from_preimage
},
    point::Point, utils::arr_copy_slice
};
use dep::std::{embedded_curve_ops::multi_scalar_mul, hash::from_field_unsafe};

pub fn compute_slotted_note_hiding_point_raw(storage_slot: Field, note_hiding_point: Point) -> Point {
    // 1. We derive the storage slot point by multiplying the storage slot with the generator G_slot.
    // We use the unsafe version because the multi_scalar_mul will constrain the scalars.
    let storage_slot_scalar = from_field_unsafe(storage_slot);
    let storage_slot_point = multi_scalar_mul([G_slot], [storage_slot_scalar]);

    // 2. Then we compute the slotted note hiding point by adding the storage slot point to the note hiding point.
    storage_slot_point + note_hiding_point
}

pub fn compute_slotted_note_hash<Note, let N: u32, let M: u32>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note.get_header();
    let note_hiding_point = note.compute_note_hiding_point();

    compute_slotted_note_hiding_point_raw(header.storage_slot, note_hiding_point).x
}

pub fn compute_siloed_nullifier<Note, let N: u32, let M: u32>(
    note_with_header: Note,
    context: &mut PrivateContext
) -> Field where Note: NoteInterface<N, M> {
    let header = note_with_header.get_header();
    let (_, inner_nullifier) = note_with_header.compute_note_hash_and_nullifier(context);

    compute_siloed_nullifier_from_preimage(header.contract_address, inner_nullifier)
}

fn compute_note_hash_for_read_request_from_slotted_and_nonce(slotted_note_hash: Field, nonce: Field) -> Field {
    // TODO(#1386): This if-else can be nuked once we have nonces injected from public
    if (nonce == 0) {
        // If nonce is zero, that means we are reading a public note.
        slotted_note_hash
    } else {
        compute_unique_note_hash(nonce, slotted_note_hash)
    }
}

pub fn compute_note_hash_for_read_request<Note, let N: u32, let M: u32>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let slotted_note_hash = compute_slotted_note_hash(note);
    let nonce = note.get_header().nonce;
    let counter = note.get_header().note_hash_counter;

    if counter != 0 {
        slotted_note_hash
    } else {
        compute_note_hash_for_read_request_from_slotted_and_nonce(slotted_note_hash, nonce)
    }
}

pub fn compute_note_hash_for_consumption<Note, let N: u32, let M: u32>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note.get_header();
    // There are 4 cases for reading a note intended for consumption:
    // 1. The note was inserted in this transaction, is revertible, or is not nullified by a revertible nullifier in 
    //    the same transaction: (note_hash_counter != 0) & (nonce == 0)
    // 2. The note was inserted in this transaction, is non-revertible, and is nullified by a revertible nullifier in 
    //    the same transaction: (note_hash_counter != 0) & (nonce != 0)
    // 3. The note was inserted in a previous transaction, and was inserted in public: (note_hash_counter == 0) & (nonce == 0)
    // 4. The note was inserted in a previous transaction, and was inserted in private: (note_hash_counter == 0) & (nonce != 0)

    let slotted_note_hash = compute_slotted_note_hash(note);

    if ((header.note_hash_counter != 0) & (header.nonce == 0)) {
        // Case 1.
        // If a note is transient, we just read the slotted_note_hash (kernel will silo by contract address).
        slotted_note_hash
    } else {
        // Case 2: If a note is non-revertible, and is nullified by a revertible nullifier, we cannot squash them in the 
        // private reset circuit. Because if the tx reverts, we will have to keep the note hash and throw away the 
        // nullifier.
        // And if the tx does not revert, both will be emitted. In which case, the nullifier must be created in the app
        // from the siloed note hash.
        // The kernel circuit will check that a nullifier with non-zero note_nonce is linked to a note hash, whose
        // siloed note hash matches the note hash specified in the nullifier.

        // Case 3 & 4: If a note is not from the current transaction, that means we are reading a settled note (from 
        // tree) created in a previous TX. So we need the siloed_note_hash which has already been hashed with
        // nonce and then contract address. This hash will match the existing leaf in the note hash
        // tree, so the kernel can just perform a membership check directly on this hash/leaf.
        let unique_note_hash = compute_note_hash_for_read_request_from_slotted_and_nonce(slotted_note_hash, header.nonce);
        compute_siloed_note_hash(header.contract_address, unique_note_hash)
        // IMPORTANT NOTE ON REDUNDANT SILOING BY CONTRACT ADDRESS: The note hash computed above is
        // "siloed" by contract address. When a note hash is computed solely for the purpose of
        // nullification, it is not strictly necessary to silo the note hash before computing
        // its nullifier. In other words, it is NOT NECESSARY for protocol security that a nullifier
        // be computed from a siloed note hash. After all, persistable note hashes and nullifiers are
        // siloed by the kernel circuit. That being said, the siloed note hash computed above CAN be
        // used for nullifier computation, and this achieves the (arguably unnecessary) property that
        // nullifiers are computed from a note hash's fully-computed note hash tree leaf.
    }
}

pub fn compute_note_hash_and_optionally_a_nullifier<T, let N: u32, let M: u32, let S: u32>(
    deserialize_content: fn([Field; N]) -> T,
    note_header: NoteHeader,
    compute_nullifier: bool,
    serialized_note: [Field; S]
) -> [Field; 4] where T: NoteInterface<N, M> {
    let mut note = deserialize_content(arr_copy_slice(serialized_note, [0; N], 0));
    note.set_header(note_header);

    let slotted_note_hash = compute_slotted_note_hash(note);
    let unique_note_hash = compute_note_hash_for_read_request_from_slotted_and_nonce(slotted_note_hash, note_header.nonce);
    let siloed_note_hash = compute_siloed_note_hash(note_header.contract_address, unique_note_hash);

    let inner_nullifier = if compute_nullifier {
        let (_, nullifier) = note.compute_note_hash_and_nullifier_without_context();
        nullifier
    } else {
        0
    };
    // docs:start:compute_note_hash_and_optionally_a_nullifier_returns
    [slotted_note_hash, unique_note_hash, siloed_note_hash, inner_nullifier]
    // docs:end:compute_note_hash_and_optionally_a_nullifier_returns
}
