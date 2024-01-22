use dep::aztec::state_vars::{
    singleton::Singleton,
};
use dep::std::option::Option;

use crate::types::{
    card_note::{CardNote, CARD_NOTE_LEN},
};

pub fn init_legendary_card(state_var: Singleton<CardNote, CARD_NOTE_LEN>, card: &mut CardNote) {
    state_var.initialize(card, Option::some(card.owner), true);
}

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
