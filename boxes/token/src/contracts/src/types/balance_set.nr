use dep::std::option::Option;
use dep::safe_math::SafeU120;
use dep::aztec::{
  context::Context,
  state_vars::set::Set,
};
use dep::aztec::protocol_types::{
    constants::MAX_READ_REQUESTS_PER_CALL,
    address::AztecAddress,
};
use dep::aztec::note::{
    note_getter::view_notes,
    note_getter_options::{NoteGetterOptions, SortOrder},
    note_viewer_options::NoteViewerOptions
};
use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
};
use crate::types::token_note::{TokenNote, TOKEN_NOTE_LEN, TokenNoteMethods};

// A set implementing standard manipulation of balances.
// Does not require spending key, but only knowledge.
// Spending key requirement should be enforced by the contract using this.
struct BalanceSet {
    owner: AztecAddress,
    set: Set<TokenNote, TOKEN_NOTE_LEN>
}

impl BalanceSet {
  pub fn new(set: Set<TokenNote, TOKEN_NOTE_LEN>, owner: AztecAddress) -> Self {
    Self {
        owner,
        set,
    }
  }

  unconstrained pub fn balance_of(self: Self) -> SafeU120 {
    self.balance_of_with_offset(0)
  }

  unconstrained pub fn balance_of_with_offset(self: Self, offset: u32) -> SafeU120 {
      // Same as SafeU120::new(0), but fewer constraints because no check. 
      let mut balance = SafeU120::min();
      // docs:start:view_notes
      let options = NoteViewerOptions::new().set_offset(offset);
      let opt_notes = self.set.view_notes(options);
      // docs:end:view_notes
      let len = opt_notes.len();
      for i in 0..len {
          if opt_notes[i].is_some() {
              balance = balance.add(opt_notes[i].unwrap_unchecked().amount);
          }
      }
      if (opt_notes[len - 1].is_some()) {
          balance = balance.add(self.balance_of_with_offset(offset + opt_notes.len() as u32));
      }

      balance
  }

    pub fn add(self: Self, addend: SafeU120) {
        let mut addend_note = TokenNote::new(addend, self.owner);

        // docs:start:insert
        self.set.insert(&mut addend_note, true);
        // docs:end:insert
    }

    pub fn sub(self: Self, subtrahend: SafeU120) {
        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend);
        let maybe_notes = self.set.get_notes(options);
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
                self.set.remove(note);
                // docs:end:remove

                minuend = minuend.add(note.amount);
            }
        }

        // This is to provide a nicer error msg,
        // without it minuend-subtrahend would still catch it, but more generic error then.
        // without the == true, it includes 'minuend.ge(subtrahend)' as part of the error.
        assert(minuend.ge(subtrahend) == true, "Balance too low");

        self.add(minuend.sub(subtrahend));
    }
}

pub fn filter_notes_min_sum(
    notes: [Option<TokenNote>; MAX_READ_REQUESTS_PER_CALL],
    min_sum: SafeU120
) -> [Option<TokenNote>; MAX_READ_REQUESTS_PER_CALL] {
    let mut selected = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
    let mut sum = SafeU120::min();
    for i in 0..notes.len() {
        if notes[i].is_some() & sum.lt(min_sum) {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum = sum.add(note.amount);
        }
    }
    selected
}
