use crate::types::card_note::{CardNote, CARD_NOTE_LEN, CARD_NOTE_BYTES_LEN};
use dep::aztec::prelude::{AztecAddress, NoteGetterOptions};

use dep::aztec::protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
use dep::aztec::note::note_getter_options::{Sort, SortOrder};

// Shows how to use NoteGetterOptions and query for notes.

// docs:start:state_vars-NoteGetterOptionsSelectSortOffset
pub fn create_points_card_getter_options(
    points: Field,
    offset: u32
) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, CARD_NOTE_BYTES_LEN, Field> {
    let mut options = NoteGetterOptions::new();
    options.select(CardNote::properties().points, points, Option::none()).sort(CardNote::properties().points, SortOrder.DESC).set_offset(offset)
}
// docs:end:state_vars-NoteGetterOptionsSelectSortOffset

// docs:start:state_vars-NoteGetterOptionsMultiSelects
pub fn create_exact_card_getter_options(
    points: u8,
    secret: Field,
    account_npk_m_hash: Field
) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, CARD_NOTE_BYTES_LEN, Field> {
    let mut options = NoteGetterOptions::new();
    options.select(CardNote::properties().points, points as Field, Option::none()).select(CardNote::properties().randomness, secret, Option::none()).select(
        CardNote::properties().npk_m_hash,
        account_npk_m_hash,
        Option::none()
    )
}
// docs:end:state_vars-NoteGetterOptionsMultiSelects

// docs:start:state_vars-OptionFilter
pub fn filter_min_points(
    cards: [Option<CardNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    min_points: u8
) -> [Option<CardNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] {
    let mut selected_cards = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    let mut num_selected = 0;
    for i in 0..cards.len() {
        if cards[i].is_some() & cards[i].unwrap_unchecked().points >= min_points {
            selected_cards[num_selected] = cards[i];
            num_selected += 1;
        }
    }
    selected_cards
}
// docs:end:state_vars-OptionFilter

// docs:start:state_vars-NoteGetterOptionsFilter
pub fn create_cards_with_min_points_getter_options(min_points: u8) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, CARD_NOTE_BYTES_LEN, u8> {
    NoteGetterOptions::with_filter(filter_min_points, min_points).sort(CardNote::properties().points, SortOrder.ASC)
}
// docs:end:state_vars-NoteGetterOptionsFilter

// docs:start:state_vars-NoteGetterOptionsPickOne
pub fn create_largest_card_getter_options() -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, CARD_NOTE_BYTES_LEN, Field> {
    let mut options = NoteGetterOptions::new();
    options.sort(CardNote::properties().points, SortOrder.DESC).set_limit(1)
}
// docs:end:state_vars-NoteGetterOptionsPickOne
