use dep::aztec::{
    protocol_types::{
        address::AztecAddress,
        constants::{
            MAX_NOTES_PER_PAGE,
            MAX_READ_REQUESTS_PER_CALL,
        },
    },
    context::{PrivateContext, PublicContext, Context},
    note::{
        note_getter_options::NoteGetterOptions,
        note_viewer_options::NoteViewerOptions,
        note_getter::view_notes,
    },
    state_vars::set::Set,
};
use dep::std;
use dep::std::{
    option::Option,
};
use dep::value_note::{
    value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN},
};

struct Card {
    strength: u16,
    points: u16,
}

impl Card {
    pub fn from_field(field: Field) -> Card {
        let value_bytes = field.to_le_bytes(32);
        let strength = (value_bytes[0] as u16) + (value_bytes[1] as u16) * 256;
        let points = (value_bytes[2] as u16) + (value_bytes[3] as u16) * 256;
        Card {
            strength,
            points,
        }
    }

    pub fn to_field(self) -> Field {
        self.strength as Field + (self.points as Field)*65536
    }

    pub fn serialize(self) -> [Field; 2] {
        [self.strength as Field, self.points as Field]
    }
}

#[test]
fn test_to_from_field() {
    let field = 1234567890;
    let card = Card::from_field(field);
    assert(card.to_field() == field);
}

struct CardNote {
    card: Card,
    note: ValueNote,
}

impl CardNote {
    fn new(
        strength: u16,
        points: u16,
        owner: AztecAddress,
    ) -> Self {
        let card = Card {
            strength,
            points,
        };
        CardNote::from_card(card, owner)
    }

    pub fn from_card(card: Card, owner: AztecAddress) -> CardNote {
        CardNote {
            card,
            note: ValueNote::new(card.to_field(), owner),
        }
    }

    pub fn from_note(note: ValueNote) -> CardNote {
        CardNote {
            card: Card::from_field(note.value),
            note,
        }
    }
}

struct Deck {
    set: Set<ValueNote, VALUE_NOTE_LEN>,
}

pub fn filter_cards<N>(
    notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL],
    desired_cards: [Card; N]
) -> [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] {
    let mut selected = [Option::none(); MAX_READ_REQUESTS_PER_CALL];

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

impl Deck {
    pub fn new(
        context: Context,
        storage_slot: Field,
    ) -> Self {
        let set = Set {
            context,
            storage_slot,
            note_interface: ValueNoteMethods,
        };
        Deck {
            set
        }
    }

    pub fn add_cards<N>(&mut self, cards: [Card; N], owner: AztecAddress) -> [CardNote]{
        let context = self.set.context.private.unwrap();

        let mut inserted_cards = [];
        for card in cards {
            let mut card_note = CardNote::from_card(card, owner);
            self.set.insert(&mut card_note.note, true);
            inserted_cards = inserted_cards.push_back(card_note);
        }

        inserted_cards
    }

    pub fn get_cards<N>(&mut self, cards: [Card; N], owner: AztecAddress) -> [CardNote; N] {
        let options = NoteGetterOptions::with_filter(filter_cards, cards);
        let maybe_notes = self.set.get_notes(options);
        let mut found_cards = [Option::none(); N];
        for i in 0..maybe_notes.len() {
            if maybe_notes[i].is_some() {
                let card_note = CardNote::from_note(
                    maybe_notes[i].unwrap_unchecked()
                );
                // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
                // spending someone else's notes).
                assert(card_note.note.owner.eq(owner));

                for j in 0..cards.len() {
                    if found_cards[j].is_none() & (cards[j].strength == card_note.card.strength) & (cards[j].points == card_note.card.points) {
                        found_cards[j] = Option::some(card_note);
                    }
                }
            }
        }

        found_cards.map(|card_note: Option<CardNote>| {
            assert(card_note.is_some(), "Card not found");
            card_note.unwrap_unchecked()
        })
    }

    pub fn remove_cards<N>(&mut self, cards: [Card; N], owner: AztecAddress) {
        let card_notes = self.get_cards(cards, owner);
        for card_note in card_notes {
            self.set.remove(card_note.note);
        }
    }

    unconstrained pub fn view_cards(self, offset: u32) -> [Option<Card>; MAX_NOTES_PER_PAGE] {
        let options = NoteViewerOptions::new().set_offset(offset);
        let opt_notes = self.set.view_notes(options);
        let mut opt_cards = [Option::none(); MAX_NOTES_PER_PAGE];
        
        for i in 0..opt_notes.len() {
            opt_cards[i] = opt_notes[i].map(|note: ValueNote| Card::from_field(note.value));
        }

        opt_cards
    }

}

global PACK_CARDS = 3; // Limited by number of write requests (max 4)

pub fn get_pack_cards(seed: Field, owner: AztecAddress, context: &mut PrivateContext) -> [Card; PACK_CARDS] {
    // generate pseudo randomness deterministically from 'seed' and user secret
    let secret = context.request_nullifier_secret_key(owner);
    let mix = secret.high + secret.low + seed;
    let random_bytes = std::hash::sha256(mix.to_le_bytes(32));

    let mut cards = [Card::from_field(0); PACK_CARDS];
    // we generate PACK_CARDS cards
    assert((PACK_CARDS as u64) < 8, "Cannot generate more than 8 cards");
    for i in 0..PACK_CARDS {
        let strength = (random_bytes[i] as u16) + (random_bytes[i + 1] as u16) * 256;
        let points = (random_bytes[i + 2] as u16) + (random_bytes[i + 3] as u16) * 256;
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
