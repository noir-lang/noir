use dep::aztec::prelude::{NoteGetterOptions, NoteViewerOptions, NoteInterface, PrivateSet, Point};
use dep::aztec::{
    context::{PrivateContext, UnconstrainedContext},
    protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    note::{note_getter::view_notes, note_emission::{NoteEmission, OuterNoteEmission}},
    keys::{getters::get_current_public_keys, public_keys::NpkM}
};
use crate::types::token_note::OwnedNote;

struct BalanceSet<T, Context> {
    set: PrivateSet<T, Context>,
}

impl<T, Context> BalanceSet<T, Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { set: PrivateSet::new(context, storage_slot) }
    }
}

impl<T> BalanceSet<T, UnconstrainedContext> {
    unconstrained pub fn balance_of<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(self: Self) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        self.balance_of_with_offset(0)
    }

    unconstrained pub fn balance_of_with_offset<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        offset: u32
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        let mut balance = U128::from_integer(0);
        // docs:start:view_notes
        let mut options = NoteViewerOptions::new();
        let notes = self.set.view_notes(options.set_offset(offset));
        // docs:end:view_notes
        for i in 0..options.limit {
            if i < notes.len() {
                balance = balance + notes.get_unchecked(i).get_amount();
            }
        }
        if (notes.len() == options.limit) {
            balance = balance + self.balance_of_with_offset(offset + options.limit);
        }

        balance
    }
}

impl<T> BalanceSet<T, &mut PrivateContext> {
    pub fn add<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m: NpkM,
        addend: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        if addend == U128::from_integer(0) {
            OuterNoteEmission::new(Option::none())
        } else {
            // We fetch the nullifier public key hash from the registry / from our PXE
            let mut addend_note = T::new(addend, owner_npk_m.hash());

            // docs:start:insert
            OuterNoteEmission::new(Option::some(self.set.insert(&mut addend_note)))
            // docs:end:insert
        }
    }

    pub fn sub<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m: NpkM,
        amount: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        let subtracted = self.try_sub(amount, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL);

        // try_sub may have substracted more or less than amount. We must ensure that we subtracted at least as much as
        // we needed, and then create a new note for the owner for the change (if any).
        assert(subtracted >= amount, "Balance too low");
        self.add(owner_npk_m, subtracted - amount)
    }

    // Attempts to remove 'target_amount' from the owner's balance. try_sub returns how much was actually subtracted
    // (i.e. the sum of the value of nullified notes), but this subtracted amount may be more or less than the target
    // amount.
    // This may seem odd, but is unfortunately unavoidable due to the number of notes available and their amounts being
    // unknown. What try_sub does is a best-effort attempt to consume as few notes as possible that add up to more than
    // `target_amount`.
    // The `max_notes` parameter is used to fine-tune the number of constraints created by this function. The gate count
    // scales relatively linearly with `max_notes`, but a lower `max_notes` parameter increases the likelihood of
    // `try_sub` subtracting an amount smaller than `target_amount`.
    pub fn try_sub<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        target_amount: U128,
        max_notes: u32
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        // We are using a preprocessor here (filter applied in an unconstrained context) instead of a filter because
        // we do not need to prove correct execution of the preprocessor.
        // Because the `min_sum` notes is not constrained, users could choose to e.g. not call it. However, all this
        // might result in is simply higher DA costs due to more nullifiers being emitted. Since we don't care
        // about proving optimal note usage, we can save these constraints and make the circuit smaller.
        let options = NoteGetterOptions::with_preprocessor(preprocess_notes_min_sum, target_amount).set_limit(max_notes);
        let notes = self.set.pop_notes(options);

        let mut subtracted = U128::from_integer(0);
        for i in 0..options.limit {
            if i < notes.len() {
                let note = notes.get_unchecked(i);
                subtracted = subtracted + note.get_amount();
            }
        }

        subtracted
    }
}

// Computes the partial sum of the notes array, stopping once 'min_sum' is reached. This can be used to minimize the
// number of notes read that add to some value, e.g. when transferring some amount of tokens.
// The preprocessor (a filter applied in an unconstrained context) does not check if total sum is larger or equal to
// 'min_sum' - all it does is remove extra notes if it does reach that value.
// Note that proper usage of this preprocessor requires for notes to be sorted in descending order.
pub fn preprocess_notes_min_sum<T, T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
    notes: [Option<T>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    min_sum: U128
) -> [Option<T>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
    let mut selected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    let mut sum = U128::from_integer(0);
    for i in 0..notes.len() {
        // Because we process notes in retrieved order, notes need to be sorted in descending amount order for this
        // filter to be useful. Consider a 'min_sum' of 4, and a set of notes with amounts [3, 2, 1, 1, 1, 1, 1]. If
        // sorted in descending order, the filter will only choose the notes with values 3 and 2, but if sorted in
        // ascending order it will choose 4 notes of value 1.
        if notes[i].is_some() & sum < min_sum {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum = sum.add(note.get_amount());
        }
    }
    selected
}
