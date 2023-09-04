mod account_contract_interface;
mod actions;
mod options;
mod storage;
mod types;

contract DocsExample {
    use crate::account_contract_interface::AccountContractInterface;
    use crate::actions;
    use crate::options::{create_account_card_getter_options, create_largest_account_card_getter_options};
    // docs:start:storage-import
    use crate::storage::Storage;
    // docs:end:storage-import
    use crate::types::{
        card_note::CardNote,
        queen::Queen,
        rules_note::RulesNote,
    };
    use dep::std::option::Option;

    global REPLACE_QUEEN_FUNCTION_SELECTOR = 11111111;
    global GET_POINTS_OF_COMMON_CARD_FUNCTION_SELECTOR = 11111111;

    #[aztec(private)]
    fn constructor(
        min_points: u8,
        max_points: u8,
        legendary_card_secret: Field,
    ) {
        let storage = Storage::init(Option::some(&mut context), Option::none());

        let mut game_rules = RulesNote::new(min_points, max_points);
        actions::init_game_rules(storage.game_rules, &mut game_rules);

        let mut legendary_card = CardNote::new(0, legendary_card_secret, 0);
        actions::init_legendary_card(storage.legendary_card, &mut legendary_card);
    }

    // docs:start:storage-init
    #[aztec(public)]
    fn lock() {
        // highlight-next-line:storage-init
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        storage.locked.write(true);
    }
    // docs:end:storage-init

    // docs:start:functions-OpenFunction
    #[aztec(public)]
    fn unlock() {
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        actions::unlock(storage.locked);
    }
    // docs:end:functions-OpenFunction

    #[aztec(public)]
    fn replace_queen(
        account: Field,
        points: u8,
    ) {
        let storage = Storage::init(Option::none(), Option::some(&mut context));

        let new_queen = Queen { account, points };
        
        assert(actions::can_replace_queen(storage.queen, new_queen));
        
        actions::replace_queen(storage.queen, new_queen);
    }

    // docs:start:state_vars-PublicStateWriteBeforeCall
    #[aztec(public)]
    fn replace_queen_unsafe() {
        let storage = Storage::init(Option::none(), Option::some(&mut context));

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
        let storage = Storage::init(Option::some(&mut context), Option::none());

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
        let storage = Storage::init(Option::some(&mut context), Option::none());

        let owner = inputs.call_context.msg_sender;
        let mut updated_card = CardNote::new(new_points, new_secret, owner);

        assert(actions::is_valid_card(storage.game_rules, updated_card));

        actions::update_legendary_card(storage.legendary_card, &mut updated_card);
    }

    #[aztec(private)]
    fn become_queen() {
        let storage = Storage::init(Option::some(&mut context), Option::none());

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
        let storage = Storage::init(Option::some(&mut context), Option::none());

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
        let storage = Storage::init(Option::some(&mut context), Option::none());

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
        let storage = Storage::init(Option::none(), Option::none());
        actions::get_total_points(storage.cards, account, 0)
    }
    // docs:end:functions-UncontrainedFunction
}
