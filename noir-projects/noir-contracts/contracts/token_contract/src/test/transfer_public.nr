use crate::test::utils;
use dep::aztec::{test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand};
use dep::authwit::cheatcodes as authwit_cheatcodes;
use crate::Token;

#[test]
unconstrained fn public_transfer() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    // Transfer tokens
    let transfer_amount = mint_amount / 10;
    let public_transfer_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 0);
    env.call_public(public_transfer_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_public_balance(token_contract_address, recipient, transfer_amount);
}

#[test]
unconstrained fn public_transfer_to_self() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    // Transfer tokens
    let transfer_amount = mint_amount / 10;
    // docs:start:call_public
    let public_transfer_call_interface = Token::at(token_contract_address).transfer_public(owner, owner, transfer_amount, 0);
    env.call_public(public_transfer_call_interface);
    // docs:end:call_public

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn public_transfer_on_behalf_of_other() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let transfer_amount = mint_amount / 10;
    let public_transfer_from_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, recipient, public_transfer_from_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Transfer tokens
    env.call_public(public_transfer_from_call_interface);
    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_public_balance(token_contract_address, recipient, transfer_amount);
}

#[test]
unconstrained fn public_transfer_failure_more_than_balance() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    // Transfer tokens
    let transfer_amount = mint_amount + 1;
    // docs:start:assert_public_fail
    let public_transfer_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 0);
    // Try to transfer tokens
    env.assert_public_call_fails(public_transfer_call_interface);
    // docs:end:assert_public_fail
    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn public_transfer_failure_on_behalf_of_self_non_zero_nonce() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Transfer tokens
    let transfer_amount = mint_amount / 10;
    let public_transfer_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, unsafe_rand());
    // Try to transfer tokens
    env.assert_public_call_fails(public_transfer_call_interface);

    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn public_transfer_failure_on_behalf_of_other_without_approval() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let transfer_amount = mint_amount / 10;
    let public_transfer_from_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 1);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Try to transfer tokens
    env.assert_public_call_fails(public_transfer_from_call_interface);
    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_public_balance(token_contract_address, recipient, 0);
}

#[test]
unconstrained fn public_transfer_failure_on_behalf_of_other_more_than_balance() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let transfer_amount = mint_amount + 1;
    // docs:start:public_authwit
    let public_transfer_from_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, recipient, public_transfer_from_call_interface);
    // docs:end:public_authwit
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Try to transfer tokens
    env.assert_public_call_fails(public_transfer_from_call_interface);
    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_public_balance(token_contract_address, recipient, 0);
}

#[test]
unconstrained fn public_transfer_failure_on_behalf_of_other_wrong_caller() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let transfer_amount = mint_amount / 10;
    let public_transfer_from_call_interface = Token::at(token_contract_address).transfer_public(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, owner, public_transfer_from_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Try to transfer tokens
    env.assert_public_call_fails(public_transfer_from_call_interface);
    // Check balances
    utils::check_public_balance(token_contract_address, owner, mint_amount);
    utils::check_public_balance(token_contract_address, recipient, 0);
}
