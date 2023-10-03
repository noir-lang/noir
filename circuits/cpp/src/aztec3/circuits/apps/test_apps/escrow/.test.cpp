#include "index.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

namespace aztec3::circuits::apps::test_apps::escrow {

class escrow_tests : public ::testing::Test {
  protected:
    static NativeOracle get_test_native_oracle(DB& db)
    {
        const NT::address contract_address = 12345;
        const NT::fr msg_sender_private_key = 123456789;
        const NT::address msg_sender = NT::fr(
            uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL));

        FunctionData<NT> const function_data{
            .selector =
                {
                    .value = 1,  // TODO: deduce this from the contract, somehow.
                },
            .is_private = true,
            .is_constructor = false,
        };

        CallContext<NT> const call_context{
            .msg_sender = msg_sender,
            .storage_contract_address = contract_address,
            .portal_contract_address = 0,
            .function_selector = function_data.selector,
            .is_delegate_call = false,
            .is_static_call = false,
            .is_contract_deployment = false,
        };

        return NativeOracle(db, contract_address, function_data, call_context, msg_sender_private_key);
    };
};

TEST_F(escrow_tests, circuit_deposit)
{
    // TODO: currently, we can't hide all of this boilerplate in a test fixture function, because each of these classes
    // contains a reference to earlier-declared classes... so we'd end up with classes containing dangling references,
    // if all this stuff were to be declared in a setup function's scope.
    // We could instead store shared_ptrs in every class...?
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext exec_ctx(builder, oracle_wrapper);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);

    auto result = deposit(exec_ctx, { amount, asset_id, memo });
    info("result: ", result);

    // info("witness: ", builder.witness);
    // info("constant variables: ", builder.constant_variables);
    // info("variables: ", builder.variables);
    info("failed?: ", builder.failed());
    info("err: ", builder.err());
    info("n: ", builder.num_gates);
}

TEST_F(escrow_tests, circuit_transfer)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext exec_ctx(builder, oracle_wrapper);

    auto amount = NT::fr(5);
    auto to = NT::address(657756);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);
    auto reveal_msg_sender_to_recipient = true;
    auto fee = NT::fr(2);

    transfer(exec_ctx, amount, to, asset_id, memo, reveal_msg_sender_to_recipient, fee);

    // info("witness: ", builder.witness);
    // info("constant variables: ", builder.constant_variables);
    // info("variables: ", builder.variables);
    info("failed?: ", builder.failed());
    info("err: ", builder.err());
    info("n: ", builder.num_gates);
}

TEST_F(escrow_tests, circuit_withdraw)
{
    C builder = C();
    DB db;
    NativeOracle native_oracle = get_test_native_oracle(db);
    OracleWrapper oracle_wrapper = OracleWrapper(builder, native_oracle);
    FunctionExecutionContext exec_ctx(builder, oracle_wrapper);

    auto amount = NT::fr(5);
    auto asset_id = NT::fr(1);
    auto memo = NT::fr(999);
    auto l1_withdrawal_address = NT::fr(657756);
    auto fee = NT::fr(2);

    withdraw(exec_ctx, amount, asset_id, memo, l1_withdrawal_address, fee);

    // info("witness: ", builder.witness);
    // info("constant variables: ", builder.constant_variables);
    // info("variables: ", builder.variables);
    info("failed?: ", builder.failed());
    info("err: ", builder.err());
    info("n: ", builder.num_gates);
}

}  // namespace aztec3::circuits::apps::test_apps::escrow