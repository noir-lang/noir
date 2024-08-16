use crate::{
    context::PrivateContext, note::{note_emission::NoteEmission, note_interface::NoteInterface},
    keys::getters::get_current_public_keys, encrypted_logs::payload::compute_encrypted_note_log,
    oracle::logs_traits::LensForEncryptedLog
};
use dep::protocol_types::{
    hash::sha256_to_field, address::AztecAddress, point::Point, abis::note_hash::NoteHash,
    constants::MAX_NOTE_HASHES_PER_CALL
};

unconstrained fn compute_unconstrained<Note, let N: u32, let NB: u32, let M: u32>(
    contract_address: AztecAddress,
    storage_slot: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress,
    note: Note
) -> ([u8; M], Field) where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    compute(
        contract_address,
        storage_slot,
        ovsk_app,
        ovpk,
        ivpk,
        recipient,
        note
    )
}

fn compute<Note, let N: u32, let NB: u32, let M: u32>(
    contract_address: AztecAddress,
    storage_slot: Field,
    ovsk_app: Field,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress,
    note: Note
) -> ([u8; M], Field) where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    let encrypted_log: [u8; M] = compute_encrypted_note_log(
        contract_address,
        storage_slot,
        ovsk_app,
        ovpk,
        ivpk,
        recipient,
        note
    );
    let log_hash = sha256_to_field(encrypted_log);
    (encrypted_log, log_hash)
}

fn emit_with_keys<Note, let N: u32, let NB: u32, let M: u32>(
    context: &mut PrivateContext,
    note: Note,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress,
    inner_compute: fn(AztecAddress, Field, Field, Point, Point, AztecAddress, Note) -> ([u8; M], Field)
) where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    let note_header = note.get_header();
    let note_hash_counter = note_header.note_hash_counter;
    let storage_slot = note_header.storage_slot;

    let note_exists = context.note_hashes.storage.any(|n: NoteHash| n.counter == note_hash_counter);
    assert(note_exists, "Can only emit a note log for an existing note.");

    let contract_address: AztecAddress = context.this_address();
    let ovsk_app: Field  = context.request_ovsk_app(ovpk.hash());

    let (encrypted_log, log_hash) = inner_compute(
        contract_address,
        storage_slot,
        ovsk_app,
        ovpk,
        ivpk,
        recipient,
        note
    );

    context.emit_raw_note_log(note_hash_counter, encrypted_log, log_hash);
}

pub fn encode_and_encrypt_note<Note, let N: u32, let NB: u32, let M: u32>(
    context: &mut PrivateContext,
    ov: AztecAddress,
    iv: AztecAddress
) -> fn[(&mut PrivateContext, AztecAddress, AztecAddress)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        let ovpk = get_current_public_keys(context, ov).ovpk_m;
        let ivpk = get_current_public_keys(context, iv).ivpk_m;
        emit_with_keys(context, e.note, ovpk, ivpk, iv, compute);
    }
}

pub fn encode_and_encrypt_note_unconstrained<Note, let N: u32, let NB: u32, let M: u32>(
    context: &mut PrivateContext,
    ov: AztecAddress,
    iv: AztecAddress
) -> fn[(&mut PrivateContext, AztecAddress, AztecAddress)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        let ovpk = get_current_public_keys(context, ov).ovpk_m;
        let ivpk = get_current_public_keys(context, iv).ivpk_m;
        emit_with_keys(context, e.note, ovpk, ivpk, iv, compute_unconstrained);
    }
}

pub fn encode_and_encrypt_note_with_keys<Note, let N: u32, let NB: u32, let M: u32>(
    context: &mut PrivateContext,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress
) -> fn[(&mut PrivateContext, Point, Point, AztecAddress)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        emit_with_keys(context, e.note, ovpk, ivpk, recipient, compute);
    }
}

pub fn encode_and_encrypt_note_with_keys_unconstrained<Note, let N: u32, let NB: u32, let M: u32>(
    context: &mut PrivateContext,
    ovpk: Point,
    ivpk: Point,
    recipient: AztecAddress
) -> fn[(&mut PrivateContext, Point, Point, AztecAddress)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        emit_with_keys(context, e.note, ovpk, ivpk, recipient, compute_unconstrained);
    }
}
