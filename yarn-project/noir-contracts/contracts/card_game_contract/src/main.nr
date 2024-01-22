mod cards;
mod game;

contract CardGame {
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::AztecAddress,
        constants::MAX_NOTES_PER_PAGE,
    };
    use dep::aztec::{
        context::Context,
        hash::pedersen_hash,
        state_vars::{
            map::Map,
            public_state::PublicState,
        },
    };

    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        value_note::{
            ValueNoteMethods,
            VALUE_NOTE_LEN,
        },
    };

    use dep::aztec::{
        abi,
        abi::{
            Hasher, PrivateContextInputs,
        },
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
    };

    use crate::cards::{
        PACK_CARDS,
        Deck, 
        Card,
        get_pack_cards,
        compute_deck_strength,
    };
    use crate::game::{
        NUMBER_OF_PLAYERS,
        NUMBER_OF_CARDS_DECK,
        PLAYABLE_CARDS,
        PlayerEntry,
        Game,
        GameSerializationMethods,
        GAME_SERIALIZED_LEN
    };

    struct Storage {
        collections: Map<AztecAddress, Deck>,
        game_decks: Map<Field, Map<AztecAddress, Deck>>,
        games: Map<Field, PublicState<Game, GAME_SERIALIZED_LEN>>,
    }

    impl Storage {
        fn init(
            context: Context,
        ) -> Self {
            Storage {
                collections: Map::new(
                    context,
                    1,
                    |context, slot| {
                        Deck::new(
                            context,
                            slot,
                        )
                    },
                ),
                game_decks: Map::new(
                    context,
                    2,
                    |context, slot| {
                        Map::new(
                            context,
                            slot,
                            |context, slot|{
                                Deck::new(
                                    context,
                                    slot,
                                )
                            }
                        )
                    },
                ),
                games: Map::new(
                    context,
                    3,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            GameSerializationMethods,
                        )
                    },
                )
            }
        }
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn buy_pack(seed: Field // The randomness used to generate the cards. Passed in for now.
    ) {
        let buyer = context.msg_sender();
        let mut cards = get_pack_cards(seed, buyer, &mut context);

        let mut collection = storage.collections.at(buyer);
        let _inserted_cards = collection.add_cards(cards, buyer);
    }

    #[aztec(private)]
    fn join_game(game: u32, cards_fields: [Field; 2]) {
        let cards = cards_fields.map(|card_field| Card::from_field(card_field));

        let player = context.msg_sender();

        let mut collection = storage.collections.at(player);
        collection.remove_cards(cards, player);
        let mut game_deck = storage.game_decks.at(game as Field).at(player);
        let _added_to_game_deck = game_deck.add_cards(cards, player);
        let selector = FunctionSelector::from_signature("on_game_joined(u32,(Field),u32)");
        let strength = compute_deck_strength(cards);
        context.call_public_function(
            context.this_address(),
            selector,
            [game as Field, player.to_field(), strength]
        );
    }

    #[aztec(public)]
    internal fn on_game_joined(game: u32, player: AztecAddress, deck_strength: u32) {
        let game_storage = storage.games.at(game as Field);

        let mut game_data = game_storage.read();
        assert(game_data.add_player(PlayerEntry { address: player, deck_strength, points: 0 }), "Game full");

        game_storage.write(game_data);
    }

    #[aztec(public)]
    fn start_game(game: u32) {
        let game_storage = storage.games.at(game as Field);

        let mut game_data = game_storage.read();
        game_data.start_game();
        game_storage.write(game_data);
    }

    #[aztec(private)]
    fn play_card(game: u32, card: Card) {
        let player = context.msg_sender();

        let mut game_deck = storage.game_decks.at(game as Field).at(player);
        game_deck.remove_cards([card], player);

        let selector = FunctionSelector::from_signature("on_card_played(u32,(Field),Field)");
        // docs:start:call_public_function
        context.call_public_function(
            context.this_address(),
            selector,
            [game as Field, player.to_field(), card.to_field()]
        );
        // docs:end:call_public_function
    }

    #[aztec(public)]
    internal fn on_card_played(game: u32, player: AztecAddress, card_as_field: Field) {
        let game_storage = storage.games.at(game as Field);

        let mut game_data = game_storage.read();

        let card = Card::from_field(card_as_field);
        let current_player = game_data.current_player();
        assert(current_player.address.eq(player), "Not your turn");
        game_data.play_card(card);

        game_storage.write(game_data);
    }

    #[aztec(private)]
    fn claim_cards(game: u32, cards_fields: [Field; PLAYABLE_CARDS]) {
        let player = context.msg_sender();
        let cards = cards_fields.map(|card_field| Card::from_field(card_field));

        let mut collection = storage.collections.at(player);
        let _inserted_cards = collection.add_cards(cards, player);

        let selector = FunctionSelector::from_signature("on_cards_claimed(u32,(Field),Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [game as Field, player.to_field(), pedersen_hash(cards_fields, 0)]
        );
    }

    #[aztec(public)]
    internal fn on_cards_claimed(game: u32, player: AztecAddress, cards_hash: Field) {
        let game_storage = storage.games.at(game as Field);
        let mut game_data = game_storage.read();

        assert(!game_data.claimed, "Already claimed");
        game_data.claimed = true;

        assert_eq(cards_hash, pedersen_hash(game_data.rounds_cards.map(|card: Card| card.to_field()), 0));

        let winner = game_data.winner();
        assert(player.eq(winner.address), "Not the winner");

        game_storage.write(game_data);
    }

    unconstrained fn view_collection_cards(owner: AztecAddress, offset: u32) -> pub [Option<Card>; MAX_NOTES_PER_PAGE] {
        let collection = storage.collections.at(owner);

        collection.view_cards(offset)
    }

    unconstrained fn view_game_cards(game: u32, player: AztecAddress, offset: u32) -> pub [Option<Card>; MAX_NOTES_PER_PAGE] {
        let game_deck = storage.game_decks.at(game as Field).at(player);

        game_deck.view_cards(offset)
    }

    unconstrained fn view_game(game: u32) -> pub Game {
        storage.games.at(game as Field).read()
    }

    // Computes note hash and nullifier.
    // Note 1: Needs to be defined by every contract producing logs.
    // Note 2: Having it in all the contracts gives us the ability to compute the note hash and nullifier differently for different kind of notes.
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNoteMethods, note_header, serialized_note)
    }
}
