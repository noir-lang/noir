use dep::protocol_types::{
    address::AztecAddress,
    constants::{MAX_NOTES_PER_PAGE, MAX_READ_REQUESTS_PER_CALL},
};
use dep::aztec::note::{
    note_getter_options::NoteGetterOptions, note_viewer_options::NoteViewerOptions,
};
use dep::aztec::state_vars::{
    immutable_singleton::ImmutableSingleton, map::Map, public_state::PublicState, set::Set,
    singleton::Singleton,
};
use dep::aztec::types::type_serialization::bool_serialization::BOOL_SERIALIZED_LEN;
use dep::std::option::Option;

use crate::types::{
    card_note::{CardNote, CARD_NOTE_LEN},
    profile_note::{ProfileNote, PROFILE_NOTE_LEN},
    queen::{Queen, QUEEN_SERIALIZED_LEN},
    rules_note::{RulesNote, RULES_NOTE_LEN},
};

// docs:start:state_vars-PublicStateRead
pub fn is_locked(state_var: PublicState<bool, BOOL_SERIALIZED_LEN>) -> bool {
    state_var.read()
}
// docs:end:state_vars-PublicStateRead

// docs:start:state_vars-PublicStateWrite
pub fn lock(state_var: PublicState<bool, BOOL_SERIALIZED_LEN>) {
    state_var.write(true);
}
// docs:end:state_vars-PublicStateWrite

pub fn unlock(state_var: PublicState<bool, BOOL_SERIALIZED_LEN>) {
    state_var.write(false);
}

// docs:start:state_vars-PublicStateReadCustom
pub fn get_current_queen(state_var: PublicState<Queen, QUEEN_SERIALIZED_LEN>) -> Queen {
    state_var.read()
}
// docs:end:state_vars-PublicStateReadCustom

pub fn can_replace_queen(state_var: PublicState<Queen, QUEEN_SERIALIZED_LEN>, new_queen: Queen) -> bool {
    let current_queen = get_current_queen(state_var);
    new_queen.points > current_queen.points
}

// docs:start:state_vars-PublicStateWriteCustom
pub fn replace_queen(state_var: PublicState<Queen, QUEEN_SERIALIZED_LEN>, new_queen: Queen) {
    state_var.write(new_queen);
}
// docs:end:state_vars-PublicStateWriteCustom

// docs:start:state_vars-PublicStateReadWriteCustom
pub fn add_points_to_queen(state_var: PublicState<Queen, QUEEN_SERIALIZED_LEN>, new_points: u8) {
    let mut queen = state_var.read();
    queen.points += new_points;
    state_var.write(queen);
}
// docs:end:state_vars-PublicStateReadWriteCustom

// docs:start:state_vars-SingletonInit
pub fn init_legendary_card(state_var: Singleton<CardNote, CARD_NOTE_LEN>, card: &mut CardNote) {
    state_var.initialize(card, Option::some(card.owner), true);
}
// docs:end:state_vars-SingletonInit

// docs:start:state_vars-SingletonReplace
pub fn update_legendary_card(state_var: Singleton<CardNote, CARD_NOTE_LEN>, card: &mut CardNote) {
    state_var.replace(card, true);
}
// docs:end:state_vars-SingletonReplace

// docs:start:state_vars-SingletonGet
pub fn get_legendary_card(state_var: Singleton<CardNote, CARD_NOTE_LEN>) -> CardNote {
    state_var.get_note(true)
}
// docs:end:state_vars-SingletonGet

// docs:start:state_vars-ImmutableSingletonInit
pub fn init_game_rules(state_var: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>, rules: &mut RulesNote) {
    state_var.initialize(rules, Option::none(), true);
}
// docs:end:state_vars-ImmutableSingletonInit

// docs:start:state_vars-ImmutableSingletonGet
pub fn is_valid_card(state_var: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>, card: CardNote) -> bool {
    let rules = state_var.get_note();
    card.points >= rules.min_points & card.points <= rules.max_points
}
// docs:end:state_vars-ImmutableSingletonGet

// docs:start:state_vars-SetInsert
pub fn add_new_card(state_var: Set<CardNote, CARD_NOTE_LEN>, card: &mut CardNote) {
    state_var.insert(card, true);
}
// docs:end:state_vars-SetInsert

// docs:start:state_vars-SetRemove
pub fn remove_card(state_var: Set<CardNote, CARD_NOTE_LEN>, card: CardNote) {
    state_var.remove(card);
}
// docs:end:state_vars-SetRemove

// docs:start:state_vars-SetGet
pub fn get_cards<FILTER_PARAMS>(
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    options: NoteGetterOptions<CardNote, CARD_NOTE_LEN, FILTER_PARAMS>
) -> [Option<CardNote>; MAX_READ_REQUESTS_PER_CALL] {
    state_var.get_notes(options)
}
// docs:end:state_vars-SetGet

// docs:start:state_vars-SetView
unconstrained pub fn view_cards(state_var: Set<CardNote, CARD_NOTE_LEN>, options: NoteViewerOptions<CardNote, CARD_NOTE_LEN>) -> [Option<CardNote>; MAX_NOTES_PER_PAGE] {
    state_var.view_notes(options)
}
// docs:end:state_vars-SetView

unconstrained pub fn get_total_points(state_var: Set<CardNote, CARD_NOTE_LEN>, account: AztecAddress, offset: u32) -> u8 {
    let options = NoteViewerOptions::new().select(2, account.to_field()).set_offset(offset);
    let mut total_points = 0;
    let notes = view_cards(state_var, options);
    for i in 0..notes.len() {
        if notes[i].is_some() {
            total_points += notes[i].unwrap_unchecked().points;
        }
    }
    if notes[notes.len() - 1].is_some() {
        total_points += get_total_points(state_var, account, offset + notes.len() as u32);
    }
    total_points
}

// docs:start:state_vars-MapAtSingletonInit
pub fn add_new_profile(state_var: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>, account: AztecAddress, profile: &mut ProfileNote) {
    state_var.at(account.to_field()).initialize(profile, Option::some(account), true);
}
// docs:end:state_vars-MapAtSingletonInit

// docs:start:state_vars-MapAtSingletonGet
pub fn get_profile(state_var: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>, account: AztecAddress) -> ProfileNote {
    state_var.at(account.to_field()).get_note(true)
}
// docs:end:state_vars-MapAtSingletonGet
