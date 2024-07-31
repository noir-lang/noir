use dep::aztec::prelude::{
    AztecAddress, NoteGetterOptions, NoteViewerOptions, NoteHeader, NoteInterface, PrivateContext,
    PrivateSet, Map
};
use dep::aztec::{
    hash::pedersen_hash, protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    note::{
    note_getter::view_notes, note_getter_options::{SortOrder, Comparator},
    note_emission::OuterNoteEmission
},
    context::UnconstrainedContext
};
use crate::types::token_note::{TokenNote, OwnedNote};

struct BalancesMap<T, Context> {
    map: PrivateSet<T, Context>
}

impl<T, Context> BalancesMap<T, Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { map: PrivateSet::new(context, storage_slot) }
    }
}

impl<T> BalancesMap<T, UnconstrainedContext> {
    unconstrained fn balance_of<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m_hash: Field
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        self.balance_of_with_offset(owner_npk_m_hash, 0)
    }

    unconstrained fn balance_of_with_offset<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m_hash: Field,
        offset: u32
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        let mut balance = U128::from_integer(0);
        let mut options = NoteViewerOptions::new();

        let notes = self.map.view_notes(
            options.select(
                T::get_owner_selector(),
                owner_npk_m_hash,
                Option::some(Comparator.EQ)
            )
        );

        for i in 0..options.limit {
            if i < notes.len() {
                balance = balance + notes.get_unchecked(i).get_amount();
            }
        }
        if (notes.len() == options.limit) {
            balance = balance + self.balance_of_with_offset(owner_npk_m_hash, offset + options.limit);
        }

        balance
    }
}

impl<T> BalancesMap<T, &mut PrivateContext> {

    pub fn to_unconstrained(self: Self) -> BalancesMap<T, UnconstrainedContext> {
        BalancesMap { map: PrivateSet::new(UnconstrainedContext::new(), self.map.storage_slot) }
    }

    pub fn add<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m_hash: Field,
        addend: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        let mut addend_note = T::new(addend, owner_npk_m_hash);
        OuterNoteEmission::new(Option::some(self.map.insert(&mut addend_note)))
    }

    pub fn sub<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner_npk_m_hash: Field,
        subtrahend: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        let mut options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend);
        let notes = self.map.get_notes(
            options.select(
                T::get_owner_selector(),
                owner_npk_m_hash,
                Option::some(Comparator.EQ)
            )
        );

        let mut minuend: U128 = U128::from_integer(0);
        for i in 0..options.limit {
            if i < notes.len() {
                let note = notes.get_unchecked(i);

                // Removes the note from the owner's set of notes.
                // This will call the the `compute_nullifer` function of the `token_note`
                // which require knowledge of the secret key (currently the users encryption key).
                // The contract logic must ensure that the spending key is used as well.
                // docs:start:remove
                self.map.remove(note);
                // docs:end:remove

                minuend = minuend + note.get_amount();
            }
        }

        // This is to provide a nicer error msg,
        // without it minuend-subtrahend would still catch it, but more generic error then.
        // without the == true, it includes 'minuend.ge(subtrahend)' as part of the error.
        assert(minuend >= subtrahend, "Balance too low");

        self.add(owner_npk_m_hash, minuend - subtrahend)
    }
}

pub fn filter_notes_min_sum<T, T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
    notes: [Option<T>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    min_sum: U128
) -> [Option<T>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
    let mut selected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    let mut sum = U128::from_integer(0);
    for i in 0..notes.len() {
        if notes[i].is_some() & sum < min_sum {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum = sum.add(note.get_amount());
        }
    }
    selected
}
