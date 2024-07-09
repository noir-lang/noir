use dep::aztec::prelude::{AztecAddress, PrivateContext, PrivateSet, NoteGetterOptions};
use dep::aztec::note::note_getter_options::SortOrder;
use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note;
use crate::{filter::filter_notes_min_sum, value_note::{ValueNote, VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN}};

// Sort the note values (0th field) in descending order.
// Pick the fewest notes whose sum is equal to or greater than `amount`.
pub fn create_note_getter_options_for_decreasing_balance(amount: Field) -> NoteGetterOptions<ValueNote, VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN, Field> {
    NoteGetterOptions::with_filter(filter_notes_min_sum, amount).sort(ValueNote::properties().value, SortOrder.DESC)
}

// Creates a new note for the recipient.
// Inserts it to the recipient's set of notes.
pub fn increment(
    balance: PrivateSet<ValueNote, &mut PrivateContext>,
    amount: Field,
    recipient: AztecAddress,
    outgoing_viewer: AztecAddress
) {
    let header = balance.context.get_header();
    let recipient_npk_m_hash = header.get_npk_m_hash(balance.context, recipient);

    let mut note = ValueNote::new(amount, recipient_npk_m_hash);
    // Insert the new note to the owner's set of notes and emit the log if value is non-zero.
    balance.insert(&mut note).emit(encode_and_encrypt_note(balance.context, outgoing_viewer, recipient));
}

// Find some of the `owner`'s notes whose values add up to the `amount`.
// Remove those notes.
// If the value of the removed notes exceeds the requested `amount`, create a new note containing the excess value, so that exactly `amount` is removed.
// Fail if the sum of the selected notes is less than the amount.
pub fn decrement(
    balance: PrivateSet<ValueNote, &mut PrivateContext>,
    amount: Field,
    owner: AztecAddress,
    outgoing_viewer: AztecAddress
) {
    let sum = decrement_by_at_most(balance, amount, owner, outgoing_viewer);
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
pub fn decrement_by_at_most(
    balance: PrivateSet<ValueNote, &mut PrivateContext>,
    max_amount: Field,
    owner: AztecAddress,
    outgoing_viewer: AztecAddress
) -> Field {
    let options = create_note_getter_options_for_decreasing_balance(max_amount);
    let notes = balance.get_notes(options);

    let mut decremented = 0;
    for i in 0..options.limit {
        if i < notes.len() {
            let note = notes.get_unchecked(i);

            decremented += destroy_note(balance, note);
        }
    }

    // Add the change value back to the owner's balance.
    let mut change_value = 0;
    if max_amount.lt(decremented) {
        change_value = decremented - max_amount;
        decremented -= change_value;
    }
    increment(balance, change_value, owner, outgoing_viewer);

    decremented
}

// Removes the note from the owner's set of notes.
// Returns the value of the destroyed note.
pub fn destroy_note(balance: PrivateSet<ValueNote, &mut PrivateContext>, note: ValueNote) -> Field {
    balance.remove(note);

    note.value
}
