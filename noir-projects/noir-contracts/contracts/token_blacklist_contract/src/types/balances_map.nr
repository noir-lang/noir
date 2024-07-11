use dep::aztec::prelude::{AztecAddress, NoteGetterOptions, NoteViewerOptions, NoteHeader, NoteInterface, PrivateSet, Map};
use dep::aztec::{
    context::{PrivateContext, UnconstrainedContext}, hash::pedersen_hash,
    protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    note::{note_getter::view_notes, note_getter_options::SortOrder, note_emission::OuterNoteEmission}
};
use crate::types::{token_note::{TokenNote, OwnedNote}};

struct BalancesMap<T, Context> {
    map: Map<AztecAddress, PrivateSet<T, Context>, Context>
}

impl<T, Context> BalancesMap<T, Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self {
            map: Map::new(
                context,
                storage_slot,
                |context, slot| PrivateSet::new(context, slot)
            )
        }
    }
}

impl<T> BalancesMap<T, UnconstrainedContext> {
    unconstrained pub fn balance_of<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner: AztecAddress
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        self.balance_of_with_offset(owner, 0)
    }

    unconstrained pub fn balance_of_with_offset<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner: AztecAddress,
        offset: u32
    ) -> U128 where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        let mut balance = U128::from_integer(0);
        // docs:start:view_notes
        let mut options = NoteViewerOptions::new();
        let notes = self.map.at(owner).view_notes(options.set_offset(offset));
        // docs:end:view_notes
        for i in 0..options.limit {
            if i < notes.len() {
                balance = balance + notes.get_unchecked(i).get_amount();
            }
        }
        if (notes.len() == options.limit) {
            balance = balance + self.balance_of_with_offset(owner, offset + options.limit);
        }

        balance
    }
}

impl<T> BalancesMap<T, &mut PrivateContext> {
    pub fn add<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner: AztecAddress,
        addend: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote {
        if addend == U128::from_integer(0) {
            OuterNoteEmission::new(Option::none())
        } else {
            let context = self.map.context;
            let header = context.get_header();

            // We fetch the nullifier public key hash from the registry / from our PXE
            let owner_npk_m_hash = header.get_npk_m_hash(context, owner);
            let mut addend_note = T::new(addend, owner_npk_m_hash);

            // docs:start:insert
            OuterNoteEmission::new(Option::some(self.map.at(owner).insert(&mut addend_note)))
            // docs:end:insert
        }
    }

    pub fn sub<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN>(
        self: Self,
        owner: AztecAddress,
        subtrahend: U128
    ) -> OuterNoteEmission<T> where T: NoteInterface<T_SERIALIZED_LEN, T_SERIALIZED_BYTES_LEN> + OwnedNote + Eq {
        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend);
        let notes = self.map.at(owner).get_notes(options);
        // docs:end:get_notes

        let mut minuend: U128 = U128::from_integer(0);
        for i in 0..options.limit {
            if i < notes.len() {
                let note = notes.get_unchecked(i);

                // Removes the note from the owner's set of notes.
                // This will call the the `compute_nullifer` function of the `token_note`
                // which require knowledge of the secret key (currently the users encryption key).
                // The contract logic must ensure that the spending key is used as well.
                // docs:start:remove
                self.map.at(owner).remove(note);
                // docs:end:remove

                minuend = minuend + note.get_amount();
            }
        }

        // This is to provide a nicer error msg,
        // without it minuend-subtrahend would still catch it, but more generic error then.
        // without the == true, it includes 'minuend.ge(subtrahend)' as part of the error.
        assert(minuend >= subtrahend, "Balance too low");

        self.add(owner, minuend - subtrahend)
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
