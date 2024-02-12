use crate::{context::PrivateContext, note::{note_header::NoteHeader, note_interface::NoteInterface}};

use dep::protocol_types::{
    address::AztecAddress,
    constants::{
    GENERATOR_INDEX__OUTER_NULLIFIER, GENERATOR_INDEX__UNIQUE_COMMITMENT,
    GENERATOR_INDEX__SILOED_COMMITMENT
},
    hash::pedersen_hash, utils::arr_copy_slice
};

fn compute_siloed_hash(contract_address: AztecAddress, inner_note_hash: Field) -> Field {
    let inputs = [contract_address.to_field(), inner_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__SILOED_COMMITMENT)
}

fn compute_unique_hash(nonce: Field, siloed_note_hash: Field) -> Field {
    let inputs = [nonce, siloed_note_hash];
    pedersen_hash(inputs, GENERATOR_INDEX__UNIQUE_COMMITMENT)
}

fn compute_inner_note_hash<Note, N>(note: Note) -> Field where Note: NoteInterface<N> {
    let header = note.get_header();
    let note_hash = note.compute_note_content_hash();

    // TODO(#1205) Do we need a generator index here?
    pedersen_hash([header.storage_slot, note_hash], 0)
}

fn compute_siloed_note_hash<Note, N>(note_with_header: Note) -> Field where Note: NoteInterface<N> {
    let header = note_with_header.get_header();

    let inner_note_hash = compute_inner_note_hash(note_with_header);

    compute_siloed_hash(header.contract_address, inner_note_hash)
}

fn compute_unique_siloed_note_hash<Note, N>(note_with_header: Note) -> Field where Note: NoteInterface<N> {
    let header = note_with_header.get_header();

    let siloed_note_hash = compute_siloed_note_hash(note_with_header);

    compute_unique_hash(header.nonce, siloed_note_hash)
}

pub fn compute_siloed_nullifier<Note, N>(
    note_with_header: Note,
    context: &mut PrivateContext
) -> Field where Note: NoteInterface<N> {
    let header = note_with_header.get_header();
    let inner_nullifier = note_with_header.compute_nullifier(context);

    let input = [header.contract_address.to_field(), inner_nullifier];
    pedersen_hash(input, GENERATOR_INDEX__OUTER_NULLIFIER)
}

pub fn compute_note_hash_for_insertion<Note, N>(note: Note) -> Field where Note: NoteInterface<N> {
    compute_inner_note_hash(note)
}

pub fn compute_note_hash_for_consumption<Note, N>(note: Note) -> Field where Note: NoteInterface<N> {
    let header = note.get_header();
    // There are 3 cases for reading a note intended for consumption:
    // 1. The note was inserted in this transaction, and is transient.
    // 2. The note was inserted in a previous transaction, and was inserted in public
    // 3. The note was inserted in a previous transaction, and was inserted in private

    if (header.is_transient) {
        // If a note is transient, we just read the inner_note_hash (kernel will silo by contract address).
        compute_inner_note_hash(note)
    } else if (header.nonce == 0) {
        // If not transient and nonce is zero, that means we are reading a public note.
        compute_siloed_note_hash(note)
    } else {
        // When nonce is nonzero, that means we are reading a settled note (from tree) created in a
        // previous TX. So we need the unique_siloed_note_hash which has already been hashed with
        // contract address and then nonce. This hash will match the existing leaf in the private
        // data tree, so the kernel can just perform a membership check directly on this hash/leaf.
        compute_unique_siloed_note_hash(note)
    }
}

pub fn compute_note_hash_and_nullifier<T, N, S>(
    deserialize_content: fn([Field; N]) -> T,
    note_header: NoteHeader,
    serialized_note: [Field; S]
) -> [Field; 4] where T: NoteInterface<N> {
    let mut note = deserialize_content(arr_copy_slice(serialized_note, [0; N], 0));
    // TODO: change this to note.setHeader(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
    T::set_header((&mut note), note_header);

    let inner_note_hash = compute_inner_note_hash(note);

    let siloed_note_hash = compute_siloed_hash(note_header.contract_address, inner_note_hash);

    let unique_siloed_note_hash = compute_unique_hash(note_header.nonce, siloed_note_hash);

    let inner_nullifier = note.compute_nullifier_without_context();

    [inner_note_hash, siloed_note_hash, unique_siloed_note_hash, inner_nullifier]
}
