use crate::test::utils;
use dep::aztec::{oracle::unsafe_rand::unsafe_rand, test::helpers::cheatcodes};
use dep::authwit::cheatcodes as authwit_cheatcodes;
use crate::Token;

#[test]
unconstrained fn unshield_on_behalf_of_self() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    let unshield_amount = mint_amount / 10;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, owner, unshield_amount, 0);
    env.call_private_void(unshield_call_interface);
    utils::check_private_balance(token_contract_address, owner, mint_amount - unshield_amount);
    utils::check_public_balance(token_contract_address, owner, mint_amount + unshield_amount);
}

#[test]
unconstrained fn unshield_on_behalf_of_other() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    let unshield_amount = mint_amount / 10;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, recipient, unshield_amount, 0);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, unshield_call_interface);
    // Impersonate recipient
    env.impersonate(recipient);
    // Unshield tokens
    env.call_private_void(unshield_call_interface);
    utils::check_private_balance(token_contract_address, owner, mint_amount - unshield_amount);
    utils::check_public_balance(token_contract_address, recipient, unshield_amount);
}

#[test(should_fail_with="Balance too low")]
unconstrained fn unshield_failure_more_than_balance() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    let unshield_amount = mint_amount + 1;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, owner, unshield_amount, 0);
    env.call_private_void(unshield_call_interface);
}

#[test(should_fail_with="invalid nonce")]
unconstrained fn unshield_failure_on_behalf_of_self_non_zero_nonce() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, owner, _, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ false);

    let unshield_amount = mint_amount + 1;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, owner, unshield_amount, unsafe_rand());
    env.call_private_void(unshield_call_interface);
}

#[test(should_fail_with="Balance too low")]
unconstrained fn unshield_failure_on_behalf_of_other_more_than_balance() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    let unshield_amount = mint_amount + 1;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, recipient, unshield_amount, 0);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, recipient, unshield_call_interface);
    // Impersonate recipient
    env.impersonate(recipient);
    // Unshield tokens
    env.call_private_void(unshield_call_interface);
}

#[test(should_fail)]
unconstrained fn unshield_failure_on_behalf_of_other_invalid_designated_caller() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    let unshield_amount = mint_amount + 1;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, recipient, unshield_amount, 0);
    authwit_cheatcodes::add_private_authwit_from_call_interface(owner, owner, unshield_call_interface);
    // Impersonate recipient
    env.impersonate(recipient);
    // Unshield tokens
    env.call_private_void(unshield_call_interface);
}

#[test(should_fail)]
unconstrained fn unshield_failure_on_behalf_of_other_no_approval() {
    let (env, token_contract_address, owner, recipient, mint_amount) = utils::setup_and_mint(/* with_account_contracts */ true);

    let unshield_amount = mint_amount + 1;
    let unshield_call_interface = Token::at(token_contract_address).unshield(owner, recipient, unshield_amount, 0);
    // Impersonate recipient
    env.impersonate(recipient);
    // Unshield tokens
    env.call_private_void(unshield_call_interface);
}
