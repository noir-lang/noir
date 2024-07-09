use crate::test::utils;
use dep::aztec::{
    test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand, hash::compute_secret_hash,
    prelude::NoteHeader
};
use crate::PrivateToken;
use crate::types::token_note::TokenNote;
use dep::authwit::cheatcodes as authwit_cheatcodes;

#[test]
unconstrained fn transfer_success() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(true);

    let transfer_amount = 1_000;

    let transfer_private_from_call_interface = PrivateToken::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);

    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, transfer_private_from_call_interface);

    env.impersonate(recipient);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
    // Check balances
    utils::check_private_balance(
        &mut env.private(),
        token_contract_address,
        owner,
        mint_amount - transfer_amount
    );
    utils::check_private_balance(
        &mut env.private(),
        token_contract_address,
        recipient,
        transfer_amount
    );
}

#[test]
unconstrained fn setup_refund_success() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(true);

    let funded_amount = 1_000;
    let user_randomness = 42;
    let fee_payer_randomness = 123;
    let mut context = env.private();
    let recipient_npk_m_hash = context.get_header().get_npk_m_hash(&mut context, recipient);

    let setup_refund_from_call_interface = PrivateToken::at(token_contract_address).setup_refund(
        recipient_npk_m_hash, // fee payer
        owner, // sponsored user
        funded_amount,
        user_randomness,
        fee_payer_randomness
    );

    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, setup_refund_from_call_interface);

    env.impersonate(recipient);

    env.call_private_void(setup_refund_from_call_interface);
    let mut context = env.private();
    let owner_npk_m_hash = context.get_header().get_npk_m_hash(&mut context, owner);
    let recipient_npk_m_hash = context.get_header().get_npk_m_hash(&mut context, recipient);

    // when the refund was set up, we would've broken the note worth mint_amount, and added back a note worth
    // mint_amount - funded_amount
    // then when completing the refund, we would've constructed a hash corresponding to a note worth 
    // funded_amount - transaction_fee
    // we "know" the transaction fee was 1 (it is hardcoded in TXE oracle)
    // but we need to notify TXE of the note (preimage)
    env.store_note_in_cache(
        &mut TokenNote {
        amount: U128::from_integer(funded_amount - 1),
        npk_m_hash: owner_npk_m_hash,
        randomness: user_randomness,
        header: NoteHeader::empty()
    },
        PrivateToken::storage().balances.slot,
        token_contract_address
    );
    env.store_note_in_cache(
        &mut TokenNote {
        amount: U128::from_integer(1),
        npk_m_hash: recipient_npk_m_hash,
        randomness: fee_payer_randomness,
        header: NoteHeader::empty()
    },
        PrivateToken::storage().balances.slot,
        token_contract_address
    );

    utils::check_private_balance(
        &mut env.private(),
        token_contract_address,
        owner,
        mint_amount - 1
    );
    utils::check_private_balance(&mut env.private(), token_contract_address, recipient, 1)
}

