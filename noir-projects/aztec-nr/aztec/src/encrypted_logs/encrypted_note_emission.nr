use crate::{
    context::PrivateContext, note::{note_emission::NoteEmission, note_interface::NoteInterface},
    encrypted_logs::payload::compute_encrypted_note_log, oracle::logs_traits::LensForEncryptedLog
};
use dep::protocol_types::{
    address::AztecAddress, grumpkin_point::GrumpkinPoint, abis::note_hash::NoteHash,
    constants::MAX_NEW_NOTE_HASHES_PER_CALL, utils::arrays::find_index
};

fn emit_with_keys<Note, N, NB, M>(
    context: &mut PrivateContext,
    note: Note,
    ovpk: GrumpkinPoint,
    ivpk: GrumpkinPoint
) where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    let note_header = note.get_header();
    let note_hash_counter = note_header.note_hash_counter;
    let storage_slot = note_header.storage_slot;

    let note_exists_index = find_index(
        context.new_note_hashes.storage,
        |n: NoteHash| n.counter == note_hash_counter
    );
    assert(
        note_exists_index as u32 != MAX_NEW_NOTE_HASHES_PER_CALL, "Can only emit a note log for an existing note."
    );

    let contract_address: AztecAddress = context.this_address();
    let ovsk_app: Field  = context.request_ovsk_app(ovpk.hash());

    let encrypted_log: [u8; M] = compute_encrypted_note_log(contract_address, storage_slot, ovsk_app, ovpk, ivpk, note);

    context.emit_raw_note_log(note_hash_counter, encrypted_log);
}

pub fn encode_and_encrypt_note<Note, N, NB, M>(
    context: &mut PrivateContext,
    ov: AztecAddress,
    iv: AztecAddress
) -> fn[(&mut PrivateContext, AztecAddress, AztecAddress)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        let header = context.get_header();
        let ovpk = header.get_ovpk_m(context, ov);
        let ivpk = header.get_ivpk_m(context, iv);
        emit_with_keys(context, e.note, ovpk, ivpk);
    }
}

pub fn encode_and_encrypt_note_with_keys<Note, N, NB, M>(
    context: &mut PrivateContext,
    ovpk: GrumpkinPoint,
    ivpk: GrumpkinPoint
) -> fn[(&mut PrivateContext, GrumpkinPoint, GrumpkinPoint)](NoteEmission<Note>) -> () where Note: NoteInterface<N, NB>, [Field; N]: LensForEncryptedLog<N, M> {
    | e: NoteEmission<Note> | {
        emit_with_keys(context, e.note, ovpk, ivpk);
    }
}
