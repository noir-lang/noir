mod account_contract_interface;
mod actions;
mod options;
mod types;

contract DocsExample {
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        state_vars::{
            immutable_singleton::ImmutableSingleton, map::Map, public_state::PublicState, set::Set,
            singleton::Singleton,
        },
    };
    // docs:start:state_vars-PublicStateBoolImport
    use dep::aztec::types::type_serialisation::bool_serialisation::{
        BoolSerialisationMethods, BOOL_SERIALISED_LEN,
    };
    // docs:end:state_vars-PublicStateBoolImport
    use crate::account_contract_interface::AccountContractInterface;
    use crate::actions;
    use crate::options::{create_account_card_getter_options, create_largest_account_card_getter_options};
    use crate::types::{
        card_note::{CardNote, CardNoteMethods, CARD_NOTE_LEN},
        profile_note::{ProfileNote, ProfileNoteMethods, PROFILE_NOTE_LEN},
        queen::{Queen, QueenSerialisationMethods, QUEEN_SERIALISED_LEN},
        rules_note::{RulesNote, RulesNoteMethods, RULES_NOTE_LEN},
    };

    // docs:start:storage-declaration
    struct Storage {
        locked: PublicState<bool, BOOL_SERIALISED_LEN>,
        queen: PublicState<Queen, QUEEN_SERIALISED_LEN>,
        game_rules: ImmutableSingleton<RulesNote, RULES_NOTE_LEN>,
        legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
        cards: Set<CardNote, CARD_NOTE_LEN>,
        profiles: Map<Singleton<ProfileNote, PROFILE_NOTE_LEN>>,
    }

    // docs:start:state_vars-PublicState
    // docs:start:state_vars-PublicStateCustomStruct
    // docs:start:state_vars-Singleton
    // docs:start:state_vars-ImmutableSingleton
    // docs:start:state_vars-Set
    // docs:start:state_vars-MapSingleton
    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                // highlight-next-line:state_vars-PublicState
                locked: PublicState::new(context, 1, BoolSerialisationMethods),
                // highlight-next-line:state_vars-PublicStateCustomStruct
                queen: PublicState::new(
                    context,
                    2,
                    QueenSerialisationMethods,
                ),
                // highlight-next-line:state_vars-ImmutableSingleton
                game_rules: ImmutableSingleton::new(context, 3, RulesNoteMethods),
                // highlight-next-line:state_vars-Singleton
                legendary_card: Singleton::new(context, 4, CardNoteMethods),
                // highlight-next-line:state_vars-Set
                cards: Set::new(context, 5, CardNoteMethods),
                // highlight-next-line:state_vars-MapSingleton
                profiles: Map::new(
                    context,
                    6,
                    |context, slot| {
                        Singleton::new(context, slot, ProfileNoteMethods)
                    },
                ),
            }
        }
    }
    // docs:end:state_vars-PublicState
    // docs:end:state_vars-PublicStateCustomStruct
    // docs:end:state_vars-Singleton
    // docs:end:state_vars-ImmutableSingleton
    // docs:end:state_vars-Set
    // docs:end:state_vars-MapSingleton
    // docs:end:storage-declaration

    global REPLACE_QUEEN_FUNCTION_SELECTOR = 11111111;
    global GET_POINTS_OF_COMMON_CARD_FUNCTION_SELECTOR = 11111111;

    #[aztec(private)]
    fn constructor(
        min_points: u8,
        max_points: u8,
        legendary_card_secret: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let mut game_rules = RulesNote::new(min_points, max_points);
        actions::init_game_rules(storage.game_rules, &mut game_rules);

        let mut legendary_card = CardNote::new(0, legendary_card_secret, 0);
        actions::init_legendary_card(storage.legendary_card, &mut legendary_card);
    }

    // docs:start:storage-init
    #[aztec(public)]
    fn lock() {
        // highlight-next-line:storage-init
        let storage = Storage::init(Context::public(&mut context));
        storage.locked.write(true);
    }
    // docs:end:storage-init

    // docs:start:functions-OpenFunction
    #[aztec(public)]
    fn unlock() {
        let storage = Storage::init(Context::public(&mut context));
        actions::unlock(storage.locked);
    }
    // docs:end:functions-OpenFunction

    #[aztec(public)]
    fn replace_queen(
        account: Field,
        points: u8,
    ) {
        let storage = Storage::init(Context::public(&mut context));

        let new_queen = Queen { account, points };
        
        assert(actions::can_replace_queen(storage.queen, new_queen));
        
        actions::replace_queen(storage.queen, new_queen);
    }

    // docs:start:state_vars-PublicStateWriteBeforeCall
    #[aztec(public)]
    fn replace_queen_unsafe() {
        let storage = Storage::init(Context::public(&mut context));

        let account = context.msg_sender();
        let points = actions::get_total_points(storage.cards, account, 0);

        let current_queen = storage.queen.read();
        assert(account != current_queen.account);
        assert(points > current_queen.points);

        AccountContractInterface::at(account).send_rewards(current_queen.points);
        
        let new_queen = Queen { account, points };
        storage.queen.write(new_queen);
    }
    // docs:end:state_vars-PublicStateWriteBeforeCall

    // docs:start:functions-SecretFunction
    #[aztec(private)]
    fn add_common_cards(secrets: [Field; 4]) {
        let storage = Storage::init(Context::private(&mut context));

        for i in 0..secrets.len() as u8 {
            let mut card = CardNote::new(0, secrets[i], 0);
            actions::add_new_card(storage.cards, &mut card);
        }
    }
    // docs:end:functions-SecretFunction

    #[aztec(private)]
    fn update_legendary_card(
        new_points: u8,
        new_secret: Field,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let owner = inputs.call_context.msg_sender;
        let mut updated_card = CardNote::new(new_points, new_secret, owner);

        assert(actions::is_valid_card(storage.game_rules, updated_card));

        actions::update_legendary_card(storage.legendary_card, &mut updated_card);
    }

    #[aztec(private)]
    fn become_queen() {
        let storage = Storage::init(Context::private(&mut context));

        let legendary_card = actions::get_legendary_card(storage.legendary_card);

        let owner = legendary_card.owner;
        let result = context.call_private_function(
            inputs.call_context.storage_contract_address, 
            GET_POINTS_OF_COMMON_CARD_FUNCTION_SELECTOR,
            [owner, 0]
        );
        let total_points = legendary_card.points + result[0] as u8;

        context.call_public_function(
            inputs.call_context.storage_contract_address, 
            REPLACE_QUEEN_FUNCTION_SELECTOR,
            [owner, total_points as Field]
        );
    }

    #[aztec(private)]
    fn get_points_of_common_cards(
        account: Field,
        offset: u32,
    ) {
        let storage = Storage::init(Context::private(&mut context));

        let mut total_points = 0;
        let options = create_account_card_getter_options(account, offset);
        let cards = actions::get_cards(storage.cards, options);
        for i in 0..cards.len() {
            if (cards[i].is_some()) {
                let card = cards[i].unwrap_unchecked();
                assert(card.owner == account);
                total_points += card.points;
            }
        }

        context.return_values.push(total_points as Field);
    }

    // docs:start:state_vars-check_return_notes
    #[aztec(private)]
    fn discard_largest_card() {
        let storage = Storage::init(Context::private(&mut context));

        let account = context.msg_sender();
        let options = create_largest_account_card_getter_options(account);
        let card = actions::get_cards(storage.cards, options)[0].unwrap_unchecked();
        // highlight-next-line:state_vars-check_return_notes
        assert(card.owner == account);

        actions::remove_card(storage.cards, card);
    }
    // docs:end:state_vars-check_return_notes

    // docs:start:functions-UncontrainedFunction
    unconstrained fn get_total_points(account: Field) -> u8 {
        let storage = Storage::init(Context::none());
        actions::get_total_points(storage.cards, account, 0)
    }
    // docs:end:functions-UncontrainedFunction


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
        b: Field
        
        // The actual return type of our circuit is the PrivateCircuitPublicInputs struct, this will be the 
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


    // Cross chain messaging section
    // Demonstrates a cross chain message 
    // docs:start:l1_to_l2_cross_chain_message 
    #[aztec(private)]
    fn send_to_l1() {


    }
    // docs:end:l1_to_l2_cross_chain_message  
}
