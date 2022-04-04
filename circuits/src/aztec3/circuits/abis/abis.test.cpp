#include <gtest/gtest.h>
#include <common/test.hpp>
#include <common/serialize.hpp>
// #include <numeric/random/engine.hpp>
#include "index.hpp"
#include <stdlib/types/turbo.hpp>

namespace aztec3::circuits::abis {

using TurboComposer = plonk::stdlib::types::turbo::Composer;
using CT = plonk::stdlib::types::CircuitTypes<TurboComposer>;
using NT = plonk::stdlib::types::NativeTypes;

class abi_tests : public ::testing::Test {};

TEST(abi_tests, test_native_function_signature)
{
    FunctionSignature<NT> function_signature = {
        .contract_address = 10,
        .vk_index = 11,
        .is_private = false,
        .is_constructor = false,
        .is_callback = false,
    };

    info("function signature: ", function_signature);

    auto buffer = to_buffer(function_signature);
    auto function_signature_2 = from_buffer<FunctionSignature<NT>>(buffer.data());

    EXPECT_EQ(function_signature, function_signature_2);
}

TEST(abi_tests, test_native_to_circuit_function_signature)
{
    FunctionSignature<NT> native_function_signature = {
        .contract_address = 10,
        .vk_index = 11,
        .is_private = false,
        .is_constructor = false,
        .is_callback = false,
    };

    info("function signature: ", native_function_signature);

    TurboComposer turbo_composer;
    FunctionSignature<CT> circuit_function_signature = native_function_signature.to_circuit_type(turbo_composer);

    info("function signature: ", circuit_function_signature);
}

TEST(abi_tests, test_native_call_context)
{
    CallContext<NT> call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
    };

    info("call context: ", call_context);
}

TEST(abi_tests, test_native_to_circuit_call_context)
{
    CallContext<NT> native_call_context = {
        .msg_sender = 10,
        .storage_contract_address = 11,
    };

    info("call context: ", native_call_context);

    TurboComposer turbo_composer;
    CallContext<CT> circuit_call_context = native_call_context.to_circuit_type(turbo_composer);

    info("call context: ", circuit_call_context);
}

TEST(abi_tests, test_native_public_inputs)
{
    PublicCircuitPublicInputs<NT> public_inputs = {
        .custom_public_inputs = { 1, 2, 3, 4, 5, 6, 7, 8 },
        .custom_outputs = { 9, 10, 11, 12 },
        .emitted_public_inputs = { 13, 14, 15, 16 },
        .emitted_outputs = { 17, 18, 19, 20 },
        .executed_callback = ExecutedCallback<NT>::empty(),
        .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
        .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
        .public_call_stack = { 26, 27, 28, 29 },
        .contract_deployment_call_stack = { 30, 31 },
        .partial_l1_call_stack = { 32, 33 },
        .callback_stack = { { { 34, 35, 36, 37 }, { 34, 35, 36, 37 } } },
        .old_private_data_tree_root = 38,
    };

    info("public_circuit_public_inputs: ", public_inputs);
}

TEST(abi_tests, test_native_to_circuit_public_circuit_public_inputs)
{
    PublicCircuitPublicInputs<NT> native_public_inputs = {
        .custom_public_inputs = { 1, 2, 3, 4, 5, 6, 7, 8 },
        .custom_outputs = { 9, 10, 11, 12 },
        .emitted_public_inputs = { 13, 14, 15, 16 },
        .emitted_outputs = { 17, 18, 19, 20 },
        .executed_callback = ExecutedCallback<NT>::empty(),
        .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
        .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
        .public_call_stack = { 26, 27, 28, 29 },
        .contract_deployment_call_stack = { 30, 31 },
        .partial_l1_call_stack = { 32, 33 },
        .callback_stack = { { { 34, 35, 36, 37 }, { 34, 35, 36, 37 } } },
        .old_private_data_tree_root = 38,
    };

    info("public_circuit_public_inputs: ", native_public_inputs);

    TurboComposer turbo_composer;
    PublicCircuitPublicInputs<CT> circuit_public_inputs = native_public_inputs.to_circuit_type(turbo_composer);

    info("public_circuit_public_inputs: ", circuit_public_inputs);
}

TEST(abi_tests, test_native_call_stack_item)
{
    PublicCircuitPublicInputs<NT> public_inputs = {
        .custom_public_inputs = { 1, 2, 3, 4, 5, 6, 7, 8 },
        .custom_outputs = { 9, 10, 11, 12 },
        .emitted_public_inputs = { 13, 14, 15, 16 },
        .emitted_outputs = { 17, 18, 19, 20 },
        .executed_callback = ExecutedCallback<NT>::empty(),
        .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
        .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
        .public_call_stack = { 26, 27, 28, 29 },
        .contract_deployment_call_stack = { 30, 31 },
        .partial_l1_call_stack = { 32, 33 },
        .callback_stack = { { { 34, 35, 36, 37 }, { 34, 35, 36, 37 } } },
        .old_private_data_tree_root = 38,
    };

    CallStackItem<NT, CallType::Public> call_stack_item = {
        .function_signature = {
            .contract_address = 10,
            .vk_index = 11,
            .is_private = false,
            .is_constructor = false,
            .is_callback = false,
        },
        .public_inputs = public_inputs,
        .call_context = {
            .msg_sender = 13,
            .storage_contract_address = 14,
        },
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call stack item: ", call_stack_item);
}

TEST(abi_tests, test_native_to_circuit_call_stack_item)
{
    PublicCircuitPublicInputs<NT> public_inputs = {
        .custom_public_inputs = { 1, 2, 3, 4, 5, 6, 7, 8 },
        .custom_outputs = { 9, 10, 11, 12 },
        .emitted_public_inputs = { 13, 14, 15, 16 },
        .emitted_outputs = { 17, 18, 19, 20 },
        .executed_callback = ExecutedCallback<NT>::empty(),
        .state_transitions = { { { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 }, { 21, 22, 23 } } },
        .state_reads = { { { 24, 25 }, { 24, 25 }, { 24, 25 }, { 24, 25 } } },
        .public_call_stack = { 26, 27, 28, 29 },
        .contract_deployment_call_stack = { 30, 31 },
        .partial_l1_call_stack = { 32, 33 },
        .callback_stack = { { { 34, 35, 36, 37 }, { 34, 35, 36, 37 } } },
        .old_private_data_tree_root = 38,
    };

    CallStackItem<NT, CallType::Public> native_call_stack_item = {
        .function_signature = {
            .contract_address = 10,
            .vk_index = 11,
            .is_private = false,
            .is_constructor = false,
            .is_callback = false,
        },
        .public_inputs = public_inputs,
        .call_context = {
            .msg_sender = 13,
            .storage_contract_address = 14,
        },
        .is_delegate_call = false,
        .is_static_call = false,
    };

    info("call stack item: ", native_call_stack_item);

    TurboComposer turbo_composer;
    CallStackItem<CT, CallType::Public> circuit_call_stack_item =
        native_call_stack_item.to_circuit_type(turbo_composer);

    info("call stack item: ", circuit_call_stack_item);
}

TEST(abi_tests, test_native_callback_stack_item)
{
    CallbackStackItem<NT> callback_stack_item = {
        .callback_public_key = 1,
        .success_callback_call_hash = 2,
        .failure_callback_call_hash = 3,
        .success_result_arg_map_acc = 4,
    };

    info("callback stack item: ", callback_stack_item);
}

TEST(abi_tests, test_native_to_circuit_callback_stack_item)
{
    CallbackStackItem<NT> native_callback_stack_item = {
        .callback_public_key = 1,
        .success_callback_call_hash = 2,
        .failure_callback_call_hash = 3,
        .success_result_arg_map_acc = 4,
    };

    info("callback stack item: ", native_callback_stack_item);

    TurboComposer turbo_composer;
    CallbackStackItem<CT> circuit_callback_stack_item = native_callback_stack_item.to_circuit_type(turbo_composer);

    info("callback stack item: ", circuit_callback_stack_item);
}

TEST(abi_tests, test_native_executed_callback)
{
    ExecutedCallback<NT> executed_callback = {
        .l1_result_hash = 1,
        .l1_results_tree_leaf_index = 2,
    };

    info("executed callback: ", executed_callback);
}

TEST(abi_tests, test_native_to_circuit_executed_callback)
{
    ExecutedCallback<NT> native_executed_callback = {
        .l1_result_hash = 1,
        .l1_results_tree_leaf_index = 2,
    };

    info("executed callback: ", native_executed_callback);

    TurboComposer turbo_composer;
    ExecutedCallback<CT> circuit_executed_callback = native_executed_callback.to_circuit_type(turbo_composer);

    info("executed callback: ", circuit_executed_callback);
}

} // namespace aztec3::circuits::abis