use dep::aztec::context::PrivateContext;
use dep::aztec::log::emit_encrypted_log;
use dep::aztec::note::note_getter_options::NoteGetterOptions;
use dep::aztec::oracle::get_public_key::get_public_key;
use dep::aztec::state_vars::set::Set;
use dep::aztec::types::point::Point;
use crate::{
    filter::get_2_notes,
    filter::get_4_notes,
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
    note_offset: Field,
    spend_note_index: Field,
) {
    let options = NoteGetterOptions::with_filter_and_offset(get_4_notes, 0, note_offset as u32);
    let maybe_notes = balance.get_notes(context, options);

    let note0 = maybe_notes[0].unwrap_or(ValueNote::dummy());
    let note1 = maybe_notes[1].unwrap_or(ValueNote::dummy());
    let note2 = maybe_notes[2].unwrap_or(ValueNote::dummy());
    let note3 = maybe_notes[3].unwrap_or(ValueNote::dummy());

    assert(spend_note_index as u64 < 4);
    
    // Check that the note being spent is actually owned by the owner.
    // Removes only the note that is meant to be actually spent from the owner's notes.
    let mut note_value = 0;
    if spend_note_index == 0 {
        if maybe_notes[0].is_some() {
            assert(owner == note0.owner);
        }
        note_value = note0.value;
        balance.remove(context, note0);
    } else if spend_note_index == 1 {
        if maybe_notes[1].is_some() {
            assert(owner == note1.owner);
        }
        note_value = note1.value;
        balance.remove(context, note1);
    } else if spend_note_index == 2 {
        if maybe_notes[2].is_some() {
            assert(owner == note2.owner);
        }
        note_value = note2.value;
        balance.remove(context, note2);
    } else {
        if maybe_notes[3].is_some() {
            assert(owner == note3.owner);
        }
        note_value = note3.value;
        balance.remove(context, note3);
    }
    
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
    let encryption_pub_key = get_public_key(recipient);
    emit_encrypted_log(
        context,
        (*context).this_address(),
        balance.storage_slot,
        encryption_pub_key,
        note.serialise(),
    );
}

/*
  Sends three amounts to three recipients.
  Why three? Because one private call currently allows `MAX_NEW_COMMITMENTS_PER_CALL = 4` output commitments.
  So we split the output notes as: 3 to recipients + 1 to the owner (the change note).
*/
fn send_notes(
    context: &mut PrivateContext,
    recipient_balances: [Set<ValueNote, VALUE_NOTE_LEN>; 3],
    amounts: [Field; 3], 
    recipients: [Field; 3],
) {
    for i in 0..3 {
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