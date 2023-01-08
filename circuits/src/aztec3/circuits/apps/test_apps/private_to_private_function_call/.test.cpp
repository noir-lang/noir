#include "index.hpp"

#include <gtest/gtest.h>
#include <common/test.hpp>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

class private_to_private_function_call_tests : public ::testing::Test {};

TEST(private_to_private_function_call_tests, test_private_to_private_function_call)
{

    C fn1_composer;
    DB db;

    const NT::address contract_address = 12345;
    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL);
    const NT::address tx_origin = msg_sender;

    NativeOracle fn1_oracle = NativeOracle(db, contract_address, msg_sender, tx_origin, msg_sender_private_key);
    OracleWrapper fn1_oracle_wrapper = OracleWrapper(fn1_composer, fn1_oracle);

    FunctionExecutionContext fn1_exec_ctx(fn1_composer, fn1_oracle_wrapper);

    auto a = NT::fr(111);
    auto b = NT::fr(222);
    auto c = NT::fr(333);

    auto result = function_1_1(fn1_exec_ctx, a, b, c);

    info("result: ", result);

    info("computed witness: ", fn1_composer.computed_witness);
    info("witness: ", fn1_composer.witness);
    // info("constant variables: ", fn1_composer.constant_variables);
    // info("variables: ", fn1_composer.variables);
    info("failed?: ", fn1_composer.failed);
    info("err: ", fn1_composer.err);
    info("n: ", fn1_composer.n);
}

} // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call