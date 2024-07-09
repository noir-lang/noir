use crate::test::utils;
use dep::aztec::{test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand, protocol_types::address::AztecAddress};
use dep::authwit::cheatcodes as authwit_cheatcodes;
use crate::Token;

#[test]
unconstrained fn transfer_private() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    // docs:start:txe_test_transfer_private
    // Transfer tokens
    let transfer_amount = 1000;
    let transfer_private_call_interface = Token::at(token_contract_address).transfer(recipient, transfer_amount);
    env.call_private_void(transfer_private_call_interface);
    // docs:end:txe_test_transfer_private

    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_private_balance(token_contract_address, recipient, transfer_amount);
}

#[test]
unconstrained fn transfer_private_to_self() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    // Transfer tokens
    let transfer_amount = 1000;
    let transfer_private_call_interface = Token::at(token_contract_address).transfer(owner, transfer_amount);
    env.call_private_void(transfer_private_call_interface);

    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn transfer_private_to_non_deployed_account() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    let not_deployed = cheatcodes::create_account();
    // Transfer tokens
    let transfer_amount = 1000;
    let transfer_private_call_interface = Token::at(token_contract_address).transfer(not_deployed.address, transfer_amount);
    env.call_private_void(transfer_private_call_interface);

    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_private_balance(token_contract_address, not_deployed.address, transfer_amount);
}

#[test]
unconstrained fn transfer_private_on_behalf_of_other() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Add authwit
    // docs:start:private_authwit
    let transfer_amount = 1000;
    let transfer_private_from_call_interface = Token::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, transfer_private_from_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
    // docs:end:private_authwit
    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_private_balance(token_contract_address, recipient, transfer_amount);
}

#[test(should_fail_with="Balance too low")]
unconstrained fn transfer_private_failure_more_than_balance() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, _, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    // Transfer tokens
    let transfer_amount = mint_amount + 1;
    let transfer_private_call_interface = Token::at(token_contract_address).transfer(recipient, transfer_amount);
    env.call_private_void(transfer_private_call_interface);
}

// docs:start:fail_with_message
#[test(should_fail_with="invalid nonce")]
unconstrained fn transfer_private_failure_on_behalf_of_self_non_zero_nonce() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, _) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Add authwit
    let transfer_amount = 1000;
    let transfer_private_from_call_interface = Token::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
}
// docs:end:fail_with_message

#[test(should_fail_with="Balance too low")]
unconstrained fn transfer_private_failure_on_behalf_of_more_than_balance() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Add authwit
    let transfer_amount = mint_amount + 1;
    let transfer_private_from_call_interface = Token::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, transfer_private_from_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
}

#[test(should_fail)]
unconstrained fn transfer_private_failure_on_behalf_of_other_without_approval() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Add authwit
    let transfer_amount = 1000;
    let transfer_private_from_call_interface = Token::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_private_balance(token_contract_address, recipient, transfer_amount);
}

#[test(should_fail)]
unconstrained fn transfer_private_failure_on_behalf_of_other_wrong_caller() {
    // Setup with account contracts. Slower since we actually deploy them, but needed for authwits.
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    // Add authwit
    let transfer_amount = 1000;
    let transfer_private_from_call_interface = Token::at(token_contract_address).transfer_from(owner, recipient, transfer_amount, 1);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, owner, transfer_private_from_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Transfer tokens
    env.call_private_void(transfer_private_from_call_interface);
    // Check balances
    utils::check_private_balance(token_contract_address, owner, mint_amount - transfer_amount);
    utils::check_private_balance(token_contract_address, recipient, transfer_amount);
}
