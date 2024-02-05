use dep::std::option::Option;
use dep::safe_math::SafeU120;
use dep::aztec::{
    context::{PrivateContext, PublicContext, Context},
    hash::pedersen_hash,
    protocol_types::{
        address::AztecAddress,
        constants::MAX_READ_REQUESTS_PER_CALL,
    },
    state_vars::{
        set::Set,
        map::Map
    },
    note::{
        note_getter::view_notes,
        note_getter_options::{NoteGetterOptions, SortOrder},
        note_viewer_options::NoteViewerOptions,
        note_header::NoteHeader,
        note_interface::NoteInterface,
    }
};
use crate::types::token_note::{TokenNote, OwnedNote};

struct BalancesMap<T> {
    map: Map<AztecAddress, Set<T>>
}

impl<T> BalancesMap<T> {
    pub fn new(
        context: Context,
        storage_slot: Field,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self {
            map: Map::new(context, storage_slot, |context, slot| Set::new(context, slot))
        }
    }

    unconstrained pub fn balance_of<T_SERIALIZED_LEN>(self: Self, owner: AztecAddress) -> SafeU120 where T: NoteInterface<T_SERIALIZED_LEN> + OwnedNote {
        self.balance_of_with_offset(owner, 0)
    }

  unconstrained pub fn balance_of_with_offset<T_SERIALIZED_LEN>(self: Self, owner: AztecAddress, offset: u32) -> SafeU120 where T: NoteInterface<T_SERIALIZED_LEN> + OwnedNote {
      // Same as SafeU120::new(0), but fewer constraints because no check. 
      let mut balance = SafeU120::min();
      // docs:start:view_notes
      let options = NoteViewerOptions::new().set_offset(offset);
      let opt_notes = self.map.at(owner).view_notes(options);
      // docs:end:view_notes
      let len = opt_notes.len();
      for i in 0..len {
          if opt_notes[i].is_some() {
              balance = balance.add(opt_notes[i].unwrap_unchecked().get_amount());
          }
      }
      if (opt_notes[len - 1].is_some()) {
          balance = balance.add(self.balance_of_with_offset(owner, offset + opt_notes.len() as u32));
      }

      balance
  }

    pub fn add<T_SERIALIZED_LEN>(self: Self, owner: AztecAddress, addend: SafeU120) where T: NoteInterface<T_SERIALIZED_LEN> + OwnedNote {
        let mut addend_note = T::new(addend, owner);

        // docs:start:insert
        self.map.at(owner).insert(&mut addend_note, true);
        // docs:end:insert
    }

    pub fn sub<T_SERIALIZED_LEN>(self: Self, owner: AztecAddress, subtrahend: SafeU120) where T: NoteInterface<T_SERIALIZED_LEN> + OwnedNote{
        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend);
        let maybe_notes = self.map.at(owner).get_notes(options);
        // docs:end:get_notes

        let mut minuend: SafeU120 = SafeU120::min();
        for i in 0..maybe_notes.len() {
            if maybe_notes[i].is_some() {
                let note = maybe_notes[i].unwrap_unchecked();

                // Removes the note from the owner's set of notes.
                // This will call the the `compute_nullifer` function of the `token_note`
                // which require knowledge of the secret key (currently the users encryption key).
                // The contract logic must ensure that the spending key is used as well.
                // docs:start:remove
                self.map.at(owner).remove(note);
                // docs:end:remove

                minuend = minuend.add(note.get_amount());
            }
        }

        // This is to provide a nicer error msg,
        // without it minuend-subtrahend would still catch it, but more generic error then.
        // without the == true, it includes 'minuend.ge(subtrahend)' as part of the error.
        assert(minuend.ge(subtrahend) == true, "Balance too low");

        self.add(owner, minuend.sub(subtrahend));
    }

}

pub fn filter_notes_min_sum<T, T_SERIALIZED_LEN>(
    notes: [Option<T>; MAX_READ_REQUESTS_PER_CALL],
    min_sum: SafeU120
) -> [Option<T>; MAX_READ_REQUESTS_PER_CALL] where T: NoteInterface<T_SERIALIZED_LEN> + OwnedNote {
    let mut selected = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
    let mut sum = SafeU120::min();
    for i in 0..notes.len() {
        if notes[i].is_some() & sum.lt(min_sum) {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum = sum.add(note.get_amount());
        }
    }
    selected
}
