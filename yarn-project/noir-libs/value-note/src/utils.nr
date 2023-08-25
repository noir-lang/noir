use dep::std::option::Option;
use dep::aztec::context::PrivateContext;
// docs:start:encrypted_import

use dep::aztec::log::emit_encrypted_log;

// docs:end:encrypted_import
use dep::aztec::note::note_getter_options::{NoteGetterOptions, SortOrder};
use dep::aztec::oracle::get_public_key::get_public_key;
use dep::aztec::state_vars::set::Set;
use crate::{
    filter::filter_notes_min_sum,
    value_note::{ValueNote, VALUE_NOTE_LEN},
};

// Sort the note values (0th field) in descending order.
// Pick the fewest notes whose sum is equal to or greater than `amount`.
fn create_note_getter_options_for_decreasing_balance(amount: Field) -> NoteGetterOptions<ValueNote, VALUE_NOTE_LEN, Field> {
    NoteGetterOptions::with_filter(filter_notes_min_sum, amount).sort(0, SortOrder.DESC)
}

// Creates a new note for the recipient.
// Inserts it to the recipient's set of notes.
fn increment(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    recipient: Field,
) {
    let mut note = ValueNote::new(amount, recipient);
    create_note(context, balance, recipient, &mut note);

    // It won't compile if Set.insert() is in an if statement :(
    // if amount as u120 > 0 {
    //     create_note(context, balance, recipient, &mut note);
    // }
}

// Find some of the `owner`'s notes whose values add up to the `amount`.
// Remove those notes.
// If the value of the removed notes exceeds the requested `amount`, create a new note containing the excess value, so that exactly `amount` is removed.
// Fail if the sum of the selected notes is less than the amount.
fn decrement(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    amount: Field,
    owner: Field,
) {
    let sum = decrement_by_at_most(context, balance, amount, owner);
    assert(sum == amount);
}

// Similar to `decrement`, except that it doesn't fail if the decremented amount is less than max_amount.
// The motivation behind this function is that there is an upper-bound on the number of notes a function may
// read and nullify. The requested decrementation `amount` might be spread across too many of the `owner`'s
// notes to 'fit' within this upper-bound, so we might have to remove an amount less than `amount`. A common
// pattern is to repeatedly call this function across many function calls, until enough notes have been nullified to
// equal `amount`.
//
// It returns the decremented amount, which should be less than or equal to max_amount.
fn decrement_by_at_most(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    max_amount: Field,
    owner: Field,
) -> Field {
    let options = create_note_getter_options_for_decreasing_balance(max_amount);
    let opt_notes = balance.get_notes(context, options);

    let mut decremented = 0;
    for i in 0..opt_notes.len() {
        if opt_notes[i].is_some() {
            decremented += destroy_note(context, balance, owner, opt_notes[i].unwrap_unchecked());
        }
    }

    // Add the change value back to the owner's balance.
    let mut change_value = 0;
    if decremented as u120 > max_amount as u120 {
        change_value = decremented - max_amount;
        decremented -= change_value;
    }
    increment(context, balance, change_value, owner);

    decremented
}

fn create_note(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    owner: Field,
    note: &mut ValueNote,
) {
    // Insert the new note to the owner's set of notes.
    balance.insert(context, note);

    // Remove this if statement if we can wrap this create_note function in an if statement.
    if note.value != 0 {
        // Emit the newly created encrypted note preimages via oracle calls.
        // docs:start:encrypted
        let application_contract_address = (*context).this_address();
        let note_storage_slot = balance.storage_slot;
        let encryption_pub_key = get_public_key(owner);
        let encrypted_data = (*note).serialise();

        emit_encrypted_log(
            context,
            application_contract_address,
            note_storage_slot,
            encryption_pub_key,
            encrypted_data,
        );
        // docs:end:encrypted
    }
}

// Removes the note from the owner's set of notes.
// Returns the value of the destroyed note.
fn destroy_note(
    context: &mut PrivateContext,
    balance: Set<ValueNote, VALUE_NOTE_LEN>,
    owner: Field,
    note: ValueNote,
) -> Field {
    // Ensure the note is actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    assert(note.owner == owner);

    balance.remove(context, note);
    
    note.value
}
