use dep::aztec::prelude::{AztecAddress, FunctionSelector, PrivateContext, NoteHeader, NoteGetterOptions, NoteViewerOptions};

use dep::aztec::{
    context::UnconstrainedContext,
    protocol_types::{traits::{ToField, Serialize, FromField}, constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL},
    encrypted_logs::encrypted_note_emission::encode_and_encrypt_note_with_keys,
    note::note_getter::view_notes, state_vars::PrivateSet, note::constants::MAX_NOTES_PER_PAGE
};
use dep::value_note::{value_note::{ValueNote, VALUE_NOTE_LEN}};

struct Card {
    // We use u32s since u16s are unsupported
    strength: u32,
    points: u32,
}

impl FromField for Card {
 fn from_field(field: Field) -> Card {
        let value_bytes = field.to_le_bytes(32);
        let strength = (value_bytes[0] as u32) + (value_bytes[1] as u32) * 256;
        let points = (value_bytes[2] as u32) + (value_bytes[3] as u32) * 256;
        Card { strength, points }
    }
}

impl ToField for Card {
 fn to_field(self) -> Field {
        self.strength as Field + (self.points as Field)*65536
    }
}

impl Serialize<2> for Card {
    fn serialize(self) -> [Field; 2] {
        [self.strength as Field, self.points as Field]
    }
}

// docs:start:pure_noir_testing
#[test]
fn test_to_from_field() {
    let field = 1234567890;
    let card = Card::from_field(field);
    assert(card.to_field() == field);
}
// docs:end:pure_noir_testing

struct CardNote {
    card: Card,
    note: ValueNote,
}

impl CardNote {
    fn new(strength: u32, points: u32, npk_m_hash: Field) -> Self {
        let card = Card { strength, points };
        CardNote::from_card(card, npk_m_hash)
    }

    pub fn from_card(card: Card, npk_m_hash: Field) -> CardNote {
        CardNote { card, note: ValueNote::new(card.to_field(), npk_m_hash) }
    }

    pub fn from_note(note: ValueNote) -> CardNote {
        CardNote { card: Card::from_field(note.value), note }
    }
}

struct Deck<Context> {
    set: PrivateSet<ValueNote, Context>,
}

pub fn filter_cards<N>(
    notes: [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    desired_cards: [Card; N]
) -> [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] {
    let mut selected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];

    let mut found = [false; N];

    for i in 0..notes.len() {
        let note = notes[i];
        if note.is_some() {
            let card_note = CardNote::from_note(note.unwrap_unchecked());
            for j in 0..N {
                if !found[j]
                    & (card_note.card.strength == desired_cards[j].strength)
                    & (card_note.card.points == desired_cards[j].points) {
                    selected[i] = note;
                    found[j] = true;
                }
            }
        }
    }

    selected
}

impl<Context> Deck<Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        let set = PrivateSet { context, storage_slot };
        Deck { set }
    }
}

impl Deck<&mut PrivateContext> {
    pub fn add_cards<N>(&mut self, cards: [Card; N], owner: AztecAddress) -> [CardNote] {
        let header = self.set.context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(self.set.context, owner);
        let msg_sender_ovpk_m = header.get_ovpk_m(self.set.context, self.set.context.msg_sender());
        let owner_ivpk_m = header.get_ivpk_m(self.set.context, owner);

        let mut inserted_cards = &[];
        for card in cards {
            let mut card_note = CardNote::from_card(card, owner_npk_m_hash);
            self.set.insert(&mut card_note.note).emit(encode_and_encrypt_note_with_keys(self.set.context, msg_sender_ovpk_m, owner_ivpk_m));
            inserted_cards = inserted_cards.push_back(card_note);
        }

        inserted_cards
    }

    pub fn get_cards<N>(&mut self, cards: [Card; N]) -> [CardNote; N] {
        let options = NoteGetterOptions::with_filter(filter_cards, cards);
        let notes = self.set.get_notes(options);

        // This array will hold the notes that correspond to each of the requested cards. It begins empty (with all the
        // options being none) and we gradually fill it up as we find the matching notes.
        let mut found_cards = [Option::none(); N];

        for i in 0..options.limit {
            if i < notes.len() {
                let card_note = CardNote::from_note(notes.get_unchecked(i));

                // For each note that we read, we search for a matching card that we have not already marked as found.
                for j in 0..cards.len() {
                    if found_cards[j].is_none()
                        & (cards[j].strength == card_note.card.strength)
                        & (cards[j].points == card_note.card.points) {
                        found_cards[j] = Option::some(card_note);
                    }
                }
            }
        }

        // And then we assert that we did indeed find all cards, since found_cards and cards have the same length.
        found_cards.map(
            |card_note: Option<CardNote>| {
            assert(card_note.is_some(), "Card not found");
            card_note.unwrap_unchecked()
        }
        )
    }

    pub fn remove_cards<N>(&mut self, cards: [Card; N]) {
        let card_notes = self.get_cards(cards);
        for card_note in card_notes {
            self.set.remove(card_note.note);
        }
    }
}

impl Deck<UnconstrainedContext> {
    unconstrained pub fn view_cards(self, offset: u32) -> BoundedVec<Card, MAX_NOTES_PER_PAGE> {
        let mut options = NoteViewerOptions::new();
        let notes = self.set.view_notes(options.set_offset(offset));

        // TODO: ideally we'd do let cards = notes.map(|note| Cards::from_field(note.value));
        // see https://github.com/noir-lang/noir/pull/5250
        let mut cards = BoundedVec::new();
        cards.len = notes.len();
        for i in 0..notes.len() {
            cards.storage[i] = Card::from_field(notes.get_unchecked(i).value);
        }

        cards
    }
}

global PACK_CARDS = 3; // Limited by number of write requests (max 4)

pub fn get_pack_cards(
    seed: Field,
    owner: AztecAddress,
    context: &mut PrivateContext
) -> [Card; PACK_CARDS] {
    let header = context.get_header();
    let owner_npk_m_hash = header.get_npk_m_hash(context, owner);

    // generate pseudo randomness deterministically from 'seed' and user secret
    let secret = context.request_nsk_app(owner_npk_m_hash);
    let mix = secret + seed;
    let mix_bytes: [u8; 32] = mix.to_le_bytes(32).as_array();
    let random_bytes = std::hash::sha256(mix_bytes);

    let mut cards = [Card::from_field(0); PACK_CARDS];
    // we generate PACK_CARDS cards
    assert(PACK_CARDS < 8, "Cannot generate more than 8 cards");
    for i in 0..PACK_CARDS {
        let strength = (random_bytes[i] as u32) + (random_bytes[i + 1] as u32) * 256;
        let points = (random_bytes[i + 2] as u32) + (random_bytes[i + 3] as u32) * 256;
        cards[i] = Card {
            strength, points
        };
    }

    cards
}

pub fn compute_deck_strength<N>(cards: [Card; N]) -> Field {
    cards.fold(
        0,
        |acc, card: Card| {
        acc + card.strength as Field
    }
    )
}
