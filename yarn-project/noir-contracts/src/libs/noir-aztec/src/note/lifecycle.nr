use crate::context::Context;
use crate::note::note_hash::compute_final_note_hash;
use crate::note::note_hash::compute_siloed_note_hash;
use crate::note::note_interface::NoteInterface;
use crate::oracle::notes::notify_created_note;
use crate::oracle::notes::notify_nullified_note;

fn create_note<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note: Note,
    note_interface: NoteInterface<Note, N>,
) -> Context {
    let mut siloed_note_hash = 0;
    let is_dummy = note_interface.is_dummy;
    let mut notify_result = 0; // TODO(https://github.com/noir-lang/noir/pull/1729): remove me (more below)
    if is_dummy(note) == false {
        let compute_note_hash = note_interface.compute_note_hash;
        let note_hash = compute_note_hash(note);
        siloed_note_hash = compute_siloed_note_hash(storage_slot, note_hash);

        let serialise = note_interface.serialise;
        let preimage = serialise(note);
        let mut myfield = 0;
        notify_result = notify_created_note(storage_slot, preimage);
        assert(notify_result == 0); // TODO(https://github.com/noir-lang/noir/pull/1729): combine with above line (more below)
    }
    // TODO(https://github.com/noir-lang/noir/pull/1729): remove when stepwise execution of acir is complete
    // replace with: `assert(notify_created_note(storage_slot, preimage) == 0);`
    context.oracle_connector = notify_result; // to force dependency/ordering between oracle calls

    context.push_new_note_hash(siloed_note_hash)
}

fn destroy_note<Note, N>(
    mut context: Context,
    storage_slot: Field,
    note: Note,
    note_interface: NoteInterface<Note, N>,
) -> Context {
    let mut nullifier = 0;
    let is_dummy = note_interface.is_dummy;
    if is_dummy(note) == false {
        let compute_note_hash = note_interface.compute_note_hash;
        let note_hash = compute_note_hash(note);
        let final_note_hash = compute_final_note_hash(storage_slot, note_hash);
        let compute_nullifier = note_interface.compute_nullifier;
        nullifier = compute_nullifier(note, final_note_hash);

        let serialise = note_interface.serialise;
        let preimage = serialise(note);
        assert(notify_nullified_note(storage_slot, nullifier, preimage) == 0);
    }

    context.push_new_nullifier(nullifier)
}