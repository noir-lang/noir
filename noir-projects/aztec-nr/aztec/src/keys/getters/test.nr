use crate::keys::getters::{get_current_public_keys, get_historical_public_keys, KEY_REGISTRY_UPDATE_BLOCKS};
use crate::context::PrivateContext;
use dep::protocol_types::address::AztecAddress;

use crate::test::helpers::{cheatcodes, test_environment::TestEnvironment, utils::TestAccount};
use dep::std::test::OracleMock;

global KEY_ORACLE_RESPONSE_LENGTH = 13; // 12 fields for the keys, one field for the partial address

fn setup() -> (TestEnvironment, PrivateContext, TestAccount) {
    let mut env = TestEnvironment::new();
    let account = cheatcodes::create_account();

    let historical_block_number = env.block_number();
    let context = env.private_at(historical_block_number);

    (env, context, account)
}

#[test(should_fail_with="Invalid public keys hint for address")]
fn test_get_current_keys_unknown_unregistered() {
    let (_, context, account) = setup();

    let _ = OracleMock::mock("getPublicKeysAndPartialAddress").returns([0; KEY_ORACLE_RESPONSE_LENGTH]).times(1);
    let _ = get_current_public_keys(&mut context, account.address);
}

#[test(should_fail_with="Invalid public keys hint for address")]
fn test_get_historical_keys_unknown_unregistered() {
    let (_, context, account) = setup();
    let historical_header = context.get_header();

    let _ = OracleMock::mock("getPublicKeysAndPartialAddress").returns([0; KEY_ORACLE_RESPONSE_LENGTH]).times(1);
    let _ = get_historical_public_keys(historical_header, account.address);
}

#[test]
fn test_get_current_keys_known_unregistered() {
    let (_, mut context, account) = setup();

    let current_keys = get_current_public_keys(&mut context, account.address);

    assert_eq(current_keys, account.keys);
    assert_eq(
        context.max_block_number.unwrap(), context.historical_header.global_variables.block_number as u32 + KEY_REGISTRY_UPDATE_BLOCKS
    );
}

#[test]
fn test_get_historical_keys_known_unregistered() {
    let (_, context, account) = setup();

    let historical_header = context.get_header();

    let historical_keys = get_historical_public_keys(historical_header, account.address);
    assert_eq(historical_keys, account.keys);
    assert(context.max_block_number.is_none());
}
