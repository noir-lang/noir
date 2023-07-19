use dep::aztec::context::Context;
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
    mut context: Context,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    owner: Field,
) -> Context {
    let options = NoteGetterOptions::with_filter(get_2_notes, 0);
    let (mut new_context, notes) = balance.get_notes(context, options);
    context = new_context;
    let note1 = notes[0];
    let note2 = notes[1];

    // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    note1.validate(owner);
    note2.validate(owner);

    let sum = note1.value + note2.value;
    assert(sum as u120 >= amount as u120);

    // Removes the 2 notes from the owner's set of notes.
    context = balance.remove(context, note1);
    context = balance.remove(context, note2);

    // Creates change note for the owner.
    let change_value = sum - amount;
    let change_note = ValueNote::new(change_value, owner);
    context = balance.insert(context, change_note);

    // Emit the newly created encrypted note preimages via oracle calls.
    let mut encrypted_data = [0; VALUE_NOTE_LEN];
    if change_note.is_dummy() == false {
        encrypted_data = change_note.serialise();
    };

    let owner_key = get_public_key(owner);
    context = emit_encrypted_log(
        context,
        context.inputs.call_context.storage_contract_address,
        balance.storage_slot,
        owner_key,
        encrypted_data,
    );

    context
}

fn send_note(
    mut context: Context,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    recipient: Field,
) -> Context {
     // Creates new note for the recipient.
    let note = ValueNote::new(amount, recipient);

    // Insert the new note to the recipient's set of notes.
    context = balance.insert(context, note);

    // Emit the newly created encrypted note preimages via oracle calls.
    let recipient_key = get_public_key(recipient);
    context = emit_encrypted_log(
        context,
        context.inputs.call_context.storage_contract_address,
        balance.storage_slot,
        recipient_key,
        note.serialise(),
    );

    context
}