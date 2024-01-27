mod options;
mod types;

// Following is a very simple game to show case use of singleton in as minimalistic way as possible
// It also serves as an e2e test that you can read and then replace the singleton in the same call
// (tests ordering in the circuit)

// you have a card (singleton). Anyone can create a bigger card. Whoever is bigger will be the leader.
// it also has dummy methods and other examples used for documentation e.g. 
// how to create custom notes, a custom struct for public state, a custom note that may be unencrypted
// also has `options.nr` which shows various ways of using `NoteGetterOptions` to query notes
// it also shows what our macros do behind the scenes!

contract DocsExample {
    // how to import dependencies defined in your workspace
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::AztecAddress,
    };
    use dep::aztec::{
        oracle::{
            debug_log::debug_log_format,
        },
        note::{
            note_header::NoteHeader,
            note_getter_options::{NoteGetterOptions, Comparator},
            note_viewer_options::{NoteViewerOptions},
            utils as note_utils,
        },
        context::{PrivateContext, PublicContext, Context},
        state_vars::{map::Map, public_state::PublicState,singleton::Singleton, immutable_singleton::ImmutableSingleton, set::Set},
    };
    // how to import methods from other files/folders within your workspace
    use crate::options::create_account_card_getter_options;
    use crate::types::{
        card_note::{CardNote, CardNoteMethods, CARD_NOTE_LEN},
        leader::{Leader, LeaderSerializationMethods, LEADER_SERIALIZED_LEN},
    };

    struct Storage {
        // Shows how to create a custom struct in Public
        leader: PublicState<Leader, LEADER_SERIALIZED_LEN>,
        // docs:start:storage-singleton-declaration
        legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
        // docs:end:storage-singleton-declaration
        // just used for docs example to show how to create a singleton map. 
        // docs:start:storage-map-singleton-declaration
        profiles: Map<AztecAddress, Singleton<CardNote, CARD_NOTE_LEN>>,
        // docs:end:storage-map-singleton-declaration
        test: Set<CardNote, CARD_NOTE_LEN>,
        imm_singleton: ImmutableSingleton<CardNote, CARD_NOTE_LEN>,
    }

    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                leader: PublicState::new(
                    context,
                    1,
                    LeaderSerializationMethods,
                ),
                // docs:start:start_vars_singleton
                legendary_card: Singleton::new(context, 2, CardNoteMethods),
                // docs:end:start_vars_singleton
                // just used for docs example (not for game play):
                // docs:start:state_vars-MapSingleton
                profiles: Map::new(
                    context,
                    3,
                    |context, slot| {
                        Singleton::new(context, slot, CardNoteMethods)
                    },
                ),
                // docs:end:state_vars-MapSingleton
                test: Set::new(context, 4, CardNoteMethods),
                imm_singleton: ImmutableSingleton::new(context, 4, CardNoteMethods),
            }
        }
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn initialize_immutable_singleton(randomness: Field, points: u8) {
        let mut new_card = CardNote::new(points, randomness, context.msg_sender());
        storage.imm_singleton.initialize(&mut new_card, true);
    }

    #[aztec(private)]
    // msg_sender() is 0 at deploy time. So created another function
    fn initialize_private(randomness: Field, points: u8) {
        let mut legendary_card = CardNote::new(points, randomness, context.msg_sender());
        // create and broadcast note
        storage.legendary_card.initialize(&mut legendary_card, true);
    }

    #[aztec(private)]
    fn insert_notes(amounts: [u8; 10]) {
        for i in 0..amounts.len() {
            let mut note = CardNote::new(amounts[i], 1, context.msg_sender());
            storage.test.insert(&mut note, true);
        }
    }

    #[aztec(private)]
    fn insert_note(amount: u8, randomness: Field) {
        let mut note = CardNote::new(amount, randomness, context.msg_sender());
        storage.test.insert(&mut note, true);
    }

    unconstrained fn read_note(amount: Field, comparator: u3) -> pub [Option<CardNote>; 10] {
        let options = NoteViewerOptions::new().select(0, amount, Option::some(comparator));
        let notes = storage.test.view_notes(options);

        notes
    }

    #[aztec(private)]
    fn update_legendary_card(randomness: Field, points: u8) {
        let mut new_card = CardNote::new(points, randomness, context.msg_sender());
        storage.legendary_card.replace(&mut new_card, true);

        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("update_leader((Field),u8)"),
            [context.msg_sender().to_field(), points as Field]
        );
    }

    #[aztec(private)]
    fn increase_legendary_points() {
        // Ensure `points` > current value
        // Also serves as a e2e test that you can `get_note()` and then `replace()`

        // docs:start:state_vars-SingletonGet
        let card = storage.legendary_card.get_note(false);
        // docs:end:state_vars-SingletonGet

        let points = card.points + 1;

        let mut new_card = CardNote::new(points, card.randomness, context.msg_sender());
        // docs:start:state_vars-SingletonReplace
        storage.legendary_card.replace(&mut new_card, true);
        // docs:end:state_vars-SingletonReplace

        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("update_leader((Field),u8)"),
            [context.msg_sender().to_field(), points as Field]
        );
    }

    #[aztec(public)]
    internal fn update_leader(account: AztecAddress, points: u8) {
        let new_leader = Leader { account, points };
        storage.leader.write(new_leader);
    }

    unconstrained fn get_leader() -> pub Leader {
        storage.leader.read()
    }

    unconstrained fn get_legendary_card() -> pub CardNote {
        storage.legendary_card.view_note()
    }

    // docs:start:singleton_is_initialized
    unconstrained fn is_legendary_initialized() -> pub bool {
        storage.legendary_card.is_initialized()
    }
    // docs:end:singleton_is_initialized

    unconstrained fn get_imm_card() -> pub CardNote {
        storage.imm_singleton.view_note()
    }

    unconstrained fn is_imm_initialized() -> pub bool {
        storage.imm_singleton.is_initialized()
    }

    // TODO: remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; CARD_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(CardNoteMethods, note_header, serialized_note)
    }

    /// Macro equivalence section
    use dep::aztec::abi;
    use dep::aztec::abi::Hasher;
    use dep::aztec::abi::PrivateContextInputs;
    use dep::aztec::abi::PrivateCircuitPublicInputs;

    // docs:start:simple_macro_example
    #[aztec(private)]
    fn simple_macro_example(a: Field, b: Field) -> Field {
        a + b
    }
    // docs:end:simple_macro_example

    // docs:start:simple_macro_example_expanded
    fn simple_macro_example_expanded(
        // ************************************************************
        // The private context inputs are made available to the circuit by the kernel
        // docs:start:context-example-inputs
        inputs: PrivateContextInputs,
        // docs:end:context-example-inputs
        // ************************************************************

        // Our original inputs!
        a: Field,
        b: Field // The actual return type of our circuit is the PrivateCircuitPublicInputs struct, this will be the 
        // input to our kernel! 
        // docs:start:context-example-return
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // docs:end:context-example-return
        // ************************************************************
        // The hasher is a structure used to generate a hash of the circuits inputs. 
        // docs:start:context-example-hasher
        let mut hasher = Hasher::new();
        hasher.add(a);
        hasher.add(b);
        // docs:end:context-example-hasher

        // The context object is created with the inputs and the hash of the inputs
        // docs:start:context-example-context
        let mut context = PrivateContext::new(inputs, hasher.hash());
        // docs:end:context-example-context

        // docs:start:storage-example-context
        let mut storage = Storage::init(Context::private(&mut context));
        // docs:end:storage-example-context
        // ************************************************************

        // Our actual program
        let result = a + b;

        // ************************************************************
        // Return values are pushed into the context
        // docs:start:context-example-context-return
        context.return_values.push(result);
        // docs:end:context-example-context-return

        // The context is returned to be consumed by the kernel circuit!
        // docs:start:context-example-finish
        context.finish()
        // docs:end:context-example-finish
        // ************************************************************
    }
    // docs:end:simple_macro_example_expanded
}
