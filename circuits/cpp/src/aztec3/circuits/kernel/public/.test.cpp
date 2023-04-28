#include "init.hpp"
#include "native_public_kernel_circuit_no_previous_kernel.hpp"
#include "native_public_kernel_circuit_public_previous_kernel.hpp"
#include "native_public_kernel_circuit_private_previous_kernel.hpp"
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/public_kernel/public_call_data.hpp>
#include <gtest/gtest.h>

#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/call_stack_item.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>
#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/circuits/abis/signed_tx_request.hpp>
#include <aztec3/circuits/abis/tx_context.hpp>
#include <aztec3/circuits/abis/tx_request.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/combined_accumulated_data.hpp>
#include <aztec3/circuits/abis/combined_constant_data.hpp>
#include <aztec3/circuits/abis/private_historic_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>
#include <aztec3/circuits/abis/types.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>

namespace {
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::PublicCircuitPublicInputs;
using aztec3::circuits::abis::PublicDataRead;
using aztec3::circuits::abis::PublicTypes;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::public_kernel::PublicCallData;
using aztec3::circuits::apps::FunctionExecutionContext;
} // namespace

namespace aztec3::circuits::kernel::public_kernel {

typedef CallStackItem<NT, PublicTypes> PublicCallStackItem;

template <size_t SIZE> std::array<NT::fr, SIZE> array_of_values(NT::uint32& count)
{
    std::array<NT::fr, SIZE> values;
    for (size_t i = 0; i < SIZE; i++) {
        values[i] = ++count;
    }
    return values;
}

std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH> generate_state_transitions(NT::uint32& count)
{
    std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH> values;
    for (size_t i = 0; i < STATE_TRANSITIONS_LENGTH; i++) {
        const auto prev = count++;
        values[i] = StateTransition<NT>{
            .storage_slot = prev,
            .old_value = prev,
            .new_value = count,
        };
    };
    return values;
}

std::array<StateRead<NT>, STATE_READS_LENGTH> generate_state_reads(NT::uint32& count)
{
    std::array<StateRead<NT>, STATE_READS_LENGTH> values;
    for (size_t i = 0; i < STATE_READS_LENGTH; i++) {
        const auto prev = count++;
        values[i] = StateRead<NT>{
            .storage_slot = prev,
            .current_value = prev,
        };
    };
    return values;
}

PublicCallStackItem generate_call_stack_item(NT::fr contract_address, NT::fr msg_sender, NT::uint32 seed = 0)
{
    NT::uint32 count = seed + 1;
    FunctionData<NT> function_data{
        .function_selector = count,
        .is_private = false,
        .is_constructor = false,
    };
    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = msg_sender,
        .portal_contract_address = ++count,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };
    std::array<NT::fr, ARGS_LENGTH> args = array_of_values<ARGS_LENGTH>(count);
    std::array<NT::fr, RETURN_VALUES_LENGTH> return_values = array_of_values<RETURN_VALUES_LENGTH>(count);
    std::array<NT::fr, EMITTED_EVENTS_LENGTH> emitted_events = array_of_values<EMITTED_EVENTS_LENGTH>(count);
    std::array<NT::fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack = array_of_values<PUBLIC_CALL_STACK_LENGTH>(count);
    std::array<NT::fr, L1_MSG_STACK_LENGTH> l1_msg_stack = array_of_values<L1_MSG_STACK_LENGTH>(count);
    std::array<StateRead<NT>, STATE_READS_LENGTH> reads = generate_state_reads(count);
    std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH> transitions = generate_state_transitions(count);

    // create the public circuit public inputs
    PublicCircuitPublicInputs<NT> public_circuit_public_inputs = PublicCircuitPublicInputs<NT>{
        .call_context = call_context,
        .args = args,
        .return_values = return_values,
        .emitted_events = emitted_events,
        .state_transitions = transitions,
        .state_reads = reads,
        .public_call_stack = public_call_stack,
        .l1_msg_stack = l1_msg_stack,

    };
    PublicCallStackItem call_stack_item = PublicCallStackItem{
        .contract_address = contract_address,
        .function_data = function_data,
        .public_inputs = public_circuit_public_inputs,
    };
    return call_stack_item;
}

/**
 * @brief Generates the inputs to the public kernel circuit
 *
 * @param is_constructor whether this public circuit call is a constructor
 * @param args_vec the private call's args
 * @return PrivateInputs<NT> - the inputs to the private call circuit
 */
PublicKernelInputsNoPreviousKernel<NT> do_public_call_get_kernel_inputs_no_previous_kernel(
    bool const is_constructor, std::vector<NT::fr> const& args_vec)
{
    NT::address contract_address = is_constructor ? 0 : 12345;
    const NT::fr portal_contract_address = 23456;

    const NT::address msg_sender = NT::fr(1);
    const NT::address tx_origin = msg_sender;

    FunctionData<NT> function_data{
        .function_selector = 1,
        .is_private = false,
        .is_constructor = is_constructor,
    };

    CallContext<NT> call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .portal_contract_address = portal_contract_address,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = is_constructor,
    };

    // sometimes need public call args as array
    std::array<NT::fr, ARGS_LENGTH> args{};
    for (size_t i = 0; i < args_vec.size(); ++i) {
        args[i] = args_vec[i];
    }

    //***************************************************************************
    // We can create a TxRequest from some of the above data. Users must sign a TxRequest in order to give permission
    // for a tx to take place - creating a SignedTxRequest.
    //***************************************************************************
    TxRequest<NT> tx_request = TxRequest<NT>{
        .from = tx_origin,
        .to = contract_address,
        .function_data = function_data,
        .args = args,
        .nonce = 0,
        .tx_context =
            TxContext<NT>{
                .is_fee_payment_tx = false,
                .is_rebate_payment_tx = false,
                .is_contract_deployment_tx = is_constructor,
                .contract_deployment_data = {},
            },
        .chain_id = 1,
    };

    SignedTxRequest<NT> signed_tx_request = SignedTxRequest<NT>{
        .tx_request = tx_request,

        //.signature = TODO: need a method for signing a TxRequest.
    };

    std::array<PublicCallStackItem, PUBLIC_CALL_STACK_LENGTH> child_call_stacks;
    NT::uint32 seed = 1000;
    NT::fr child_contract_address = 100000;
    NT::fr child_msg_sender = contract_address;
    std::array<NT::fr, PUBLIC_CALL_STACK_LENGTH> call_stack_hashes;
    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        child_call_stacks[i] = generate_call_stack_item(child_contract_address, child_msg_sender, seed);
        call_stack_hashes[i] = child_call_stacks[i].hash();
        child_msg_sender = child_contract_address;
        child_contract_address++;
    }

    std::array<fr, RETURN_VALUES_LENGTH> return_values = array_of_values<RETURN_VALUES_LENGTH>(seed);
    std::array<fr, EMITTED_EVENTS_LENGTH> emitted_events = array_of_values<EMITTED_EVENTS_LENGTH>(seed);
    std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH> state_transitions = generate_state_transitions(seed);
    std::array<StateRead<NT>, STATE_READS_LENGTH> state_reads = generate_state_reads(seed);
    std::array<fr, L1_MSG_STACK_LENGTH> l1_msg_stack = array_of_values<L1_MSG_STACK_LENGTH>(seed);
    fr historic_public_data_tree_root = ++seed;

    // create the public circuit public inputs
    PublicCircuitPublicInputs<NT> public_circuit_public_inputs = PublicCircuitPublicInputs<NT>{
        .call_context = call_context,
        .args = args,
        .return_values = return_values,
        .emitted_events = emitted_events,
        .state_transitions = state_transitions,
        .state_reads = state_reads,
        .public_call_stack = call_stack_hashes,
        .l1_msg_stack = l1_msg_stack,
        .historic_public_data_tree_root = historic_public_data_tree_root,
    };

    const PublicCallStackItem call_stack_item{
        .contract_address = tx_request.to,
        .function_data = tx_request.function_data,
        .public_inputs = public_circuit_public_inputs,
    };

    PublicCallData<NT> public_call_data = {
        .call_stack_item = call_stack_item,
        .public_call_stack_preimages = child_call_stacks,
        .bytecode_hash = 1234567,
    };

    //***************************************************************************
    // Now we can construct the full inputs to the kernel circuit
    //***************************************************************************
    PublicKernelInputsNoPreviousKernel<NT> public_kernel_inputs = {
        .signed_tx_request = signed_tx_request,
        .public_call = { public_call_data },
    };

    return public_kernel_inputs;
}

TEST(public_kernel_tests, no_previous_kernel_public_call_should_succeed)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_FALSE(dc.failed());
}

TEST(public_kernel_tests, circuit_outputs_should_be_correctly_populated)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_FALSE(dc.failed());

    ASSERT_FALSE(public_inputs.is_private);
    ASSERT_EQ(public_inputs.constants.tx_context, inputs.signed_tx_request.tx_request.tx_context);

    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    for (size_t i = 0; i < STATE_TRANSITIONS_LENGTH; i++) {
        const auto& st = inputs.public_call.call_stack_item.public_inputs.state_transitions[i];
        const auto public_write = PublicDataTransition<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, st.storage_slot),
            .new_value = compute_public_data_tree_value<NT>(st.new_value),
        };
        ASSERT_EQ(public_inputs.end.state_transitions[i], public_write);
    }

    for (size_t i = 0; i < STATE_READS_LENGTH; i++) {
        const auto& st = inputs.public_call.call_stack_item.public_inputs.state_reads[i];
        const auto public_read = PublicDataRead<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, st.storage_slot),
            .value = compute_public_data_tree_value<NT>(st.current_value),
        };
        ASSERT_EQ(public_inputs.end.state_reads[i], public_read);
    }
}

TEST(public_kernel_tests, only_valid_state_reads_should_be_propagated)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    // modify the state reads so only 2 are valid and only those should be propagated
    const auto first_valid = StateRead<NT>{
        .storage_slot = 123456789,
        .current_value = 76543,
    };
    const auto second_valid = StateRead<NT>{
        .storage_slot = 123456789,
        .current_value = 76543,
    };
    std::array<StateRead<NT>, STATE_TRANSITIONS_LENGTH> reads = std::array<StateRead<NT>, STATE_TRANSITIONS_LENGTH>();
    reads[1] = first_valid;
    reads[3] = second_valid;
    inputs.public_call.call_stack_item.public_inputs.state_reads = reads;

    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_FALSE(dc.failed());

    ASSERT_FALSE(public_inputs.is_private);
    ASSERT_EQ(public_inputs.constants.tx_context, inputs.signed_tx_request.tx_request.tx_context);

    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    // only the 2 valid reads should have been propagated
    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    const auto public_read_1 = PublicDataRead<NT>{
        .leaf_index = compute_public_data_tree_index<NT>(contract_address, first_valid.storage_slot),
        .value = compute_public_data_tree_value<NT>(first_valid.current_value),
    };
    const auto public_read_2 = PublicDataRead<NT>{
        .leaf_index = compute_public_data_tree_index<NT>(contract_address, second_valid.storage_slot),
        .value = compute_public_data_tree_value<NT>(second_valid.current_value),
    };
    ASSERT_EQ(public_inputs.end.state_reads[0], public_read_1);
    ASSERT_EQ(public_inputs.end.state_reads[1], public_read_2);
}

TEST(public_kernel_tests, only_valid_state_transitions_should_be_propagated)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    // modify the state transitions so only 2 are valid and only those should be propagated
    const auto first_valid = StateTransition<NT>{
        .storage_slot = 123456789,
        .old_value = 76543,
        .new_value = 76544,
    };
    const auto second_valid = StateTransition<NT>{
        .storage_slot = 123456789,
        .old_value = 76543,
        .new_value = 76544,
    };
    std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH> transitions =
        std::array<StateTransition<NT>, STATE_TRANSITIONS_LENGTH>();
    transitions[1] = first_valid;
    transitions[3] = second_valid;
    inputs.public_call.call_stack_item.public_inputs.state_transitions = transitions;

    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_FALSE(dc.failed());

    ASSERT_FALSE(public_inputs.is_private);
    ASSERT_EQ(public_inputs.constants.tx_context, inputs.signed_tx_request.tx_request.tx_context);

    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    // only the 2 valid transitions should have been propagated
    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    const auto public_write_1 = PublicDataTransition<NT>{
        .leaf_index = compute_public_data_tree_index<NT>(contract_address, first_valid.storage_slot),
        .new_value = compute_public_data_tree_value<NT>(first_valid.new_value),
    };
    const auto public_write_2 = PublicDataTransition<NT>{
        .leaf_index = compute_public_data_tree_index<NT>(contract_address, second_valid.storage_slot),
        .new_value = compute_public_data_tree_value<NT>(second_valid.new_value),
    };
    ASSERT_EQ(public_inputs.end.state_transitions[0], public_write_1);
    ASSERT_EQ(public_inputs.end.state_transitions[1], public_write_2);
}

TEST(public_kernel_tests, constructor_should_fail)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.function_data.is_constructor = true;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, constructor_should_fail_2)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.public_inputs.call_context.is_contract_deployment = true;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, no_bytecode_hash_should_fail)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.bytecode_hash = 0;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, delegate_call_should_fail)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.public_inputs.call_context.is_delegate_call = true;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, static_call_should_fail)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.public_inputs.call_context.is_static_call = true;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, storage_contract_address_must_equal_contract_address)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    const NT::fr contract_address = inputs.public_call.call_stack_item.contract_address;
    inputs.public_call.call_stack_item.public_inputs.call_context.storage_contract_address = contract_address + 1;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, contract_address_must_be_valid)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.contract_address = 0;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, function_selector_must_be_valid)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.function_data.function_selector = 0;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, private_call_should_fail)
{
    DummyComposer dc;
    PublicKernelInputsNoPreviousKernel<NT> inputs =
        do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

    inputs.public_call.call_stack_item.function_data.is_private = true;
    auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
    ASSERT_TRUE(dc.failed());
}

TEST(public_kernel_tests, inconsistent_call_hash_should_fail)
{
    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        DummyComposer dc;
        PublicKernelInputsNoPreviousKernel<NT> inputs =
            do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

        // change a value of something in the call stack pre-image
        inputs.public_call.public_call_stack_preimages[i].public_inputs.args[0]++;
        auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
        ASSERT_TRUE(dc.failed());
    }
}

TEST(public_kernel_tests, incorrect_storage_contract_address_fails_for_regular_calls)
{
    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        DummyComposer dc;
        PublicKernelInputsNoPreviousKernel<NT> inputs =
            do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });

        // change the storage contract address so it does not equal the contract address
        const NT::fr new_contract_address =
            NT::fr(inputs.public_call.public_call_stack_preimages[i].contract_address) + 1;
        inputs.public_call.public_call_stack_preimages[i].public_inputs.call_context.storage_contract_address =
            new_contract_address;
        auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
        ASSERT_TRUE(dc.failed());
    }
}

TEST(public_kernel_tests, incorrect_msg_sender_fails_for_regular_calls)
{
    for (size_t i = 0; i < PUBLIC_CALL_STACK_LENGTH; i++) {
        DummyComposer dc;
        PublicKernelInputsNoPreviousKernel<NT> inputs =
            do_public_call_get_kernel_inputs_no_previous_kernel(false, std::vector<NT::fr>{ 1 });
        const auto msg_sender = i == 0 ? inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender
                                       : inputs.public_call.public_call_stack_preimages[i - 1].contract_address;
        const NT::fr new_msg_sender = NT::fr(msg_sender) + 1;
        // change the storage contract address so it does not equal the contract address
        inputs.public_call.public_call_stack_preimages[i].public_inputs.call_context.storage_contract_address =
            new_msg_sender;
        auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
        ASSERT_TRUE(dc.failed());
    }
}

// TEST(public_kernel_tests, public_previous_kernel)
// {
//     DummyComposer dc;
//     PublicKernelInputs<NT> inputs;
//     auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dc, inputs);
//     ASSERT_TRUE(dc.failed());
// }

// TEST(public_kernel_tests, private_previous_kernel)
// {
//     DummyComposer dc;
//     PublicKernelInputs<NT> inputs;
//     auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dc, inputs);
//     ASSERT_TRUE(dc.failed());
// }
} // namespace aztec3::circuits::kernel::public_kernel