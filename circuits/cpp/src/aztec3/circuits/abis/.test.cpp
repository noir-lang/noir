#include <gtest/gtest.h>
#include <barretenberg/common/test.hpp>
#include <barretenberg/common/serialize.hpp>
#include "index.hpp"
#include <barretenberg/stdlib/types/types.hpp>

namespace aztec3::circuits::abis {

using Composer = plonk::stdlib::types::Composer;
using CT = aztec3::utils::types::CircuitTypes<Composer>;
using NT = aztec3::utils::types::NativeTypes;

class abi_tests : public ::testing::Test {};

TEST(abi_tests, test_native_function_data)
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

TEST(abi_tests, test_native_to_circuit_function_data)
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

TEST(abi_tests, test_native_call_context)
{
    CallContext<NT> call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
        .tx_origin = 12,
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call context: ", call_context);
}

TEST(abi_tests, test_native_to_circuit_call_context)
{
    CallContext<NT> native_call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
        .tx_origin = 12,
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call context: ", native_call_context);

    Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
    CallContext<CT> circuit_call_context = native_call_context.to_circuit_type(composer);

    info("call context: ", circuit_call_context);
}

// TEST(abi_tests, test_native_public_inputs)
// {
//     PublicCircuitPublicInputs<NT> public_inputs = {
//         .args = { 1, 2, 3, 4, 5, 6, 7, 8 },
//         .return_values = { 9, 10, 11, 12 },
//         .emitted_events = { 13, 14, 15, 16 },
//         .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
//         .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
//         .public_call_stack = { 26, 27, 28, 29 },
//         .contract_deployment_call_stack = { 30, 31 },
//         .partial_l1_call_stack = { 32, 33 },
//         .historic_private_data_tree_root = 38,
//     };

//     info("public_circuit_public_inputs: ", public_inputs);
// }

// TEST(abi_tests, test_native_to_circuit_public_circuit_public_inputs)
// {
//     PublicCircuitPublicInputs<NT> native_public_inputs = {
//         .args = { 1, 2, 3, 4, 5, 6, 7, 8 },
//         .return_values = { 9, 10, 11, 12 },
//         .emitted_events = { 13, 14, 15, 16 },
//         .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
//         .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
//         .public_call_stack = { 26, 27, 28, 29 },
//         .contract_deployment_call_stack = { 30, 31 },
//         .partial_l1_call_stack = { 32, 33 },
//         .historic_private_data_tree_root = 38,
//     };

//     info("public_circuit_public_inputs: ", native_public_inputs);

//     Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
//     PublicCircuitPublicInputs<CT> circuit_public_inputs = native_public_inputs.to_circuit_type(composer);

//     info("public_circuit_public_inputs: ", circuit_public_inputs);
// }

// TEST(abi_tests, test_native_call_stack_item)
// {
//     PublicCircuitPublicInputs<NT> public_inputs = {
//         .args = { 1, 2, 3, 4, 5, 6, 7, 8 },
//         .return_values = { 9, 10, 11, 12 },
//         .emitted_events = { 13, 14, 15, 16 },
//         .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
//         .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
//         .public_call_stack = { 26, 27, 28, 29 },
//         .contract_deployment_call_stack = { 30, 31 },
//         .partial_l1_call_stack = { 32, 33 },
//         .historic_private_data_tree_root = 38,
//     };

//     CallStackItem<NT, CallType::Public> call_stack_item = {
//         .function_data = {
//             // .contract_address = 10,
//             .function_selector = 11,
//             .is_private = false,
//             .is_constructor = false,
//         },
//         .public_inputs = public_inputs,
//         .call_context = {
//             .msg_sender = 13,
//             .storage_contract_address = 14,
//         },
//         .is_delegate_call = false,
//         .is_static_call = false,
//     };

//     info("call stack item: ", call_stack_item);
// }

// TEST(abi_tests, test_native_to_circuit_call_stack_item)
// {
//     PublicCircuitPublicInputs<NT> public_inputs = {
//         .args = { 1, 2, 3, 4, 5, 6, 7, 8 },
//         .return_values = { 9, 10, 11, 12 },
//         .emitted_events = { 13, 14, 15, 16 },
//         .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
//         .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
//         .public_call_stack = { 26, 27, 28, 29 },
//         .contract_deployment_call_stack = { 30, 31 },
//         .partial_l1_call_stack = { 32, 33 },
//         .historic_private_data_tree_root = 38,
//     };

//     CallStackItem<NT, CallType::Public> native_call_stack_item = {
//         .function_data = {
//             // .contract_address = 10,
//             .function_selector = 11,
//             .is_private = false,
//             .is_constructor = false,
//         },
//         .public_inputs = public_inputs,
//         .call_context = {
//             .msg_sender = 13,
//             .storage_contract_address = 14,
//         },
//         .is_delegate_call = false,
//         .is_static_call = false,
//     };

//     info("call stack item: ", native_call_stack_item);

//     Composer composer = Composer("../barretenberg/cpp/srs_db/ignition");
//     CallStackItem<CT, CallType::Public> circuit_call_stack_item =
//         native_call_stack_item.to_circuit_type(composer);

//     info("call stack item: ", circuit_call_stack_item);
// }

} // namespace aztec3::circuits::abis