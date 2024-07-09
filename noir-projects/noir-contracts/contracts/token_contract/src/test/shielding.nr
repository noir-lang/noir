use crate::test::utils;
use dep::aztec::{test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand, hash::compute_secret_hash};
use dep::authwit::cheatcodes as authwit_cheatcodes;
use crate::{types::transparent_note::TransparentNote, Token};

#[test]
unconstrained fn shielding_on_behalf_of_self() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens
    let shield_amount = mint_amount / 10;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    env.call_public(shield_call_interface);

    // Store a note in the cache so we can redeem it
    env.store_note_in_cache(
        &mut TransparentNote::new(shield_amount, secret_hash),
        Token::storage().pending_shields.slot,
        token_contract_address
    );

    // Redeem our shielded tokens
    let redeem_shield_call_interface = Token::at(token_contract_address).redeem_shield(owner, shield_amount, secret);
    env.call_private_void(redeem_shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount - shield_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount + shield_amount);
}

#[test]
unconstrained fn shielding_on_behalf_of_other() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);

    // Shield tokens on behalf of owner
    let shield_amount = 1000;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, recipient, shield_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Shield tokens
    env.call_public(shield_call_interface);

    // Become owner again
    env.impersonate(owner);
    // Store a note in the cache so we can redeem it
    env.store_note_in_cache(
        &mut TransparentNote::new(shield_amount, secret_hash),
        Token::storage().pending_shields.slot,
        token_contract_address
    );

    // Redeem our shielded tokens
    let redeem_shield_call_interface = Token::at(token_contract_address).redeem_shield(owner, shield_amount, secret);
    env.call_private_void(redeem_shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount - shield_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount + shield_amount);
}

#[test]
unconstrained fn shielding_failure_on_behalf_of_self_more_than_balance() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens
    let shield_amount = mint_amount + 1;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    env.assert_public_call_fails(shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn shielding_failure_on_behalf_of_self_invalid_nonce() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens
    let shield_amount = mint_amount / 10;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, unsafe_rand());
    env.assert_public_call_fails(shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn shielding_failure_on_behalf_of_other_more_than_balance() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens on behalf of owner
    let shield_amount = mint_amount + 1;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, recipient, shield_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Shield tokens
    env.assert_public_call_fails(shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn shielding_failure_on_behalf_of_other_wrong_caller() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens on behalf of owner
    let shield_amount = mint_amount + 1;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, owner, shield_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Shield tokens
    env.assert_public_call_fails(shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn shielding_failure_on_behalf_of_other_without_approval() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let secret = unsafe_rand();
    let secret_hash = compute_secret_hash(secret);
    // Shield tokens on behalf of owner
    let shield_amount = mint_amount + 1;
    let shield_call_interface = Token::at(token_contract_address).shield(owner, shield_amount, secret_hash, 0);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Shield tokens
    env.assert_public_call_fails(shield_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

