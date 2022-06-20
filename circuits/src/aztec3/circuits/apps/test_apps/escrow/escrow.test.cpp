#include <gtest/gtest.h>
#include <common/test.hpp>
// #include <common/serialize.hpp>
// #include <stdlib/types/turbo.hpp>
// #include <aztec3/oracle/oracle.hpp>
// #include <aztec3/circuits/apps/oracle_wrapper.hpp>
// #include <numeric/random/engine.hpp>
#include "index.hpp"
// #include "deposit.hpp"
// #include <aztec3/constants.hpp>
// #include <crypto/pedersen/pedersen.hpp>
// #include <stdlib/hash/pedersen/pedersen.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

class escrow_tests : public ::testing::Test {};

TEST(escrow_tests, test_deposit)
{

    Composer composer;
    DB db;

    const NT::address contract_address = 12345;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::fr msg_sender_private_key = 123456789;

    NativeOracle oracle = NativeOracle(db, contract_address, msg_sender, msg_sender_private_key);
    OracleWrapper oracle_wrapper = OracleWrapper(composer, oracle);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    auto result = deposit(composer, oracle_wrapper, amount, asset_id, memo);
    info("result: ", result);

    info("computed witness: ", composer.computed_witness);
    info("witness: ", composer.witness);
    // info("constant variables: ", composer.constant_variables);
    // info("variables: ", composer.variables);
    info("failed?: ", composer.failed);
    info("err: ", composer.err);
    info("n: ", composer.n);
}

TEST(escrow_tests, test_transfer)
{

    Composer composer;
    DB db;

    const NT::address contract_address = 12345;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::fr msg_sender_private_key = 123456789;

    CallContext<NT> call_context = {
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .is_fee_payment = true,
    };

    NativeOracle oracle = NativeOracle(db, call_context, msg_sender_private_key);
    OracleWrapper oracle_wrapper = OracleWrapper(composer, oracle);

    auto amount = NT::fr(5);
    auto to = NT::address(657756);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);
    auto reveal_msg_sender_to_recipient = true;
    auto fee = NT::fr(2);

    transfer(composer, oracle_wrapper, amount, to, asset_id, memo, reveal_msg_sender_to_recipient, fee);

    info("computed witness: ", composer.computed_witness);
    info("witness: ", composer.witness);
    // info("constant variables: ", composer.constant_variables);
    // info("variables: ", composer.variables);
    info("failed?: ", composer.failed);
    info("err: ", composer.err);
    info("n: ", composer.n);
}

TEST(escrow_tests, test_withdraw)
{

    Composer composer;
    DB db;

    const NT::address contract_address = 12345;
    const NT::address msg_sender =
        NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));
    const NT::fr msg_sender_private_key = 123456789;

    CallContext<NT> call_context = {
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .is_fee_payment = true,
    };

    NativeOracle oracle = NativeOracle(db, call_context, msg_sender_private_key);
    OracleWrapper oracle_wrapper = OracleWrapper(composer, oracle);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);
    auto l1_withdrawal_address = NT::fr(657756);
    auto fee = NT::fr(2);

    withdraw(composer, oracle_wrapper, amount, asset_id, memo, l1_withdrawal_address, fee);

    info("computed witness: ", composer.computed_witness);
    info("witness: ", composer.witness);
    // info("constant variables: ", composer.constant_variables);
    // info("variables: ", composer.variables);
    info("failed?: ", composer.failed);
    info("err: ", composer.err);
    info("n: ", composer.n);
}

} // namespace aztec3::circuits::apps::test_apps::escrow