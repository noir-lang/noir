#include "index.hpp"
#include "previous_kernel_data.hpp"

#include "aztec3/circuits/abis/combined_accumulated_data.hpp"

#include <barretenberg/common/serialize.hpp>
#include <barretenberg/serialize/cbind.hpp>

#include <gtest/gtest.h>

namespace {
// Composer
using Composer = plonk::UltraComposer;

// Types
using CT = aztec3::utils::types::CircuitTypes<Composer>;
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
        .function_selector = 11,
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
        .function_selector = 11,
        .is_private = false,
        .is_constructor = false,
    };

    info("function data: ", native_function_data);

    Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
    FunctionData<CT> const circuit_function_data = native_function_data.to_circuit_type(composer);

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

    Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
    CallContext<CT> const circuit_call_context = native_call_context.to_circuit_type(composer);

    info("call context: ", circuit_call_context);
}

}  // namespace aztec3::circuits::abis
