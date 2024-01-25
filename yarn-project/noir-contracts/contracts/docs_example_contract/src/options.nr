use crate::types::card_note::{CardNote, CARD_NOTE_LEN};
use dep::aztec::protocol_types::{
    address::AztecAddress,
    constants::MAX_READ_REQUESTS_PER_CALL,
};
use dep::aztec::note::note_getter_options::{NoteGetterOptions, Sort, SortOrder};
use dep::std::option::Option;

// Shows how to use NoteGetterOptions and query for notes.

// docs:start:state_vars-NoteGetterOptionsSelectSortOffset
pub fn create_account_card_getter_options(
    account: AztecAddress,
    offset: u32
) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, Field> {
    NoteGetterOptions::new().select(2, account.to_field(), Option::none()).sort(0, SortOrder.DESC).set_offset(offset)
}
// docs:end:state_vars-NoteGetterOptionsSelectSortOffset

// docs:start:state_vars-NoteGetterOptionsMultiSelects
pub fn create_exact_card_getter_options(
    points: u8,
    secret: Field,
    account: AztecAddress
) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, Field> {
    NoteGetterOptions::new().select(0, points as Field, Option::none()).select(1, secret, Option::none()).select(2, account.to_field(), Option::none())
}
// docs:end:state_vars-NoteGetterOptionsMultiSelects

// docs:start:state_vars-OptionFilter
pub fn filter_min_points(
    cards: [Option<CardNote>; MAX_READ_REQUESTS_PER_CALL],
    min_points: u8
) -> [Option<CardNote>; MAX_READ_REQUESTS_PER_CALL] {
    let mut selected_cards = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
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
pub fn create_account_cards_with_min_points_getter_options(account: AztecAddress, min_points: u8) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, u8> {
    NoteGetterOptions::with_filter(filter_min_points, min_points).select(2, account.to_field(), Option::none()).sort(0, SortOrder.ASC)
}
// docs:end:state_vars-NoteGetterOptionsFilter

// docs:start:state_vars-NoteGetterOptionsPickOne
pub fn create_largest_account_card_getter_options(account: AztecAddress) -> NoteGetterOptions<CardNote, CARD_NOTE_LEN, Field> {
    NoteGetterOptions::new().select(2, account.to_field(), Option::none()).sort(0, SortOrder.DESC).set_limit(1)
}
// docs:end:state_vars-NoteGetterOptionsPickOne
