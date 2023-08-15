use dep::aztec::context::PrivateContext;
use dep::aztec::log::emit_encrypted_log;
use dep::aztec::note::note_getter_options::NoteGetterOptions;
use dep::aztec::oracle::get_public_key::get_public_key;
use dep::aztec::state_vars::set::Set;
use dep::aztec::types::point::Point;
use crate::{
    filter::get_2_notes,
    value_note::{ValueNote, VALUE_NOTE_LEN},
};

fn spend_notes(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    owner: Field,
) {
    let options = NoteGetterOptions::with_filter(get_2_notes, 0);
    let maybe_notes = balance.get_notes(context, options);

    let note0 = maybe_notes[0].unwrap_or(ValueNote::dummy());
    let note1 = maybe_notes[1].unwrap_or(ValueNote::dummy());

    // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    if maybe_notes[0].is_some() {
        assert(owner == note0.owner);
        // Removes the note from the owner's set of notes.
        balance.remove(context, note0);
    }
    if maybe_notes[1].is_some() {
        assert(owner == note1.owner);
        balance.remove(context, note1);
    }

    let sum = note0.value + note1.value;
    assert(sum as u120 >= amount as u120);

    // Creates change note for the owner.
    let change_value = sum - amount;
    let mut change_note = ValueNote::new(change_value, owner);
    balance.insert(context, &mut change_note);

    // Emit the newly created encrypted note preimages via oracle calls.
    let mut encrypted_data = [0; VALUE_NOTE_LEN];
    if change_value != 0 {
        encrypted_data = change_note.serialise();
    };

    let encryption_pub_key = get_public_key(owner);
    emit_encrypted_log(
        context,
        (*context).this_address(),
        balance.storage_slot,
        encryption_pub_key,
        encrypted_data,
    );
}

fn send_note(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    recipient: Field,
) {
     // Creates new note for the recipient.
    let mut note = ValueNote::new(amount, recipient);

    // Insert the new note to the recipient's set of notes.
    balance.insert(context, &mut note);

    // Emit the newly created encrypted note preimages via oracle calls.
    let encryption_pub_key = get_public_key(recipient);
    emit_encrypted_log(
        context,
        (*context).this_address(),
        balance.storage_slot,
        encryption_pub_key,
        note.serialise(),
    );
}