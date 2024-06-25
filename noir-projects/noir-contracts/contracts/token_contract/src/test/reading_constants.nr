use crate::test::utils;
use dep::aztec::test::helpers::cheatcodes;
use crate::Token;

// It is not possible to deserialize strings in Noir ATM, so name and symbol cannot be checked yet.

#[test]
unconstrained fn check_decimals_private() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, _, _) = utils::setup(/* with_account_contracts */ false);

    // Check decimals
    let private_get_decimals_call_interface = Token::at(token_contract_address).private_get_decimals();
    let result = env.call_private(private_get_decimals_call_interface);

    assert(result == 18);
}

#[test]
unconstrained fn check_decimals_public() {
    // Setup without account contracts. We are not using authwits here, so dummy accounts are enough
    let (env, token_contract_address, _, _) = utils::setup(/* with_account_contracts */ false);

    // Check decimals
    let public_get_decimals_call_interface = Token::at(token_contract_address).public_get_decimals();
    let result = env.call_public(public_get_decimals_call_interface);

    assert(result == 18 as u8);
}
