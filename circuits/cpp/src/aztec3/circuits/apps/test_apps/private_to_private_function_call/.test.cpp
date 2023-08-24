#include "index.hpp"

#include "aztec3/circuits/abis/call_context.hpp"
#include "aztec3/circuits/abis/function_data.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

class private_to_private_function_call_tests : public ::testing::Test {};

TEST(private_to_private_function_call_tests, circuit_private_to_private_function_call)
{
    C fn1_builder = C();
    DB db;

    const NT::address contract_address = 12345;
    const NT::fr msg_sender_private_key = 123456789;
    const NT::address msg_sender =
        uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL, 0x2ef9f7f09867fd6eULL);

    const FunctionData<NT> function_data{
        .function_selector =
            FunctionSelector<NT>{
                .value = 1,  // TODO: deduce this from the contract, somehow.
            },
        .is_private = true,
        .is_constructor = false,
    };

    const CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .portal_contract_address = 0,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };

    NativeOracle fn1_oracle = NativeOracle(db, contract_address, function_data, call_context, msg_sender_private_key);
    OracleWrapper fn1_oracle_wrapper = OracleWrapper(fn1_builder, fn1_oracle);

    FunctionExecutionContext fn1_exec_ctx(fn1_builder, fn1_oracle_wrapper);

    auto a = NT::fr(111);
    auto b = NT::fr(222);
    auto c = NT::fr(333);

    function_1_1(fn1_exec_ctx, { a, b, c, 0, 0, 0, 0, 0 });

    const auto& function_1_1_public_inputs = fn1_exec_ctx.get_final_private_circuit_public_inputs();

    info("function_1_1_public_inputs: ", function_1_1_public_inputs);

    // info("witness: ", fn1_builder.witness);
    // info("constant variables: ", fn1_builder.constant_variables);
    // info("variables: ", fn1_builder.variables);
    info("failed?: ", fn1_builder.failed());
    info("err: ", fn1_builder.err());
    info("n: ", fn1_builder.num_gates);
}

}  // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call