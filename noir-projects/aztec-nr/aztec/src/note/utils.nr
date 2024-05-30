use crate::{context::PrivateContext, note::{note_header::NoteHeader, note_interface::NoteInterface}};

use dep::protocol_types::{
    address::AztecAddress,
    constants::{
    GENERATOR_INDEX__OUTER_NULLIFIER, GENERATOR_INDEX__UNIQUE_NOTE_HASH,
    GENERATOR_INDEX__SILOED_NOTE_HASH, GENERATOR_INDEX__INNER_NOTE_HASH
},
    hash::pedersen_hash, utils::arr_copy_slice
};

fn compute_siloed_hash(contract_address: AztecAddress, unique_note_hash: Field) -> Field {
    let inputs = [contract_address.to_field(), unique_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__SILOED_NOTE_HASH)
}

fn compute_unique_hash(nonce: Field, inner_note_hash: Field) -> Field {
    let inputs = [nonce, inner_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__UNIQUE_NOTE_HASH)
}

fn compute_inner_note_hash<Note, N, M>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note.get_header();
    let note_hash = note.compute_note_content_hash();

    pedersen_hash(
        [header.storage_slot, note_hash],
        GENERATOR_INDEX__INNER_NOTE_HASH
    )
}

fn compute_unique_note_hash<Note, N, M>(note_with_header: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note_with_header.get_header();

    let inner_note_hash = compute_inner_note_hash(note_with_header);

    compute_unique_hash(header.nonce, inner_note_hash)
}

fn compute_siloed_note_hash<Note, N, M>(note_with_header: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note_with_header.get_header();

    let unique_note_hash = if (header.nonce == 0) {
        // If nonce is zero, that means we are reading a public note.
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
        // Remove this once notes added from public also include nonces.
        compute_inner_note_hash(note_with_header)
    } else {
        compute_unique_note_hash(note_with_header)
    };

    compute_siloed_hash(header.contract_address, unique_note_hash)
}

pub fn compute_siloed_nullifier<Note, N, M>(
    note_with_header: Note,
    context: &mut PrivateContext
) -> Field where Note: NoteInterface<N, M> {
    let header = note_with_header.get_header();
    let inner_nullifier = note_with_header.compute_nullifier(context);

    let input = [header.contract_address.to_field(), inner_nullifier];
    pedersen_hash(input, GENERATOR_INDEX__OUTER_NULLIFIER)
}

pub fn compute_note_hash_for_insertion<Note, N, M>(note: Note) -> Field where Note: NoteInterface<N, M> {
    compute_inner_note_hash(note)
}

pub fn compute_note_hash_for_read_request<Note, N, M>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note.get_header();

    if (header.nonce != 0) {
        compute_unique_note_hash(note)
    } else {
        compute_inner_note_hash(note)
    }
}

pub fn compute_note_hash_for_consumption<Note, N, M>(note: Note) -> Field where Note: NoteInterface<N, M> {
    let header = note.get_header();
    // There are 3 cases for reading a note intended for consumption:
    // 1. The note was inserted in this transaction, and is transient.
    // 2. The note was inserted in a previous transaction, and was inserted in public
    // 3. The note was inserted in a previous transaction, and was inserted in private

    if (header.note_hash_counter != 0) {
        // If a note is transient, we just read the inner_note_hash (kernel will silo by contract address).
        compute_inner_note_hash(note)
    } else {
        // If a note is not transient, that means we are reading a settled note (from tree) created in a
        // previous TX. So we need the siloed_note_hash which has already been hashed with
        // nonce and then contract address. This hash will match the existing leaf in the note hash
        // tree, so the kernel can just perform a membership check directly on this hash/leaf.
        compute_siloed_note_hash(note)
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

pub fn compute_note_hash_and_nullifier<T, N, M, S>(
    // docs:start:compute_note_hash_and_nullifier_args
    deserialize_content: fn([Field; N]) -> T,
    note_header: NoteHeader,
    serialized_note: [Field; S] // docs:end:compute_note_hash_and_nullifier_args
) -> [Field; 4] where T: NoteInterface<N, M> {
    let mut note = deserialize_content(arr_copy_slice(serialized_note, [0; N], 0));
    // TODO: change this to note.set_header(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    T::set_header((&mut note), note_header);

    let inner_note_hash = compute_inner_note_hash(note);

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
    // Should always be calling compute_unique_hash() once notes added from public also include nonces.
    let unique_note_hash = if note_header.nonce != 0 {
        compute_unique_hash(note_header.nonce, inner_note_hash)
    } else {
        inner_note_hash
    };

    let siloed_note_hash = compute_siloed_hash(note_header.contract_address, unique_note_hash);

    let inner_nullifier = note.compute_nullifier_without_context();
    // docs:start:compute_note_hash_and_nullifier_returns
    [inner_note_hash, unique_note_hash, siloed_note_hash, inner_nullifier]
    // docs:end:compute_note_hash_and_nullifier_returns
}
