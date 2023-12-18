use dep::std::option::Option;
use dep::aztec::context::PrivateContext;
use dep::aztec::note::note_getter_options::{NoteGetterOptions, SortOrder};
use dep::aztec::oracle::get_public_key::get_public_key;
use dep::aztec::state_vars::set::Set;
use crate::{
    filter::filter_notes_min_sum,
    value_note::{ValueNote, VALUE_NOTE_LEN},
};
use dep::protocol_types::address::AztecAddress;

// Sort the note values (0th field) in descending order.
// Pick the fewest notes whose sum is equal to or greater than `amount`.
pub fn create_note_getter_options_for_decreasing_balance(amount: Field) -> NoteGetterOptions<ValueNote, VALUE_NOTE_LEN, Field> {
    NoteGetterOptions::with_filter(filter_notes_min_sum, amount).sort(0, SortOrder.DESC)
}

// Creates a new note for the recipient.
// Inserts it to the recipient's set of notes.
pub fn increment(balance: Set<ValueNote, VALUE_NOTE_LEN>, amount: Field, recipient: AztecAddress) {
    let mut note = ValueNote::new(amount, recipient);
    // Insert the new note to the owner's set of notes and emit the log if value is non-zero.
    balance.insert(&mut note, amount != 0);
}

// Find some of the `owner`'s notes whose values add up to the `amount`.
// Remove those notes.
// If the value of the removed notes exceeds the requested `amount`, create a new note containing the excess value, so that exactly `amount` is removed.
// Fail if the sum of the selected notes is less than the amount.
pub fn decrement(balance: Set<ValueNote, VALUE_NOTE_LEN>, amount: Field, owner: AztecAddress) {
    let sum = decrement_by_at_most(balance, amount, owner);
    assert(sum == amount, "Balance too low");
}

// Similar to `decrement`, except that it doesn't fail if the decremented amount is less than max_amount.
// The motivation behind this function is that there is an upper-bound on the number of notes a function may
// read and nullify. The requested decrementation `amount` might be spread across too many of the `owner`'s
// notes to 'fit' within this upper-bound, so we might have to remove an amount less than `amount`. A common
// pattern is to repeatedly call this function across many function calls, until enough notes have been nullified to
// equal `amount`.
//
// It returns the decremented amount, which should be less than or equal to max_amount.
pub fn decrement_by_at_most(balance: Set<ValueNote, VALUE_NOTE_LEN>, max_amount: Field, owner: AztecAddress) -> Field {
    let options = create_note_getter_options_for_decreasing_balance(max_amount);
    let opt_notes = balance.get_notes(options);

    let mut decremented = 0;
    for i in 0..opt_notes.len() {
        if opt_notes[i].is_some() {
            decremented += destroy_note(balance, owner, opt_notes[i].unwrap_unchecked());
        }
    }

    // Add the change value back to the owner's balance.
    let mut change_value = 0;
    if decremented as u120 > max_amount as u120 {
        change_value = decremented - max_amount;
        decremented -= change_value;
    }
    increment(balance, change_value, owner);

    decremented
}

// Removes the note from the owner's set of notes.
// Returns the value of the destroyed note.
pub fn destroy_note(balance: Set<ValueNote, VALUE_NOTE_LEN>, owner: AztecAddress, note: ValueNote) -> Field {
    // Ensure the note is actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    assert(note.owner.eq(owner));

    balance.remove(note);

    note.value
}
