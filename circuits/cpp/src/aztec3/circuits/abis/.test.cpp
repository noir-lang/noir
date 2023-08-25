#include "index.hpp"
#include "previous_kernel_data.hpp"

#include "aztec3/circuits/abis/combined_accumulated_data.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

namespace {
// Builder
using Builder = UltraCircuitBuilder;

// Types
using CT = aztec3::utils::types::CircuitTypes<Builder>;
using NT = aztec3::utils::types::NativeTypes;
}  // namespace

namespace aztec3::circuits::abis {

class abi_tests : public ::testing::Test {};

TEST(abi_tests, msgpack_schema_smoke_test)
{
    // Just exercise these to make sure they don't error
    // They will test for any bad serialization methods
    msgpack_schema_to_string(CombinedAccumulatedData<NT>{});
    CombinedAccumulatedData<NT> cad;
    EXPECT_EQ(msgpack::check_msgpack_method(cad), "");
}

TEST(abi_tests, native_read_write_call_context)
{
    CallContext<NT> const call_context = {
        .msg_sender = 1,
        .storage_contract_address = 2,
        .portal_contract_address = 3,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };

    info("call_context: ", call_context);

    auto buffer = to_buffer(call_context);
    auto call_context_2 = from_buffer<CallContext<NT>>(buffer.data());

    EXPECT_EQ(call_context, call_context_2);
}

TEST(abi_tests, native_read_write_function_data)
{
    FunctionData<NT> const function_data = {
        .selector =
            {
                .value = 11,
            },
        .is_private = false,
        .is_constructor = false,
    };

    info("function data: ", function_data);

    auto buffer = to_buffer(function_data);
    auto function_data_2 = from_buffer<FunctionData<NT>>(buffer.data());

    EXPECT_EQ(function_data, function_data_2);
}

TEST(abi_tests, native_to_circuit_function_data)
{
    FunctionData<NT> const native_function_data = {
        .selector =
            {
                .value = 11,
            },
        .is_private = false,
        .is_constructor = false,
    };

    info("function data: ", native_function_data);

    Builder builder = Builder();
    FunctionData<CT> const circuit_function_data = native_function_data.to_circuit_type(builder);

    info("function data: ", circuit_function_data);
}

TEST(abi_tests, native_call_context)
{
    CallContext<NT> const call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
        .portal_contract_address = 12,
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call context: ", call_context);
}

TEST(abi_tests, native_to_circuit_call_context)
{
    CallContext<NT> const native_call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
        .portal_contract_address = 12,
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call context: ", native_call_context);

    Builder builder = Builder();
    CallContext<CT> const circuit_call_context = native_call_context.to_circuit_type(builder);

    info("call context: ", circuit_call_context);
}

}  // namespace aztec3::circuits::abis
