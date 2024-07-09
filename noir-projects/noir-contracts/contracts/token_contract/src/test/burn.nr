use crate::test::utils;
use dep::aztec::{test::helpers::cheatcodes, oracle::unsafe_rand::unsafe_rand};
use dep::authwit::cheatcodes as authwit_cheatcodes;
use crate::Token;

#[test]
unconstrained fn burn_public_success() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    let burn_amount = mint_amount / 10;

    // Burn less than balance 
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, 0);
    env.call_public(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount - burn_amount);
}

#[test]
unconstrained fn burn_public_on_behalf_of_other() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let burn_amount = mint_amount / 10;

    // Burn on behalf of other
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, recipient, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Burn tokens
    env.call_public(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount - burn_amount);
}

#[test]
unconstrained fn burn_public_failure_more_than_balance() {
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    // Burn more than balance 
    let burn_amount = mint_amount * 10;
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, 0);
    env.assert_public_call_fails(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn burn_public_failure_on_behalf_of_self_non_zero_nonce() {
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    // Burn on behalf of self with non-zero nonce
    let burn_amount = mint_amount / 10;
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, unsafe_rand());
    env.assert_public_call_fails(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn burn_public_failure_on_behalf_of_other_without_approval() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    // Burn on behalf of other without approval
    let burn_amount = mint_amount / 10;
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, unsafe_rand());
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.assert_public_call_fails(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount);

    // Burn on behalf of other, wrong designated caller
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, owner, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.assert_public_call_fails(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn burn_public_failure_on_behalf_of_other_wrong_caller() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    // Burn on behalf of other, wrong designated caller
    let burn_amount = mint_amount / 10;
    let burn_call_interface = Token::at(token_contract_address).burn_public(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_public_authwit_from_call_interface(owner, owner, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.assert_public_call_fails(burn_call_interface);
    utils::check_public_balance(token_contract_address, owner, mint_amount);
}

#[test]
unconstrained fn burn_private_on_behalf_of_self() {
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);
    let burn_amount = mint_amount / 10;

    // Burn less than balance 
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, 0);
    env.call_private_void(burn_call_interface);
    utils::check_private_balance(token_contract_address, owner, mint_amount - burn_amount);
}

#[test]
unconstrained fn burn_private_on_behalf_of_other() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);
    let burn_amount = mint_amount / 10;

    // Burn on behalf of other
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    // Burn tokens
    env.call_private_void(burn_call_interface);
    utils::check_private_balance(token_contract_address, owner, mint_amount - burn_amount);
}

#[test(should_fail_with="Balance too low")]
unconstrained fn burn_private_failure_more_than_balance() {
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    // Burn more than balance 
    let burn_amount = mint_amount * 10;
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, 0);
    env.call_private_void(burn_call_interface);
    // Private doesnt revert, so we cannot check balances here since notes have already been nullified. Test is done.
}

#[test(should_fail_with="invalid nonce")]
unconstrained fn burn_private_failure_on_behalf_of_self_non_zero_nonce() {
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    // Burn more than balance 
    let burn_amount = mint_amount / 10;
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, unsafe_rand());
    env.call_private_void(burn_call_interface);
    // Private doesnt revert, so we cannot check balances here since notes have already been nullified. Test is done.
}

#[test(should_fail)]
unconstrained fn burn_private_failure_on_behalf_of_other_more_than_balance() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    // Burn more than balance 
    let burn_amount = mint_amount * 10;
    // Burn on behalf of other
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.call_private_void(burn_call_interface);
    // Private doesnt revert, so we cannot check balances here since notes have already been nullified. Test is done.
}

#[test(should_fail)]
unconstrained fn burn_private_failure_on_behalf_of_other_without_approval() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    // Burn more than balance 
    let burn_amount = mint_amount / 10;
    // Burn on behalf of other
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, unsafe_rand());
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.call_private_void(burn_call_interface);
    // Private doesnt revert, so we cannot check balances here since notes have already been nullified. Test is done.
}

#[test(should_fail)]
unconstrained fn burn_private_failure_on_behalf_of_other_wrong_designated_caller() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    // Burn more than balance 
    let burn_amount = mint_amount / 10;
    // Burn on behalf of other
    let burn_call_interface = Token::at(token_contract_address).burn(owner, burn_amount, unsafe_rand());
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, owner, burn_call_interface);
    // Impersonate recipient to perform the call
    env.impersonate(recipient);
    env.call_private_void(burn_call_interface);
    // Private doesnt revert, so we cannot check balances here since notes have already been nullified. Test is done.
}
