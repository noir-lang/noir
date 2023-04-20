#include <gtest/gtest.h>
#include <barretenberg/common/test.hpp>
#include <barretenberg/common/serialize.hpp>
#include "index.hpp"
#include <barretenberg/stdlib/types/types.hpp>

#include "previous_kernel_data.hpp"
#include "private_kernel/private_inputs.hpp"

namespace aztec3::circuits::abis {

using Composer = plonk::stdlib::types::Composer;
using CT = aztec3::utils::types::CircuitTypes<Composer>;
using NT = aztec3::utils::types::NativeTypes;

class abi_tests : public ::testing::Test {};

TEST(abi_tests, native_read_write_call_context)
{
    CallContext<NT> call_context = {
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
    FunctionData<NT> function_data = {
        .function_selector = 11,
        .is_private = false,
        .is_constructor = false,
    };

    info("function data: ", function_data);

    auto buffer = to_buffer(function_data);
    auto function_data_2 = from_buffer<FunctionData<NT>>(buffer.data());

    EXPECT_EQ(function_data, function_data_2);
}

// TEST(abi_tests, native_read_write_previous_kernel_data)
// {
//     private_kernel::PreviousKernelData<NT> previous_kernel_data = {
//         .public_inputs = private_kernel::PublicInputs<NT>(),
//         .proof = NT::Proof(),
//         .vk = std::make_shared<NT::VK>(), // This won't work - you need to construct a vk from something, and we
//         don't have that "something" in this test. .vk_index = 0, .vk_path = { 0 },
//     };

//     info("previous_kernel_data: ", previous_kernel_data);

//     auto buffer = to_buffer(previous_kernel_data);
//     auto previous_kernel_data_2 = from_buffer<private_kernel::PreviousKernelData<NT>>(buffer.data());

//     EXPECT_EQ(previous_kernel_data, previous_kernel_data_2);
// }

TEST(abi_tests, native_to_circuit_function_data)
{
    FunctionData<NT> native_function_data = {
        .function_selector = 11,
        .is_private = false,
        .is_constructor = false,
    };

    info("function data: ", native_function_data);

    Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
    FunctionData<CT> circuit_function_data = native_function_data.to_circuit_type(composer);

    info("function data: ", circuit_function_data);
}

TEST(abi_tests, native_call_context)
{
    CallContext<NT> call_context = {
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
    CallContext<NT> native_call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
        .portal_contract_address = 12,
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call context: ", native_call_context);

    Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
    CallContext<CT> circuit_call_context = native_call_context.to_circuit_type(composer);

    info("call context: ", circuit_call_context);
}

} // namespace aztec3::circuits::abis