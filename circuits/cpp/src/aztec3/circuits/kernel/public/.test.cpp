#include "init.hpp"
#include "native_public_kernel_circuit_private_previous_kernel.hpp"
#include "native_public_kernel_circuit_public_previous_kernel.hpp"

#include "aztec3/circuits/abis/call_context.hpp"
#include "aztec3/circuits/abis/call_stack_item.hpp"
#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/combined_constant_data.hpp"
#include "aztec3/circuits/abis/contract_storage_update_request.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/public_kernel/public_call_data.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp"
#include "aztec3/circuits/abis/tx_context.hpp"
#include "aztec3/circuits/abis/tx_request.hpp"
#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <gtest/gtest.h>

#include <array>

namespace aztec3::circuits::kernel::public_kernel {
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CombinedAccumulatedData;
using aztec3::circuits::abis::CombinedConstantData;
using aztec3::circuits::abis::ContractStorageRead;
using aztec3::circuits::abis::ContractStorageUpdateRequest;
using aztec3::circuits::abis::HistoricBlockData;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::OptionallyRevealedData;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::PublicCircuitPublicInputs;
using aztec3::circuits::abis::PublicDataRead;
using aztec3::circuits::abis::PublicTypes;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::public_kernel::PublicCallData;
using aztec3::utils::source_arrays_are_in_target;
using PublicCallStackItem = CallStackItem<NT, aztec3::circuits::abis::PublicTypes>;

template <size_t SIZE>
std::array<NT::fr, SIZE> array_of_values(NT::uint32& count, NT::uint32 num_values_required = SIZE)
{
    ASSERT(num_values_required <= SIZE);
    std::array<NT::fr, SIZE> values{};
    for (size_t i = 0; i < num_values_required; i++) {
        values[i] = ++count;
    }
    return values;
}

template <typename T, size_t SIZE> std::array<T, SIZE> empty_array_of_values()
{
    std::array<T, SIZE> values{};
    return values;
}

std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>
generate_contract_storage_update_requests(NT::uint32& count,
                                          NT::uint32 num_values_required = MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL)
{
    std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> values;
    for (size_t i = 0; i < num_values_required; i++) {
        const auto prev = count++;
        values[i] = ContractStorageUpdateRequest<NT>{
            .storage_slot = prev,
            .old_value = prev,
            .new_value = count,
        };
    };
    return values;
}

std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> generate_contract_storage_reads(
    NT::uint32& count, NT::uint32 num_values_required = MAX_PUBLIC_DATA_READS_PER_CALL)
{
    std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> values;
    for (size_t i = 0; i < num_values_required; i++) {
        const auto prev = count++;
        values[i] = ContractStorageRead<NT>{
            .storage_slot = prev,
            .current_value = prev,
        };
    };
    return values;
}

PublicCallStackItem generate_call_stack_item(NT::fr contract_address,
                                             NT::fr msg_sender,
                                             NT::fr storage_contract_address,
                                             NT::fr portal_contract_address,
                                             NT::boolean is_delegate_call,
                                             NT::uint32 seed = 0)
{
    NT::uint32 count = seed + 1;
    FunctionData<NT> const function_data{
        .selector =
            {
                .value = count,
            },
        .is_private = false,
        .is_constructor = false,
    };
    CallContext<NT> const call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = storage_contract_address,
        .portal_contract_address = portal_contract_address,
        .is_delegate_call = is_delegate_call,
        .is_static_call = false,
        .is_contract_deployment = false,
    };
    fr const args_hash = count;
    std::array<NT::fr, RETURN_VALUES_LENGTH> const return_values = array_of_values<RETURN_VALUES_LENGTH>(count);
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> const public_call_stack =
        array_of_values<MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>(count);
    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_CALL> const new_commitments =
        array_of_values<MAX_NEW_COMMITMENTS_PER_CALL>(count);
    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_CALL> const new_nullifiers =
        array_of_values<MAX_NEW_NULLIFIERS_PER_CALL>(count);
    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> const new_l2_to_l1_msgs =
        array_of_values<MAX_NEW_L2_TO_L1_MSGS_PER_CALL>(count);
    std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> const reads =
        generate_contract_storage_reads(count);
    std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> const update_requests =
        generate_contract_storage_update_requests(count);

    // create the public circuit public inputs
    auto const public_circuit_public_inputs = PublicCircuitPublicInputs<NT>{
        .call_context = call_context,
        .args_hash = args_hash,
        .return_values = return_values,
        .contract_storage_update_requests = update_requests,
        .contract_storage_reads = reads,
        .public_call_stack = public_call_stack,
        .new_commitments = new_commitments,
        .new_nullifiers = new_nullifiers,
        .new_l2_to_l1_msgs = new_l2_to_l1_msgs,

    };
    auto call_stack_item = PublicCallStackItem{
        .contract_address = contract_address,
        .function_data = function_data,
        .public_inputs = public_circuit_public_inputs,
    };
    return call_stack_item;
}

PublicDataRead<NT> public_data_read_from_contract_storage_read(ContractStorageRead<NT> const& contract_storage_read,
                                                               NT::fr const& contract_address)
{
    return PublicDataRead<NT>{
        .leaf_index = compute_public_data_tree_index<NT>(contract_address, contract_storage_read.storage_slot),
        .value = compute_public_data_tree_value<NT>(contract_storage_read.current_value),
    };
}

PublicDataUpdateRequest<NT> public_data_update_request_from_contract_storage_update_request(
    ContractStorageUpdateRequest<NT> const& contract_storage_update_request, NT::fr const& contract_address)
{
    return PublicDataUpdateRequest<NT>{
        .leaf_index =
            compute_public_data_tree_index<NT>(contract_address, contract_storage_update_request.storage_slot),
        .old_value = compute_public_data_tree_value<NT>(contract_storage_update_request.old_value),
        .new_value = compute_public_data_tree_value<NT>(contract_storage_update_request.new_value),
    };
}

std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> public_data_reads_from_contract_storage_reads(
    std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> const& public_data_reads,
    NT::fr const& contract_address)
{
    std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> values;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_READS_PER_CALL; i++) {
        const auto& read = public_data_reads[i];
        if (read.is_empty()) {
            continue;
        }
        values[i] = public_data_read_from_contract_storage_read(read, contract_address);
    }
    return values;
}

std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>
public_data_update_requests_from_contract_storage_update_requests(
    std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> const& update_requests,
    NT::fr const& contract_address)
{
    std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> values;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; i++) {
        const auto& update_request = update_requests[i];
        if (update_request.is_empty()) {
            continue;
        }
        values[i] = public_data_update_request_from_contract_storage_update_request(update_request, contract_address);
    }
    return values;
}

std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> new_commitments_as_siloed_commitments(
    std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> const& new_commitments, NT::fr const& contract_address)
{
    std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> siloed_commitments{};
    for (size_t i = 0; i < MAX_NEW_COMMITMENTS_PER_CALL; ++i) {
        if (!new_commitments[i].is_zero()) {
            siloed_commitments[i] = silo_commitment<NT>(contract_address, new_commitments[i]);
        }
    }
    return siloed_commitments;
}

std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> new_nullifiers_as_siloed_nullifiers(
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> const& new_nullifiers, NT::fr const& contract_address)
{
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> siloed_nullifiers{};
    for (size_t i = 0; i < MAX_NEW_NULLIFIERS_PER_CALL; ++i) {
        if (!new_nullifiers[i].is_zero()) {
            siloed_nullifiers[i] = silo_nullifier<NT>(contract_address, new_nullifiers[i]);
        }
    }
    return siloed_nullifiers;
}

std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> new_l2_messages_from_message(
    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> const& new_messages,
    NT::fr const& contract_address,
    fr const& portal_contract_address,
    fr const& chain_id,
    fr const& version)
{
    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> formatted_msgs{};
    for (size_t i = 0; i < MAX_NEW_L2_TO_L1_MSGS_PER_CALL; ++i) {
        if (!new_messages[i].is_zero()) {
            formatted_msgs[i] = compute_l2_to_l1_hash<NT>(
                contract_address, version, portal_contract_address, chain_id, new_messages[i]);
        }
    }
    return formatted_msgs;
}

/**
 * @brief Generates the inputs to the public kernel circuit
 *
 * @param is_constructor whether this public circuit call is a constructor
 * @param args_vec the private call's args
 * @return PrivateInputs<NT> - the inputs to the private call circuit
 */
PublicKernelInputs<NT> get_kernel_inputs_with_previous_kernel(NT::boolean private_previous)
{
    NT::uint32 seed = 1000;

    NT::address contract_address = 12345;
    const NT::fr portal_contract_address = 23456;

    const NT::address msg_sender = NT::fr(1);

    FunctionData<NT> const function_data{
        .selector =
            {
                .value = 1,
            },
        .is_private = false,
        .is_constructor = false,
    };

    CallContext<NT> const call_context{
        .msg_sender = msg_sender,
        .storage_contract_address = contract_address,
        .portal_contract_address = portal_contract_address,
        .is_delegate_call = false,
        .is_static_call = false,
        .is_contract_deployment = false,
    };

    std::vector<NT::fr> const args = { 1, 2, 3 };

    //***************************************************************************
    // We can create a TxRequest from some of the above data.
    //***************************************************************************
    auto const tx_request = TxRequest<NT>{ .origin = contract_address,
                                           .function_data = function_data,
                                           .args_hash = compute_var_args_hash<NT>(args),
                                           .tx_context = TxContext<NT>{
                                               .is_fee_payment_tx = false,
                                               .is_rebate_payment_tx = false,
                                               .is_contract_deployment_tx = false,
                                               .contract_deployment_data = {},
                                               .chain_id = 1,
                                           } };

    std::array<PublicCallStackItem, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> child_call_stacks;
    NT::fr child_contract_address = 100000;
    NT::fr child_portal_contract_address = 200000;
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> call_stack_hashes{};
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        // NOLINTNEXTLINE(readability-suspicious-call-argument)
        child_call_stacks[i] = generate_call_stack_item(child_contract_address,
                                                        contract_address,
                                                        child_contract_address,
                                                        child_portal_contract_address,
                                                        false,
                                                        seed);
        call_stack_hashes[i] = child_call_stacks[i].hash();
        child_contract_address++;
        child_portal_contract_address++;
    }

    std::array<fr, RETURN_VALUES_LENGTH> const return_values =
        array_of_values<RETURN_VALUES_LENGTH>(seed, RETURN_VALUES_LENGTH / 2);
    std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> const update_requests =
        generate_contract_storage_update_requests(seed, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL / 2);
    std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> const reads =
        generate_contract_storage_reads(seed, MAX_PUBLIC_DATA_READS_PER_CALL / 2);
    std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> const new_commitments =
        array_of_values<MAX_NEW_COMMITMENTS_PER_CALL>(seed, MAX_NEW_COMMITMENTS_PER_CALL / 2);
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> const new_nullifiers =
        array_of_values<MAX_NEW_NULLIFIERS_PER_CALL>(seed, MAX_NEW_NULLIFIERS_PER_CALL / 2);
    std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> const new_l2_to_l1_msgs =
        array_of_values<MAX_NEW_L2_TO_L1_MSGS_PER_CALL>(seed, MAX_NEW_L2_TO_L1_MSGS_PER_CALL / 2);
    std::array<fr, NUM_FIELDS_PER_SHA256> const unencrypted_logs_hash =
        array_of_values<NUM_FIELDS_PER_SHA256>(seed, NUM_FIELDS_PER_SHA256);
    fr const unencrypted_log_preimages_length = ++seed;
    HistoricBlockData<NT> block_data = {
        .private_data_tree_root = ++seed,
        .nullifier_tree_root = ++seed,
        .contract_tree_root = ++seed,
        .l1_to_l2_messages_tree_root = ++seed,
        .blocks_tree_root = ++seed,
        .private_kernel_vk_tree_root = ++seed,
        .public_data_tree_root = ++seed,
        .global_variables_hash = ++seed,
    };

    // create the public circuit public inputs
    auto const public_circuit_public_inputs = PublicCircuitPublicInputs<NT>{
        .call_context = call_context,
        .args_hash = compute_var_args_hash<NT>(args),
        .return_values = return_values,
        .contract_storage_update_requests = update_requests,
        .contract_storage_reads = reads,
        .public_call_stack = call_stack_hashes,
        .new_commitments = new_commitments,
        .new_nullifiers = new_nullifiers,
        .new_l2_to_l1_msgs = new_l2_to_l1_msgs,
        .unencrypted_logs_hash = unencrypted_logs_hash,
        .unencrypted_log_preimages_length = unencrypted_log_preimages_length,
        .historic_block_data = block_data,
    };

    const PublicCallStackItem call_stack_item{
        .contract_address = contract_address,
        .function_data = tx_request.function_data,
        .public_inputs = public_circuit_public_inputs,
    };

    const PublicCallData<NT> public_call_data = {
        .call_stack_item = call_stack_item,
        .public_call_stack_preimages = child_call_stacks,
        .portal_contract_address = portal_contract_address,
        .bytecode_hash = 1234567,
    };

    // TODO(914) Should this be unused?
    [[maybe_unused]] HistoricBlockData<NT> const historic_tree_roots = {
        .private_data_tree_root = 1000,
        .contract_tree_root = 2000,
        .l1_to_l2_messages_tree_root = 3000,
        .private_kernel_vk_tree_root = 4000,
    };

    CombinedConstantData<NT> const end_constants = { .block_data =
                                                         HistoricBlockData<NT>{ .private_data_tree_root = ++seed,
                                                                                .nullifier_tree_root = ++seed,
                                                                                .contract_tree_root = ++seed,
                                                                                .private_kernel_vk_tree_root = ++seed },
                                                     .tx_context = TxContext<NT>{
                                                         .is_fee_payment_tx = false,
                                                         .is_rebate_payment_tx = false,
                                                         .is_contract_deployment_tx = false,
                                                         .contract_deployment_data = {},
                                                     } };

    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX> public_call_stack{};
    public_call_stack[0] = public_call_data.call_stack_item.hash();

    CombinedAccumulatedData<NT> const end_accumulated_data = {
        .new_commitments =
            array_of_values<MAX_NEW_COMMITMENTS_PER_TX>(seed, private_previous ? MAX_NEW_COMMITMENTS_PER_TX / 2 : 0),
        .new_nullifiers =
            array_of_values<MAX_NEW_NULLIFIERS_PER_TX>(seed, private_previous ? MAX_NEW_NULLIFIERS_PER_TX / 2 : 0),
        .private_call_stack = array_of_values<MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>(seed, 0),
        .public_call_stack = public_call_stack,
        .new_l2_to_l1_msgs = array_of_values<MAX_NEW_L2_TO_L1_MSGS_PER_TX>(
            seed, private_previous ? MAX_NEW_L2_TO_L1_MSGS_PER_TX / 2 : 0),
        .encrypted_logs_hash = array_of_values<NUM_FIELDS_PER_SHA256>(
            seed, private_previous ? 2 : 0),  // only private kernel is producing encrypted logs
        .unencrypted_logs_hash = array_of_values<NUM_FIELDS_PER_SHA256>(seed, NUM_FIELDS_PER_SHA256),
        .encrypted_log_preimages_length = private_previous ? ++seed : 0,
        .unencrypted_log_preimages_length = ++seed,
        .new_contracts = std::array<NewContractData<NT>, MAX_NEW_CONTRACTS_PER_TX>(),
        .optionally_revealed_data =
            std::array<OptionallyRevealedData<NT>, MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX>(),
        .public_data_update_requests =
            std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>(),
        .public_data_reads = std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_TX>()
    };

    const KernelCircuitPublicInputs<NT> public_inputs = {
        .end = end_accumulated_data,
        .constants = end_constants,
        .is_private = private_previous,
    };

    const PreviousKernelData<NT> previous_kernel = {
        .public_inputs = public_inputs,
    };

    // NOLINTNEXTLINE(misc-const-correctness)
    PublicKernelInputs<NT> kernel_inputs = {
        .previous_kernel = previous_kernel,
        .public_call = public_call_data,
    };
    return kernel_inputs;
}  // namespace aztec3::circuits::kernel::public_kernel

template <typename KernelInput>
void validate_public_kernel_outputs_correctly_propagated(const KernelInput& inputs,
                                                         const KernelCircuitPublicInputs<NT>& public_inputs)
{
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    size_t st_index = 0;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; i++) {
        const auto& contract_storage_update_request =
            inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests[i];
        if (contract_storage_update_request.is_empty()) {
            continue;
        }
        const auto public_data_update_request = public_data_update_request_from_contract_storage_update_request(
            contract_storage_update_request, contract_address);
        ASSERT_EQ(public_inputs.end.public_data_update_requests[st_index++], public_data_update_request);
    }

    size_t sr_index = 0;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_READS_PER_TX; i++) {
        const auto& read = inputs.public_call.call_stack_item.public_inputs.contract_storage_reads[i];
        if (read.is_empty()) {
            continue;
        }
        const auto public_read = public_data_read_from_contract_storage_read(read, contract_address);
        ASSERT_EQ(public_inputs.end.public_data_reads[sr_index++], public_read);
    }
}

void validate_private_data_propagation(DummyBuilder& builder,
                                       const PublicKernelInputs<NT>& inputs,
                                       const KernelCircuitPublicInputs<NT>& public_inputs)
{
    ASSERT_TRUE(source_arrays_are_in_target(builder,
                                            inputs.previous_kernel.public_inputs.end.private_call_stack,
                                            std::array<NT::fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>{},
                                            public_inputs.end.private_call_stack));

    ASSERT_TRUE(source_arrays_are_in_target(builder,
                                            inputs.previous_kernel.public_inputs.end.new_contracts,
                                            std::array<NewContractData<NT>, MAX_NEW_CONTRACTS_PER_TX>(),
                                            public_inputs.end.new_contracts));

    ASSERT_EQ(inputs.previous_kernel.public_inputs.end.encrypted_logs_hash,
              inputs.previous_kernel.public_inputs.end.encrypted_logs_hash);
    ASSERT_EQ(inputs.previous_kernel.public_inputs.end.encrypted_log_preimages_length,
              inputs.previous_kernel.public_inputs.end.encrypted_log_preimages_length);

    ASSERT_EQ(inputs.previous_kernel.public_inputs.end.optionally_revealed_data,
              public_inputs.end.optionally_revealed_data);
}

TEST(public_kernel_tests, only_valid_public_data_reads_should_be_propagated)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__only_valid_public_data_reads_should_be_propagated");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // modify the contract storage reads so only 2 are valid and only those should be propagated
    const auto first_valid = ContractStorageRead<NT>{
        .storage_slot = 123456789,
        .current_value = 76543,
    };
    const auto second_valid = ContractStorageRead<NT>{
        .storage_slot = 123456789,
        .current_value = 76543,
    };
    std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> reads =
        std::array<ContractStorageRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL>();
    reads[1] = first_valid;
    reads[3] = second_valid;
    inputs.public_call.call_stack_item.public_inputs.contract_storage_reads = reads;

    // adjust the call stack item hash for the current call in the previous iteration
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] = inputs.public_call.call_stack_item.hash();

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_EQ(dummyBuilder.get_first_failure(), utils::CircuitError::no_error());
    ASSERT_FALSE(dummyBuilder.failed());

    ASSERT_FALSE(public_inputs.is_private);
    ASSERT_EQ(public_inputs.constants.tx_context, inputs.previous_kernel.public_inputs.constants.tx_context);

    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    // only the 2 valid reads should have been propagated
    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    const auto public_read_1 = public_data_read_from_contract_storage_read(first_valid, contract_address);
    const auto public_read_2 = public_data_read_from_contract_storage_read(second_valid, contract_address);
    ASSERT_EQ(public_inputs.end.public_data_reads[0], public_read_1);
    ASSERT_EQ(public_inputs.end.public_data_reads[1], public_read_2);
}

TEST(public_kernel_tests, only_valid_update_requests_should_be_propagated)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__only_valid_update_requests_should_be_propagated");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // modify the contract storage update requests so only 2 are valid and only those should be propagated
    const auto first_valid = ContractStorageUpdateRequest<NT>{
        .storage_slot = 123456789,
        .old_value = 76543,
        .new_value = 76544,
    };
    const auto second_valid = ContractStorageUpdateRequest<NT>{
        .storage_slot = 987654321,
        .old_value = 86543,
        .new_value = 86544,
    };
    std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> update_requests =
        std::array<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>();
    update_requests[1] = first_valid;
    update_requests[3] = second_valid;
    inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests = update_requests;

    // adjust the call stack item hash for the current call in the previous iteration
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] = inputs.public_call.call_stack_item.hash();

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_EQ(dummyBuilder.get_first_failure(), utils::CircuitError::no_error());
    ASSERT_FALSE(dummyBuilder.failed());

    ASSERT_FALSE(public_inputs.is_private);
    ASSERT_EQ(public_inputs.constants.tx_context, inputs.previous_kernel.public_inputs.constants.tx_context);

    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    // only the 2 valid update requests should have been propagated
    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    const auto public_write_1 =
        public_data_update_request_from_contract_storage_update_request(first_valid, contract_address);
    const auto public_write_2 =
        public_data_update_request_from_contract_storage_update_request(second_valid, contract_address);
    ASSERT_EQ(public_inputs.end.public_data_update_requests[0], public_write_1);
    ASSERT_EQ(public_inputs.end.public_data_update_requests[1], public_write_2);
}

TEST(public_kernel_tests, constructor_should_fail)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__constructor_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.call_stack_item.function_data.is_constructor = true;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__CONSTRUCTOR_NOT_ALLOWED);
}

TEST(public_kernel_tests, constructor_should_fail_2)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__constructor_should_fail_2");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.call_stack_item.public_inputs.call_context.is_contract_deployment = true;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_DEPLOYMENT_NOT_ALLOWED);
}

TEST(public_kernel_tests, no_bytecode_hash_should_fail)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__no_bytecode_hash_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.bytecode_hash = 0;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__BYTECODE_HASH_INVALID);
}


TEST(public_kernel_tests, invalid_is_internal)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__no_bytecode_hash_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // Make the call internal but msg_sender != storage_contract_address.
    inputs.public_call.call_stack_item.function_data.is_internal = true;
    inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender = 1;
    inputs.public_call.call_stack_item.public_inputs.call_context.storage_contract_address = 2;

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__IS_INTERNAL_BUT_NOT_SELF_CALL);
}

TEST(public_kernel_tests, contract_address_must_be_valid)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__contract_address_must_be_valid");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.call_stack_item.contract_address = 0;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_ADDRESS_INVALID);
}

TEST(public_kernel_tests, function_selector_must_be_valid)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__function_selector_must_be_valid");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.call_stack_item.function_data.selector = {
        .value = 0,
    };
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__FUNCTION_SIGNATURE_INVALID);
}

TEST(public_kernel_tests, private_call_should_fail)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__private_call_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    inputs.public_call.call_stack_item.function_data.is_private = true;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__PRIVATE_FUNCTION_NOT_ALLOWED);
}

TEST(public_kernel_tests, inconsistent_call_hash_should_fail)
{
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        DummyBuilder dummyBuilder = DummyBuilder(format("public_kernel_tests__inconsistent_call_hash_should_fail-", i));
        PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

        // change a value of something in the call stack pre-image
        inputs.public_call.public_call_stack_preimages[i].public_inputs.args_hash++;
        auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
        ASSERT_TRUE(dummyBuilder.failed());
        ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_MISMATCH);
    }
}

TEST(public_kernel_tests, incorrect_storage_contract_address_fails_for_regular_calls)
{
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        DummyBuilder dummyBuilder =
            DummyBuilder(format("public_kernel_tests__incorrect_storage_contract_address_fails_for_regular_calls-", i));
        PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

        // change the storage contract address so it does not equal the contract address
        const NT::fr new_contract_address =
            NT::fr(inputs.public_call.public_call_stack_preimages[i].contract_address) + 1;
        inputs.public_call.public_call_stack_preimages[i].public_inputs.call_context.storage_contract_address =
            new_contract_address;
        // update the call stack item hash after the change in the preimage
        inputs.public_call.call_stack_item.public_inputs.public_call_stack[i] =
            inputs.public_call.public_call_stack_preimages[i].hash();
        auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
        ASSERT_TRUE(dummyBuilder.failed());
        ASSERT_EQ(dummyBuilder.get_first_failure().code,
                  CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_STORAGE_ADDRESS);
    }
}

TEST(public_kernel_tests, incorrect_msg_sender_fails_for_regular_calls)
{
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        DummyBuilder dummyBuilder =
            DummyBuilder(format("public_kernel_tests__incorrect_msg_sender_fails_for_regular_calls-", i));
        PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);
        // set the msg sender to be the address of the called contract, which is wrong
        const auto new_msg_sender = inputs.public_call.public_call_stack_preimages[i].contract_address;
        // change the storage contract address so it does not equal the contract address
        inputs.public_call.public_call_stack_preimages[i].public_inputs.call_context.msg_sender = new_msg_sender;
        // update the call stack item hash after the change in the preimage
        inputs.public_call.call_stack_item.public_inputs.public_call_stack[i] =
            inputs.public_call.public_call_stack_preimages[i].hash();
        auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
        ASSERT_TRUE(dummyBuilder.failed());
        ASSERT_EQ(dummyBuilder.get_first_failure().code,
                  CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_MSG_SENDER);
    }
}

TEST(public_kernel_tests, public_kernel_circuit_succeeds_for_mixture_of_regular_and_delegate_calls)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_succeeds_for_mixture_of_regular_and_delegate_calls");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    const auto contract_address = NT::fr(inputs.public_call.call_stack_item.contract_address);
    const auto origin_msg_sender = NT::fr(inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender);
    const auto contract_portal_address = NT::fr(inputs.public_call.portal_contract_address);

    // redefine the child calls/stacks to use some delegate calls
    NT::uint32 const seed = 1000;
    NT::fr child_contract_address = 100000;
    NT::fr child_portal_contract_address = 200000;
    NT::boolean is_delegate_call = false;
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        inputs.public_call.public_call_stack_preimages[i] =
            // NOLINTNEXTLINE(readability-suspicious-call-argument)
            generate_call_stack_item(child_contract_address,
                                     is_delegate_call ? origin_msg_sender : contract_address,
                                     is_delegate_call ? contract_address : child_contract_address,
                                     is_delegate_call ? contract_portal_address : child_portal_contract_address,
                                     is_delegate_call,
                                     seed);
        inputs.public_call.call_stack_item.public_inputs.public_call_stack[i] =
            inputs.public_call.public_call_stack_preimages[i].hash();

        // change the next call type
        is_delegate_call = !is_delegate_call;
        child_contract_address++;
        child_portal_contract_address++;
    }

    // we update the hash of the current call stack item in the previous kernel,
    // since we modified the hash of the nested calls, which changes the hash of the parent item
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] = inputs.public_call.call_stack_item.hash();

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_EQ(dummyBuilder.get_first_failure(), utils::CircuitError::no_error());
    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, public_kernel_circuit_fails_on_incorrect_msg_sender_in_delegate_call)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_fails_on_incorrect_msg_sender_in_delegate_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    const auto contract_address = NT::fr(inputs.public_call.call_stack_item.contract_address);
    // const auto origin_msg_sender = NT::fr(inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender);
    const auto contract_portal_address = NT::fr(inputs.public_call.portal_contract_address);

    // set the first call stack item to be a delegate call
    std::array<PublicCallStackItem, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> child_call_stacks;
    NT::uint32 const seed = 1000;
    NT::fr const child_contract_address = 100000;
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> call_stack_hashes{};
    child_call_stacks[0] =
        // NOLINTNEXTLINE(readability-suspicious-call-argument)
        generate_call_stack_item(child_contract_address,
                                 contract_address,  // this should be the origin_msg_sender, not the contract address
                                 contract_address,
                                 contract_portal_address,
                                 true,
                                 seed);
    call_stack_hashes[0] = child_call_stacks[0].hash();

    inputs.public_call.call_stack_item.public_inputs.public_call_stack = call_stack_hashes;
    inputs.public_call.public_call_stack_preimages = child_call_stacks;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_MSG_SENDER);
}

TEST(public_kernel_tests, public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_fails_on_incorrect_storage_contract_in_delegate_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // const auto contract_address = NT::fr(inputs.public_call.call_stack_item.contract_address);
    const auto origin_msg_sender = NT::fr(inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender);
    const auto contract_portal_address = NT::fr(inputs.public_call.portal_contract_address);

    // set the first call stack item to be a delegate call
    std::array<PublicCallStackItem, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> child_call_stacks;
    NT::uint32 const seed = 1000;
    NT::fr const child_contract_address = 100000;
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> call_stack_hashes{};
    child_call_stacks[0] = generate_call_stack_item(child_contract_address,
                                                    origin_msg_sender,
                                                    child_contract_address,  // this should be contract_address
                                                    contract_portal_address,
                                                    true,
                                                    seed);
    call_stack_hashes[0] = child_call_stacks[0].hash();

    inputs.public_call.call_stack_item.public_inputs.public_call_stack = call_stack_hashes;
    inputs.public_call.public_call_stack_preimages = child_call_stacks;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_STORAGE_ADDRESS);
}

TEST(public_kernel_tests, public_kernel_circuit_fails_on_incorrect_portal_contract_in_delegate_call)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_fails_on_incorrect_portal_contract_in_delegate_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    const auto contract_address = NT::fr(inputs.public_call.call_stack_item.contract_address);
    const auto origin_msg_sender = NT::fr(inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender);
    // const auto contract_portal_address = NT::fr(inputs.public_call.portal_contract_address);

    // set the first call stack item to be a delegate call
    std::array<PublicCallStackItem, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> child_call_stacks;
    NT::uint32 const seed = 1000;
    NT::fr const child_contract_address = 100000;
    NT::fr const child_portal_contract = 200000;
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> call_stack_hashes{};
    // NOLINTNEXTLINE(readability-suspicious-call-argument)
    child_call_stacks[0] = generate_call_stack_item(child_contract_address,
                                                    origin_msg_sender,
                                                    contract_address,
                                                    child_portal_contract,  // this should be contract_portal_address
                                                    true,
                                                    seed);
    call_stack_hashes[0] = child_call_stacks[0].hash();

    inputs.public_call.call_stack_item.public_inputs.public_call_stack = call_stack_hashes;
    inputs.public_call.public_call_stack_preimages = child_call_stacks;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_PORTAL_ADDRESS);
}

TEST(public_kernel_tests, public_kernel_circuit_only_checks_non_empty_call_stacks)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_only_checks_non_empty_call_stacks");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    const auto contract_address = NT::fr(inputs.public_call.call_stack_item.contract_address);
    const auto origin_msg_sender = NT::fr(inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender);
    // const auto contract_portal_address = NT::fr(inputs.public_call.portal_contract_address);

    // set all but the first call stack item to have a zero call stack hash
    // these call stack items will have an contract portal address but will be ignored as the call stack will be ignored
    std::array<PublicCallStackItem, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>& child_call_stacks =
        inputs.public_call.public_call_stack_preimages;
    std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>& call_stack_hashes =
        inputs.public_call.call_stack_item.public_inputs.public_call_stack;
    NT::uint32 const seed = 1000;
    NT::fr const child_contract_address = 100000;
    NT::fr const child_portal_contract = 200000;
    for (size_t i = 1; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        // NOLINTNEXTLINE(readability-suspicious-call-argument)
        child_call_stacks[i] = generate_call_stack_item(child_contract_address,
                                                        origin_msg_sender,
                                                        contract_address,
                                                        child_portal_contract,  // shuold be contract_portal_address
                                                        false,
                                                        seed);
        // setting this to zero makes the call stack item be ignored so it won't fail
        call_stack_hashes[i] = 0;
    }
    // adjust the call stack item hash for the current call in the previous iteration
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] = inputs.public_call.call_stack_item.hash();

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_EQ(dummyBuilder.get_first_failure(), utils::CircuitError::no_error());
    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, public_kernel_circuit_with_private_previous_kernel_should_succeed)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_with_private_previous_kernel_should_succeed");
    PublicKernelInputs<NT> const inputs = get_kernel_inputs_with_previous_kernel(true);
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, circuit_outputs_should_be_correctly_populated_with_previous_private_kernel)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__circuit_outputs_should_be_correctly_populated_with_previous_private_kernel");
    PublicKernelInputs<NT> const inputs = get_kernel_inputs_with_previous_kernel(true);
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);

    // test that the prior set of private kernel public inputs were copied to the outputs
    validate_private_data_propagation(dummyBuilder, inputs, public_inputs);

    validate_public_kernel_outputs_correctly_propagated(inputs, public_inputs);
    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, private_previous_kernel_non_empty_private_call_stack_should_fail)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__private_previous_kernel_non_empty_private_call_stack_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);
    inputs.previous_kernel.public_inputs.end.private_call_stack[0] = 1;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__NON_EMPTY_PRIVATE_CALL_STACK);
}

TEST(public_kernel_tests, private_previous_kernel_empty_public_call_stack_should_fail)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__private_previous_kernel_empty_public_call_stack_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);
    inputs.previous_kernel.public_inputs.end.public_call_stack =
        std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>{};
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__EMPTY_PUBLIC_CALL_STACK);
}

TEST(public_kernel_tests, private_previous_kernel_non_private_previous_kernel_should_fail)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__private_previous_kernel_non_private_previous_kernel_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);
    inputs.previous_kernel.public_inputs.is_private = false;
    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__PREVIOUS_KERNEL_NOT_PRIVATE);
}

TEST(public_kernel_tests, previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call)
{
    DummyBuilder dummyBuilder = DummyBuilder(
        "public_kernel_tests__previous_private_kernel_fails_if_contract_storage_update_requests_on_static_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // the function call has contract storage update requests so setting it to static should fail
    inputs.public_call.call_stack_item.public_inputs.call_context.is_static_call = true;

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(
        dummyBuilder.get_first_failure().code,
        CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_CONTRACT_STORAGE_UPDATE_REQUESTS_PROHIBITED_FOR_STATIC_CALL);
}

TEST(public_kernel_tests, previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call)
{
    DummyBuilder dummyBuilder = DummyBuilder(
        "public_kernel_tests__previous_private_kernel_fails_if_incorrect_storage_contract_on_delegate_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(true);

    // the function call has the contract address and storage contract address equal and so it should fail for a
    // delegate call
    inputs.public_call.call_stack_item.public_inputs.call_context.is_delegate_call = true;

    auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_INVALID_STORAGE_ADDRESS_FOR_DELEGATE_CALL);
}

TEST(public_kernel_tests, public_kernel_circuit_with_public_previous_kernel_should_succeed)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_kernel_circuit_with_public_previous_kernel_should_succeed");
    PublicKernelInputs<NT> const inputs = get_kernel_inputs_with_previous_kernel(false);
    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, public_previous_kernel_empty_public_call_stack_should_fail)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_previous_kernel_empty_public_call_stack_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);
    inputs.previous_kernel.public_inputs.end.public_call_stack =
        std::array<NT::fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>{};
    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__EMPTY_PUBLIC_CALL_STACK);
}

TEST(public_kernel_tests, public_previous_kernel_private_previous_kernel_should_fail)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__public_previous_kernel_private_previous_kernel_should_fail");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);
    inputs.previous_kernel.public_inputs.is_private = true;
    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code, CircuitErrorCode::PUBLIC_KERNEL__PREVIOUS_KERNEL_NOT_PUBLIC);
}

TEST(public_kernel_tests, circuit_outputs_should_be_correctly_populated_with_previous_public_kernel)
{
    DummyBuilder dummyBuilder =
        DummyBuilder("public_kernel_tests__circuit_outputs_should_be_correctly_populated_with_previous_public_kernel");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);

    // setup 2 previous data writes on the public inputs
    const auto first_write = PublicDataUpdateRequest<NT>{
        .leaf_index = 123456789,
        .old_value = 76543,
        .new_value = 76544,
    };
    const auto second_write = PublicDataUpdateRequest<NT>{
        .leaf_index = 987654321,
        .old_value = 86543,
        .new_value = 86544,
    };
    std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> initial_writes =
        std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>();
    initial_writes[0] = first_write;
    initial_writes[1] = second_write;
    inputs.previous_kernel.public_inputs.end.public_data_update_requests = initial_writes;

    // setup 2 previous data reads on the public inputs
    const auto first_read = PublicDataRead<NT>{
        .leaf_index = 123456789,
        .value = 96543,
    };
    const auto second_read = PublicDataRead<NT>{
        .leaf_index = 987654321,
        .value = 96544,
    };
    std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_TX> initial_reads =
        std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_TX>();
    initial_reads[0] = first_read;
    initial_reads[1] = second_read;
    inputs.previous_kernel.public_inputs.end.public_data_reads = initial_reads;

    // setup 2 previous new commitments
    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX> initial_commitments{};
    initial_commitments[0] = fr(1);
    initial_commitments[1] = fr(2);
    inputs.previous_kernel.public_inputs.end.new_commitments = initial_commitments;

    // setup 2 previous new nullifiers
    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX> initial_nullifiers{};
    initial_nullifiers[0] = fr(12345);
    initial_nullifiers[1] = fr(67890);
    inputs.previous_kernel.public_inputs.end.new_nullifiers = initial_nullifiers;

    // setup 1 new l2 to l1 messages
    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_TX> initial_l2_to_l1_messages{};
    initial_l2_to_l1_messages[0] = fr(1);
    inputs.previous_kernel.public_inputs.end.new_l2_to_l1_msgs = initial_l2_to_l1_messages;

    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);

    // test that the prior set of private kernel public inputs were copied to the outputs
    validate_private_data_propagation(dummyBuilder, inputs, public_inputs);

    // this call should have been popped from the public call stack and the stack of call pre images pushed on
    for (size_t i = 0; i < MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL; i++) {
        ASSERT_EQ(public_inputs.end.public_call_stack[i],
                  inputs.public_call.call_stack_item.public_inputs.public_call_stack[i]);
    }

    // we should now see the public data reads and writes, new commitments, new_nullifiers,
    // l2_to_l1_messages from this iteration appended to the combined output
    ASSERT_EQ(array_length(public_inputs.end.public_data_reads),
              array_length(inputs.previous_kernel.public_inputs.end.public_data_reads) +
                  array_length(inputs.public_call.call_stack_item.public_inputs.contract_storage_reads));
    ASSERT_EQ(array_length(public_inputs.end.public_data_update_requests),
              array_length(inputs.previous_kernel.public_inputs.end.public_data_update_requests) +
                  array_length(inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests));
    ASSERT_EQ(array_length(public_inputs.end.new_commitments),
              array_length(inputs.previous_kernel.public_inputs.end.new_commitments) +
                  array_length(inputs.public_call.call_stack_item.public_inputs.new_commitments));
    ASSERT_EQ(array_length(public_inputs.end.new_nullifiers),
              array_length(inputs.previous_kernel.public_inputs.end.new_nullifiers) +
                  array_length(inputs.public_call.call_stack_item.public_inputs.new_nullifiers));
    ASSERT_EQ(array_length(public_inputs.end.new_l2_to_l1_msgs),
              array_length(inputs.previous_kernel.public_inputs.end.new_l2_to_l1_msgs) +
                  array_length(inputs.public_call.call_stack_item.public_inputs.new_l2_to_l1_msgs));


    const auto contract_address = inputs.public_call.call_stack_item.contract_address;
    const auto portal_contract_address = inputs.public_call.portal_contract_address;
    std::array<PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL> const expected_new_writes =
        public_data_update_requests_from_contract_storage_update_requests(
            inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests, contract_address);

    // Unencrypted logs hash and preimage lengths should now be correctly accumulated
    auto const& public_inputs_unencrypted_logs_hash = inputs.previous_kernel.public_inputs.end.unencrypted_logs_hash;
    auto const& unencrypted_logs_hash = inputs.public_call.call_stack_item.public_inputs.unencrypted_logs_hash;
    // TODO(914) Should this be unused?
    [[maybe_unused]] auto const& expected_unencrypted_logs_hash =
        accumulate_sha256<NT>({ public_inputs_unencrypted_logs_hash[0],
                                public_inputs_unencrypted_logs_hash[1],
                                unencrypted_logs_hash[0],
                                unencrypted_logs_hash[1] });

    auto const& public_inputs_unencrypted_log_preimages_length =
        inputs.previous_kernel.public_inputs.end.unencrypted_log_preimages_length;
    auto const& unencrypted_log_preimages_length =
        inputs.public_call.call_stack_item.public_inputs.unencrypted_log_preimages_length;
    ASSERT_EQ(public_inputs.end.unencrypted_log_preimages_length,
              unencrypted_log_preimages_length + public_inputs_unencrypted_log_preimages_length);

    ASSERT_TRUE(source_arrays_are_in_target(dummyBuilder,
                                            inputs.previous_kernel.public_inputs.end.public_data_update_requests,
                                            expected_new_writes,
                                            public_inputs.end.public_data_update_requests));

    std::array<PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_CALL> const expected_new_reads =
        public_data_reads_from_contract_storage_reads(
            inputs.public_call.call_stack_item.public_inputs.contract_storage_reads, contract_address);

    ASSERT_TRUE(source_arrays_are_in_target(dummyBuilder,
                                            inputs.previous_kernel.public_inputs.end.public_data_reads,
                                            expected_new_reads,
                                            public_inputs.end.public_data_reads));

    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_CALL> const expected_new_commitments =
        new_commitments_as_siloed_commitments(inputs.public_call.call_stack_item.public_inputs.new_commitments,
                                              contract_address);

    ASSERT_TRUE(source_arrays_are_in_target(dummyBuilder,
                                            inputs.previous_kernel.public_inputs.end.new_commitments,
                                            expected_new_commitments,
                                            public_inputs.end.new_commitments));

    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_CALL> const expected_new_nullifiers = new_nullifiers_as_siloed_nullifiers(
        inputs.public_call.call_stack_item.public_inputs.new_nullifiers, contract_address);

    ASSERT_TRUE(source_arrays_are_in_target(dummyBuilder,
                                            inputs.previous_kernel.public_inputs.end.new_nullifiers,
                                            expected_new_nullifiers,
                                            public_inputs.end.new_nullifiers));

    // Reading the chain id and version from the tx context
    fr const chain_id = inputs.previous_kernel.public_inputs.constants.tx_context.chain_id;
    fr const version = inputs.previous_kernel.public_inputs.constants.tx_context.version;

    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> const expected_new_messages =
        new_l2_messages_from_message(inputs.public_call.call_stack_item.public_inputs.new_l2_to_l1_msgs,
                                     contract_address,
                                     portal_contract_address,
                                     chain_id,
                                     version);

    ASSERT_TRUE(source_arrays_are_in_target(dummyBuilder,
                                            inputs.previous_kernel.public_inputs.end.new_l2_to_l1_msgs,
                                            expected_new_messages,
                                            public_inputs.end.new_l2_to_l1_msgs));

    ASSERT_FALSE(dummyBuilder.failed());
}

TEST(public_kernel_tests, previous_public_kernel_fails_if_contract_storage_update_requests_on_static_call)
{
    DummyBuilder dummyBuilder = DummyBuilder(
        "public_kernel_tests__previous_public_kernel_fails_if_contract_storage_update_requests_on_static_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);

    // the function call has contract storage update requests so setting it to static should fail
    inputs.public_call.call_stack_item.public_inputs.call_context.is_static_call = true;

    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(
        dummyBuilder.get_first_failure().code,
        CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_CONTRACT_STORAGE_UPDATE_REQUESTS_PROHIBITED_FOR_STATIC_CALL);
}

TEST(public_kernel_tests, previous_public_kernel_fails_if_incorrect_storage_contract_on_delegate_call)
{
    DummyBuilder dummyBuilder = DummyBuilder(
        "public_kernel_tests__previous_public_kernel_fails_if_incorrect_storage_contract_on_delegate_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);

    // the function call has the contract address and storage contract address equal and so it should fail for a
    // delegate call
    inputs.public_call.call_stack_item.public_inputs.call_context.is_delegate_call = true;

    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_INVALID_STORAGE_ADDRESS_FOR_DELEGATE_CALL);
}

TEST(public_kernel_tests, public_kernel_fails_creating_new_commitments_on_static_call)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_fails_creating_new_commitments_on_static_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);

    // the function call has the contract address and storage contract address equal and so it should fail for a
    // delegate call
    inputs.public_call.call_stack_item.public_inputs.call_context.is_static_call = true;

    // set previously set items to 0
    inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests =
        empty_array_of_values<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>();

    // regenerate call data hash
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] =
        get_call_stack_item_hash(inputs.public_call.call_stack_item);

    // Update call stack hash
    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__NEW_COMMITMENTS_PROHIBITED_IN_STATIC_CALL);
}

TEST(public_kernel_tests, public_kernel_fails_creating_new_nullifiers_on_static_call)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_fails_creating_new_nullifiers_on_static_call");
    PublicKernelInputs<NT> inputs = get_kernel_inputs_with_previous_kernel(false);

    // the function call has the contract address and storage contract address equal and so it should fail for a
    // delegate call
    inputs.public_call.call_stack_item.public_inputs.call_context.is_static_call = true;

    // set previously set items to 0
    inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests =
        empty_array_of_values<ContractStorageUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>();
    inputs.public_call.call_stack_item.public_inputs.new_commitments =
        empty_array_of_values<NT::fr, MAX_NEW_COMMITMENTS_PER_CALL>();

    // regenerate call data hash
    inputs.previous_kernel.public_inputs.end.public_call_stack[0] =
        get_call_stack_item_hash(inputs.public_call.call_stack_item);

    // Update call stack hash
    auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dummyBuilder, inputs);
    ASSERT_TRUE(dummyBuilder.failed());
    ASSERT_EQ(dummyBuilder.get_first_failure().code,
              CircuitErrorCode::PUBLIC_KERNEL__NEW_NULLIFIERS_PROHIBITED_IN_STATIC_CALL);
}

TEST(public_kernel_tests, logs_are_handled_as_expected)
{
    DummyBuilder dummyBuilder = DummyBuilder("public_kernel_tests__logs_are_handled_as_expected");
    PublicKernelInputs<NT> const& inputs = get_kernel_inputs_with_previous_kernel(true);
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& zero_hash{};

    // Ensure encrypted logs hash values are non-zero
    ASSERT_NE(inputs.previous_kernel.public_inputs.end.encrypted_logs_hash, zero_hash);

    auto const& public_inputs = native_public_kernel_circuit_private_previous_kernel(dummyBuilder, inputs);

    // Encrypted logs hash values are propagated form input to output without change
    ASSERT_EQ(inputs.previous_kernel.public_inputs.end.encrypted_logs_hash, public_inputs.end.encrypted_logs_hash);

    // Unencrypted logs hash and preimage lengths should now be correctly accumulated
    auto const& public_inputs_unencrypted_logs_hash = inputs.previous_kernel.public_inputs.end.unencrypted_logs_hash;
    auto const& unencrypted_logs_hash = inputs.public_call.call_stack_item.public_inputs.unencrypted_logs_hash;
    // TODO(914) Should this be unused?
    [[maybe_unused]] auto const& expected_unencrypted_logs_hash =
        accumulate_sha256<NT>({ public_inputs_unencrypted_logs_hash[0],
                                public_inputs_unencrypted_logs_hash[1],
                                unencrypted_logs_hash[0],
                                unencrypted_logs_hash[1] });

    auto const& public_inputs_unencrypted_log_preimages_length =
        inputs.previous_kernel.public_inputs.end.unencrypted_log_preimages_length;
    auto const& unencrypted_log_preimages_length =
        inputs.public_call.call_stack_item.public_inputs.unencrypted_log_preimages_length;
    ASSERT_EQ(public_inputs.end.unencrypted_log_preimages_length,
              unencrypted_log_preimages_length + public_inputs_unencrypted_log_preimages_length);

    ASSERT_FALSE(dummyBuilder.failed());
}

}  // namespace aztec3::circuits::kernel::public_kernel