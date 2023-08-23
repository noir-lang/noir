use dep::aztec::context::PrivateContext;
use dep::aztec::log::emit_encrypted_log;
use dep::aztec::note::note_getter_options::NoteGetterOptions;
use dep::aztec::oracle::get_public_key::get_public_key;
use dep::aztec::state_vars::set::Set;
use dep::aztec::types::point::Point;
use crate::{
    filter::filter_notes_min_sum,
    value_note::{ValueNote, VALUE_NOTE_LEN},
};

fn spend_notes(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    owner: Field,
) {
    let options = NoteGetterOptions::with_filter(filter_notes_min_sum, amount);
    let opt_notes = balance.get_notes(context, options);

    let mut sum = 0;
    for i in 0..opt_notes.len() {
        if opt_notes[i].is_some() {
            let note = opt_notes[i].unwrap_unchecked();

            // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
            // spending someone else's notes).
            assert(note.owner == owner);

            // Removes the note from the owner's set of notes.
            balance.remove(context, note);

            sum += note.value;
        }
    }

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

/*
  Spends one note from a set of 4 notes.
  Details: Reads 4 notes from the user's `balance`: [n_{o}, n_{o+1}, n_{o+2}, n_{o+3}]
  where "o" is the note_offset. Then, spends the note: [n_{o + i}] 
  where "i" is the spend_note_index. This gives more control to the user on which
  note is to be spent. 
*/
fn spend_one_note(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    owner: Field,
    spend_note_offset: u32,
) {
    let options = NoteGetterOptions::new().set_limit(1).set_offset(spend_note_offset);
    let opt_notes = balance.get_notes(context, options);

    // The note should always exist.
    let note = opt_notes[0].unwrap();

    // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    assert(owner == note.owner);

    // Removes the note from the owner's set of notes.
    balance.remove(context, note);

    let note_value = note.value;
    
    // Assert that the note chosen to spend has enough funds.
    assert(note_value as u64 >= amount as u64);

    // Creates change note for the owner.
    let change_value = note_value - amount;
    let mut change_note = ValueNote::new(change_value, owner);

    // Insert the change note to the owner's sets of notes.
    balance.insert(context, &mut change_note);

    // Emit the newly created encrypted note preimages via oracle calls.
    let mut encrypted_data = [0; VALUE_NOTE_LEN];
    if change_value != 0 {
        encrypted_data = change_note.serialise();
    };

    let encryption_pub_key = get_public_key(owner);
    emit_encrypted_log(
        context,
        context.inputs.call_context.storage_contract_address,
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
    // docs:start:encrypted

    let application_contract_address = (*context).this_address();
    let note_storage_slot = balance.storage_slot;
    let encryption_pub_key = get_public_key(recipient);
    let encrypted_data = note.serialise();

    emit_encrypted_log(
        context,
        application_contract_address,
        note_storage_slot,
        encryption_pub_key,
        encrypted_data,
    );

    // docs:end:encrypted
}

/*
  Sends three amounts to three recipients.
  Why three? Because one private call currently allows `MAX_NEW_COMMITMENTS_PER_CALL = 4` output commitments.
  So we split the output notes as: 3 to recipients + 1 to the owner (the change note).
*/
fn send_notes<NUM_RECIPIENTS>(
    context: &mut PrivateContext,
    recipient_balances: [Set<ValueNote, VALUE_NOTE_LEN>; NUM_RECIPIENTS],
    amounts: [Field; NUM_RECIPIENTS], 
    recipients: [Field; NUM_RECIPIENTS],
) {
    for i in 0..recipients.len() {
        // Creates a new note for the i-th recipients
        let mut recipient_note = ValueNote::new(amounts[i], recipients[i]);

        // Insert the new notes to the i-th recipient's sets of notes.
        recipient_balances[i].insert(context, &mut recipient_note);

        // Get recipient encryption keys.
        let recipient_encryption_pub_key = get_public_key(recipients[i]);

        // Emit the newly created encrypted note preimages via oracle calls.
        let mut recipient_encrypted_data = [0; VALUE_NOTE_LEN];
        if recipient_note.value != 0 {
            recipient_encrypted_data = recipient_note.serialise();
        };
        emit_encrypted_log(
            context,
            context.inputs.call_context.storage_contract_address,
            recipient_balances[i].storage_slot,
            recipient_encryption_pub_key,
            recipient_encrypted_data,
        );
    }
}