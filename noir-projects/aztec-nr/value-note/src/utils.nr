use dep::aztec::prelude::{AztecAddress, PrivateContext, PrivateSet, NoteGetterOptions};
use dep::aztec::note::note_getter_options::SortOrder;
use dep::aztec::protocol_types::grumpkin_point::GrumpkinPoint;
use dep::aztec::keys::getters::{get_npk_m_hash, get_ivpk_m};
use crate::{filter::filter_notes_min_sum, value_note::{ValueNote, VALUE_NOTE_LEN}};

// Sort the note values (0th field) in descending order.
// Pick the fewest notes whose sum is equal to or greater than `amount`.
pub fn create_note_getter_options_for_decreasing_balance(amount: Field) -> NoteGetterOptions<ValueNote, VALUE_NOTE_LEN, Field> {
    NoteGetterOptions::with_filter(filter_notes_min_sum, amount).sort(ValueNote::properties().value, SortOrder.DESC)
}

// Creates a new note for the recipient.
// Inserts it to the recipient's set of notes.
pub fn increment(balance: PrivateSet<ValueNote>, amount: Field, recipient: AztecAddress) {
    let context = balance.context.private.unwrap();
    let recipient_npk_m_hash = get_npk_m_hash(context, recipient);
    let recipient_ivpk_m = get_ivpk_m(context, recipient);

    let mut note = ValueNote::new(amount, recipient_npk_m_hash);
    // Insert the new note to the owner's set of notes and emit the log if value is non-zero.
    balance.insert(&mut note, amount != 0, recipient_ivpk_m);
}

// Find some of the `owner`'s notes whose values add up to the `amount`.
// Remove those notes.
// If the value of the removed notes exceeds the requested `amount`, create a new note containing the excess value, so that exactly `amount` is removed.
// Fail if the sum of the selected notes is less than the amount.
pub fn decrement(balance: PrivateSet<ValueNote>, amount: Field, owner: AztecAddress) {
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
pub fn decrement_by_at_most(
    balance: PrivateSet<ValueNote>,
    max_amount: Field,
    owner: AztecAddress
) -> Field {
    let options = create_note_getter_options_for_decreasing_balance(max_amount);
    let opt_notes = balance.get_notes(options);

    let owner_npk_m_hash = get_npk_m_hash(balance.context.private.unwrap(), owner);

    let mut decremented = 0;
    for i in 0..opt_notes.len() {
        if opt_notes[i].is_some() {
            let note = opt_notes[i].unwrap_unchecked();

            // This is similar to destroy_note, except we only compute the owner_npk_m_hash once instead of doing it in
            // each loop iteration.

            // Ensure the note is actually owned by the owner (to prevent user from generating a valid proof while
            // spending someone else's notes).
            // TODO (#6312): This will break with key rotation. Fix this. Will not be able to pass this after rotating keys.
            assert(note.npk_m_hash.eq(owner_npk_m_hash));
            decremented += note.value;

            balance.remove(note);
        }
    }

    // Add the change value back to the owner's balance.
    let mut change_value = 0;
    if max_amount.lt(decremented) {
        change_value = decremented - max_amount;
        decremented -= change_value;
    }
    increment(balance, change_value, owner);

    decremented
}

// Removes the note from the owner's set of notes.
// Returns the value of the destroyed note.
pub fn destroy_note(balance: PrivateSet<ValueNote>, owner: AztecAddress, note: ValueNote) -> Field {
    // Ensure the note is actually owned by the owner (to prevent user from generating a valid proof while
    // spending someone else's notes).
    let owner_npk_m_hash = get_npk_m_hash(balance.context.private.unwrap(), owner);

    // TODO (#6312): This will break with key rotation. Fix this. Will not be able to pass this after rotating keys.
    assert(note.npk_m_hash.eq(owner_npk_m_hash));

    balance.remove(note);

    note.value
}
