use crate::{test::utils, Token, types::token_note::TokenNote};

use dep::aztec::{
    test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand, hash::compute_secret_hash,
    prelude::NoteHeader, protocol_types::storage::map::derive_storage_slot_in_map,
    keys::getters::get_current_public_keys
};
use dep::authwit::cheatcodes as authwit_cheatcodes;

#[test]
unconstrained fn setup_refund_success() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(true);

    // Renaming owner and recipient to match naming in Token
    let user = owner;
    let fee_payer = recipient;

    let funded_amount = 1_000;
    let user_randomness = 42;
    let fee_payer_randomness = 123;
    let mut context = env.private();

    let setup_refund_from_call_interface = Token::at(token_contract_address).setup_refund(
        fee_payer,
        user,
        funded_amount,
        user_randomness,
        fee_payer_randomness
    );

    authwit_cheatcodes::add_private_authwit_from_call_interface(user, fee_payer, setup_refund_from_call_interface);

    env.impersonate(fee_payer);

    env.call_private_void(setup_refund_from_call_interface);
    let mut context = env.private();
    let user_npk_m_hash = get_current_public_keys(&mut context, user).npk_m.hash();
    let fee_payer_npk_m_hash = get_current_public_keys(&mut context, fee_payer).npk_m.hash();

    let fee_payer_balances_slot = derive_storage_slot_in_map(Token::storage().balances.slot, fee_payer);
    let user_balances_slot = derive_storage_slot_in_map(Token::storage().balances.slot, user);

    // When the refund was set up, we would've spent the note worth mint_amount, and inserted a note worth
    //`mint_amount - funded_amount`. When completing the refund, we would've constructed a hash corresponding to a note
    // worth  `funded_amount - transaction_fee`. We "know" the transaction fee was 1 (it is hardcoded in
    // `executePublicFunction` TXE oracle) but we need to notify TXE of the note (preimage).
    env.store_note_in_cache(
        &mut TokenNote {
        amount: U128::from_integer(funded_amount - 1),
        npk_m_hash: user_npk_m_hash,
        randomness: user_randomness,
        header: NoteHeader::empty()
    },
        user_balances_slot,
        token_contract_address
    );
    env.store_note_in_cache(
        &mut TokenNote {
        amount: U128::from_integer(1),
        npk_m_hash: fee_payer_npk_m_hash,
        randomness: fee_payer_randomness,
        header: NoteHeader::empty()
    },
        fee_payer_balances_slot,
        token_contract_address
    );

    utils::check_private_balance(token_contract_address, user, mint_amount - 1);
    utils::check_private_balance(token_contract_address, fee_payer, 1)
}

// TODO(#7694): Ideally we would check the error message here but it's currently not supported by TXE. Once this
// is supported, check the message here and delete try deleting the corresponding e2e test.
// #[test(should_fail_with = "funded amount not enough to cover tx fee")]
#[test(should_fail)]
unconstrained fn setup_refund_insufficient_funded_amount() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(true);

    // Renaming owner and recipient to match naming in Token
    let user = owner;
    let fee_payer = recipient;

    // We set funded amount to 0 to make the transaction fee higher than the funded amount
    let funded_amount = 0;
    let user_randomness = 42;
    let fee_payer_randomness = 123;
    let mut context = env.private();

    let setup_refund_from_call_interface = Token::at(token_contract_address).setup_refund(
        fee_payer,
        user,
        funded_amount,
        user_randomness,
        fee_payer_randomness
    );

    authwit_cheatcodes::add_private_authwit_from_call_interface(user, fee_payer, setup_refund_from_call_interface);

    env.impersonate(fee_payer);

    // The following should fail with "funded amount not enough to cover tx fee" because funded amount is 0
    env.call_private_void(setup_refund_from_call_interface);
}
