use dep::std::option::Option;
use dep::aztec::constants_gen::{MAX_READ_REQUESTS_PER_CALL, MAX_NOTES_PER_PAGE};
use dep::aztec::context::{
    PrivateContext,
};
use dep::aztec::note::{
    note_getter_options::NoteGetterOptions,
    note_viewer_options::NoteViewerOptions,
};
use dep::aztec::state_vars::{
    immutable_singleton::ImmutableSingleton,
    map::Map,
    public_state::PublicState,
    set::Set,
    singleton::Singleton,
};
use dep::aztec::types::type_serialisation::bool_serialisation::{
    BOOL_SERIALISED_LEN,
};

use crate::types::{
    card_note::{CardNote, CARD_NOTE_LEN},
    profile_note::{ProfileNote, PROFILE_NOTE_LEN},
    queen::{Queen, QUEEN_SERIALISED_LEN},
    rules_note::{RulesNote, RULES_NOTE_LEN},
};

// docs:start:state_vars-PublicStateRead
fn is_locked(state_var: PublicState<bool, BOOL_SERIALISED_LEN>) -> bool {
    state_var.read()
}
// docs:end:state_vars-PublicStateRead

// docs:start:state_vars-PublicStateWrite
fn lock(state_var: PublicState<bool, BOOL_SERIALISED_LEN>) {
    state_var.write(true);
}
// docs:end:state_vars-PublicStateWrite

fn unlock(state_var: PublicState<bool, BOOL_SERIALISED_LEN>) {
    state_var.write(false);
}

// docs:start:state_vars-PublicStateReadCustom
fn get_current_queen(state_var: PublicState<Queen, QUEEN_SERIALISED_LEN>) -> Queen {
    state_var.read()
}
// docs:end:state_vars-PublicStateReadCustom

fn can_replace_queen(
    state_var: PublicState<Queen, QUEEN_SERIALISED_LEN>,
    new_queen: Queen,
  ) -> bool {
    let current_queen = get_current_queen(state_var);
    new_queen.points > current_queen.points
}

// docs:start:state_vars-PublicStateWriteCustom
fn replace_queen(
    state_var: PublicState<Queen, QUEEN_SERIALISED_LEN>,
    new_queen: Queen,
) {
    state_var.write(new_queen);
}
// docs:end:state_vars-PublicStateWriteCustom

// docs:start:state_vars-PublicStateReadWriteCustom
fn add_points_to_queen(
    state_var: PublicState<Queen, QUEEN_SERIALISED_LEN>,
    new_points: u8,
) {
    let mut queen = state_var.read();
    queen.points += new_points;
    state_var.write(queen);
}
// docs:end:state_vars-PublicStateReadWriteCustom

// docs:start:state_vars-SingletonInit
fn init_legendary_card(
    context: &mut PrivateContext,
    state_var: Singleton<CardNote, CARD_NOTE_LEN>,
    card: &mut CardNote,
) {
    state_var.initialise(context, card);
}
// docs:end:state_vars-SingletonInit

// docs:start:state_vars-SingletonReplace
fn update_legendary_card(
    context: &mut PrivateContext,
    state_var: Singleton<CardNote, CARD_NOTE_LEN>,
    card: &mut CardNote,
) {
    state_var.replace(context, card);
}
// docs:end:state_vars-SingletonReplace

// docs:start:state_vars-SingletonGet
fn get_legendary_card(
    context: &mut PrivateContext,
    state_var: Singleton<CardNote, CARD_NOTE_LEN>,
) -> CardNote {
    state_var.get_note(context)
}
// docs:end:state_vars-SingletonGet

// docs:start:state_vars-ImmutableSingletonInit
fn init_game_rules(
    context: &mut PrivateContext,
    state_var: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>,
    rules: &mut RulesNote,
) {
    state_var.initialise(context, rules);
}
// docs:end:state_vars-ImmutableSingletonInit

// docs:start:state_vars-ImmutableSingletonGet
fn is_valid_card(
    context: &mut PrivateContext,
    state_var: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>,
    card: CardNote,
) -> bool {
    let rules = state_var.get_note(context);
    card.points >= rules.min_points & card.points <= rules.max_points
}
// docs:end:state_vars-ImmutableSingletonGet

// docs:start:state_vars-SetInsert
fn add_new_card(
    context: &mut PrivateContext,
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    card: &mut CardNote,
  ) {
    state_var.insert(context, card);
}
// docs:end:state_vars-SetInsert

// docs:start:state_vars-SetRemove
fn remove_card(
    context: &mut PrivateContext,
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    card: CardNote,
) {
    state_var.remove(context, card);
}
// docs:end:state_vars-SetRemove

// docs:start:state_vars-SetGet
fn get_cards<FILTER_PARAMS>(
    context: &mut PrivateContext,
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    options: NoteGetterOptions<CardNote, CARD_NOTE_LEN, FILTER_PARAMS>,
) -> [Option<CardNote>; MAX_READ_REQUESTS_PER_CALL] {
    state_var.get_notes(context, options)
}
// docs:end:state_vars-SetGet

// docs:start:state_vars-SetView
unconstrained fn view_cards(
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    options: NoteViewerOptions<CardNote, CARD_NOTE_LEN>,
) -> [Option<CardNote>; MAX_NOTES_PER_PAGE] {
    state_var.view_notes(options)
}
// docs:end:state_vars-SetView

unconstrained fn get_total_points(state_var: Set<CardNote, CARD_NOTE_LEN>, account: Field, offset: u32) -> u8 {
    let options = NoteViewerOptions::new().select(2, account).set_offset(offset);
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

// docs:start:state_vars-SetContains
fn assert_contains_card<RERUEN_LEN, FILTER_PARAMS>(
    context: &mut PrivateContext,
    state_var: Set<CardNote, CARD_NOTE_LEN>,
    card: CardNote,
) {
    state_var.assert_contains_and_remove(context, card);
}
// docs:end:state_vars-SetContains

// docs:start:state_vars-MapAtSingletonInit
fn add_new_profile(
    context: &mut PrivateContext,
    state_var: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>,
    account: Field,
    profile: &mut ProfileNote,
) {
    state_var.at(account).initialise(context, profile);
}
// docs:end:state_vars-MapAtSingletonInit

// docs:start:state_vars-MapAtSingletonGet
fn get_profile(
    context: &mut PrivateContext,
    state_var: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>,
    account: Field,
) -> ProfileNote {
    state_var.at(account).get_note(context)
}
// docs:end:state_vars-MapAtSingletonGet