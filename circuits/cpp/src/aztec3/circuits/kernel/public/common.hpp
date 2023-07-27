#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/contract_storage_read.hpp"
#include "aztec3/circuits/abis/contract_storage_update_request.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/public_data_update_request.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::ContractStorageRead;
using aztec3::circuits::abis::ContractStorageUpdateRequest;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PublicDataRead;
using aztec3::circuits::abis::PublicDataUpdateRequest;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::check_membership;
using aztec3::circuits::compute_public_data_tree_index;
using aztec3::circuits::compute_public_data_tree_value;
using aztec3::circuits::root_from_sibling_path;
using aztec3::utils::array_length;
using aztec3::utils::array_pop;
using aztec3::utils::array_push;
using aztec3::utils::push_array_to_array;

namespace aztec3::circuits::kernel::public_kernel {

/**
 * @brief Validate that all pre-images on the call stack hash to equal the accumulated data
 * @tparam The type of kernel input
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_call_stack(DummyBuilder& builder, KernelInput const& public_kernel_inputs)
{
    // Ensures that the stack of pre-images corresponds to the call stack
    auto& stack = public_kernel_inputs.public_call.call_stack_item.public_inputs.public_call_stack;
    auto& preimages = public_kernel_inputs.public_call.public_call_stack_preimages;

    // grab our contract address, our storage contract address and our portal contract address to verify
    // child executions in the case of delegate call types
    auto our_contract_address = public_kernel_inputs.public_call.call_stack_item.contract_address;
    auto our_storage_address =
        public_kernel_inputs.public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    auto our_msg_sender = public_kernel_inputs.public_call.call_stack_item.public_inputs.call_context.msg_sender;
    auto our_portal_contract_address =
        public_kernel_inputs.public_call.call_stack_item.public_inputs.call_context.portal_contract_address;

    for (size_t i = 0; i < stack.size(); ++i) {
        const auto& hash = stack[i];
        const auto& preimage = preimages[i];

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".
        if (hash == 0) {
            continue;
        }

        const auto is_delegate_call = preimage.public_inputs.call_context.is_delegate_call;
        const auto is_static_call = preimage.public_inputs.call_context.is_static_call;
        const auto contract_being_called = preimage.contract_address;

        const auto calculated_hash = preimage.hash();
        builder.do_assert(
            hash == calculated_hash,
            format(
                "public_call_stack[", i, "] = ", hash, "; does not reconcile with calculatedHash = ", calculated_hash),
            CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_MISMATCH);

        // here we validate the msg sender for each call on the stack
        // we need to consider regular vs delegate calls
        const auto preimage_msg_sender = preimage.public_inputs.call_context.msg_sender;
        const auto expected_msg_sender = is_delegate_call ? our_msg_sender : our_contract_address;
        builder.do_assert(expected_msg_sender == preimage_msg_sender,
                          format("call_stack_msg_sender[",
                                 i,
                                 "] = ",
                                 preimage_msg_sender,
                                 " expected ",
                                 expected_msg_sender,
                                 "; does not reconcile"),
                          CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_MSG_SENDER);

        // here we validate the storage address for each call on the stack
        // we need to consider regular vs delegate calls
        const auto preimage_storage_address = preimage.public_inputs.call_context.storage_contract_address;
        const auto expected_storage_address = is_delegate_call ? our_storage_address : contract_being_called;
        builder.do_assert(expected_storage_address == preimage_storage_address,
                          format("call_stack_storage_address[",
                                 i,
                                 "] = ",
                                 preimage_storage_address,
                                 " expected ",
                                 expected_storage_address,
                                 "; does not reconcile"),
                          CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_STORAGE_ADDRESS);

        // if it is a delegate call then we check that the portal contract in the pre image is our portal contract
        const auto preimage_portal_address = preimage.public_inputs.call_context.portal_contract_address;
        const auto expected_portal_address = our_portal_contract_address;
        builder.do_assert(!is_delegate_call || expected_portal_address == preimage_portal_address,
                          format("call_stack_portal_address[",
                                 i,
                                 "] = ",
                                 preimage_portal_address,
                                 " expected ",
                                 expected_portal_address,
                                 "; does not reconcile for a delagate call"),
                          CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_INVALID_PORTAL_ADDRESS);

        const auto num_contract_storage_update_requests =
            array_length(preimage.public_inputs.contract_storage_update_requests);
        builder.do_assert(
            !is_static_call || num_contract_storage_update_requests == 0,
            format("contract_storage_update_requests[", i, "] should be empty for a static call"),
            CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_CONTRACT_STORAGE_UPDATES_PROHIBITED_FOR_STATIC_CALL);
    }
};

/**
 * @brief Validates the call context of the current iteration
 * @tparam The type of kernel input
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_call_context(DummyBuilder& builder, KernelInput const& public_kernel_inputs)
{
    const auto& call_stack_item = public_kernel_inputs.public_call.call_stack_item;
    const auto is_delegate_call = call_stack_item.public_inputs.call_context.is_delegate_call;
    const auto is_static_call = call_stack_item.public_inputs.call_context.is_static_call;
    const auto contract_address = call_stack_item.contract_address;
    const auto storage_contract_address = call_stack_item.public_inputs.call_context.storage_contract_address;
    const auto contract_storage_update_requests_length =
        array_length(call_stack_item.public_inputs.contract_storage_update_requests);

    builder.do_assert(!is_delegate_call || contract_address != storage_contract_address,
                      std::string("curent contract address must not match storage contract address for delegate calls"),
                      CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_INVALID_STORAGE_ADDRESS_FOR_DELEGATE_CALL);

    builder.do_assert(
        !is_static_call || contract_storage_update_requests_length == 0,
        std::string("No contract storage update requests are allowed for static calls"),
        CircuitErrorCode::PUBLIC_KERNEL__CALL_CONTEXT_CONTRACT_STORAGE_UPDATE_REQUESTS_PROHIBITED_FOR_STATIC_CALL);
};

/**
 * @brief Validates the kernel execution of the current iteration
 * @tparam The type of kernel input
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_kernel_execution(DummyBuilder& builder, KernelInput const& public_kernel_inputs)
{
    common_validate_call_context(builder, public_kernel_inputs);
    common_validate_call_stack(builder, public_kernel_inputs);
};

/**
 * @brief Validates inputs to the kernel circuit that are common to all invocation scenarios
 * @tparam The type of kernel input
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_inputs(DummyBuilder& builder, KernelInput const& public_kernel_inputs)
{
    // Validates commons inputs for all type of kernel inputs
    const auto& this_call_stack_item = public_kernel_inputs.public_call.call_stack_item;
    builder.do_assert(this_call_stack_item.public_inputs.call_context.is_contract_deployment == false,
                      "Contract deployment can't be a public function",
                      CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_DEPLOYMENT_NOT_ALLOWED);
    builder.do_assert(this_call_stack_item.contract_address != 0,
                      "Contract address must be non-zero",
                      CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_ADDRESS_INVALID);
    builder.do_assert(this_call_stack_item.function_data.function_selector != 0,
                      "Function signature must be non-zero",
                      CircuitErrorCode::PUBLIC_KERNEL__FUNCTION_SIGNATURE_INVALID);
    builder.do_assert(this_call_stack_item.function_data.is_constructor == false,
                      "Constructors can't be public functions",
                      CircuitErrorCode::PUBLIC_KERNEL__CONSTRUCTOR_NOT_ALLOWED);
    builder.do_assert(this_call_stack_item.function_data.is_private == false,
                      "Cannot execute a private function with the public kernel circuit",
                      CircuitErrorCode::PUBLIC_KERNEL__PRIVATE_FUNCTION_NOT_ALLOWED);
    builder.do_assert(public_kernel_inputs.public_call.bytecode_hash != 0,
                      "Bytecode hash must be non-zero",
                      CircuitErrorCode::PUBLIC_KERNEL__BYTECODE_HASH_INVALID);

    if (this_call_stack_item.function_data.is_internal) {
        auto const target = this_call_stack_item.contract_address;
        auto const sender = this_call_stack_item.public_inputs.call_context.msg_sender;

        builder.do_assert(target == sender,
                          "call is internal, but msg_sender is not self",
                          CircuitErrorCode::PUBLIC_KERNEL__IS_INTERNAL_BUT_NOT_SELF_CALL);
    }
}

template <typename KernelInput, typename Builder>
void perform_static_call_checks(Builder& builder, KernelInput const& public_kernel_inputs)
{
    // If the call is a static call, there should be no new commitments or nullifiers.
    const auto& public_call_public_inputs = public_kernel_inputs.public_call.call_stack_item.public_inputs;

    const auto& is_static_call = public_call_public_inputs.call_context.is_static_call;
    const auto& new_commitments = public_call_public_inputs.new_commitments;
    const auto& new_nullifiers = public_call_public_inputs.new_nullifiers;

    if (is_static_call) {
        builder.do_assert(utils::is_array_empty(new_commitments) == true,
                          "no new commitments must be created for static calls",
                          CircuitErrorCode::PUBLIC_KERNEL__NEW_COMMITMENTS_PROHIBITED_IN_STATIC_CALL);
        builder.do_assert(utils::is_array_empty(new_nullifiers) == true,
                          "no new nullifiers must be created for static calls",
                          CircuitErrorCode::PUBLIC_KERNEL__NEW_NULLIFIERS_PROHIBITED_IN_STATIC_CALL);
    }
}

/**
 * @brief Proagates valid (i.e. non-empty) update requests from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void propagate_valid_public_data_update_requests(Builder& builder,
                                                 KernelInput const& public_kernel_inputs,
                                                 KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    const auto& contract_address = public_kernel_inputs.public_call.call_stack_item.contract_address;
    const auto& update_requests =
        public_kernel_inputs.public_call.call_stack_item.public_inputs.contract_storage_update_requests;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; ++i) {
        const auto& update_request = update_requests[i];
        if (update_request.is_empty()) {
            continue;
        }
        const auto new_write = PublicDataUpdateRequest<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, update_request.storage_slot),
            .old_value = compute_public_data_tree_value<NT>(update_request.old_value),
            .new_value = compute_public_data_tree_value<NT>(update_request.new_value),
        };
        array_push(
            builder,
            circuit_outputs.end.public_data_update_requests,
            new_write,
            format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many public data update requests in one tx"));
    }
}

/**
 * @brief Proagates valid (i.e. non-empty) public data reads from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void propagate_valid_public_data_reads(Builder& builder,
                                       KernelInput const& public_kernel_inputs,
                                       KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    const auto& contract_address = public_kernel_inputs.public_call.call_stack_item.contract_address;
    const auto& reads = public_kernel_inputs.public_call.call_stack_item.public_inputs.contract_storage_reads;
    for (size_t i = 0; i < MAX_PUBLIC_DATA_READS_PER_TX; ++i) {
        const auto& contract_storage_read = reads[i];
        if (contract_storage_read.is_empty()) {
            continue;
        }
        const auto new_read = PublicDataRead<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, contract_storage_read.storage_slot),
            .value = compute_public_data_tree_value<NT>(contract_storage_read.current_value),
        };
        array_push(builder,
                   circuit_outputs.end.public_data_reads,
                   new_read,
                   format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many public data reads in one tx"));
    }
}

/**
 * @brief Propagates new commitments from this iteration to the circuit output.
 *
 * @tparam The type of the kernel input
 * @tparam The builder type
 * @param public_kernel_inputs The inputs to this iteration to the kernel circuit.
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void propagate_new_commitments(Builder& builder,
                               KernelInput const& public_kernel_inputs,
                               KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Get the new commitments
    const auto& public_call_public_inputs = public_kernel_inputs.public_call.call_stack_item.public_inputs;

    const auto& new_commitments = public_call_public_inputs.new_commitments;
    const auto& storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_CALL> siloed_new_commitments{};
    for (size_t i = 0; i < new_commitments.size(); ++i) {
        if (!new_commitments[i].is_zero()) {
            siloed_new_commitments[i] = silo_commitment<NT>(storage_contract_address, new_commitments[i]);
        }
    }

    push_array_to_array(builder,
                        siloed_new_commitments,
                        circuit_outputs.end.new_commitments,
                        format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many new commitments in one tx"));
}

/**
 * @brief Propagates new nullifiers from this iteration to the circuit output.
 *
 * @tparam The type of the kernel input
 * @tparam The builder type
 * @param public_kernel_inputs The inputs to this iteration to the kernel circuit.
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void propagate_new_nullifiers(Builder& builder,
                              KernelInput const& public_kernel_inputs,
                              KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Get the new commitments
    const auto& public_call_public_inputs = public_kernel_inputs.public_call.call_stack_item.public_inputs;

    const auto& new_nullifiers = public_call_public_inputs.new_nullifiers;
    const auto& storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;

    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX> siloed_new_nullifiers{};
    for (size_t i = 0; i < new_nullifiers.size(); ++i) {
        if (!new_nullifiers[i].is_zero()) {
            siloed_new_nullifiers[i] = silo_nullifier<NT>(storage_contract_address, new_nullifiers[i]);
        }
    }

    push_array_to_array(builder,
                        siloed_new_nullifiers,
                        circuit_outputs.end.new_nullifiers,
                        format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many new nullifiers in one tx"));
}

/**
 * @brief Propagates new l2 to l1 messages from this iteration to the circuit output.
 *
 * @tparam The type of the kernel input
 * @param public_kernel_inputs The inputs to this iteration to the kernel circuit.
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void propagate_new_l2_to_l1_messages(Builder& builder,
                                     KernelInput const& public_kernel_inputs,
                                     KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Get the new l2 messages
    const auto& public_call_public_inputs = public_kernel_inputs.public_call.call_stack_item.public_inputs;

    const auto& portal_contract_address = public_kernel_inputs.public_call.portal_contract_address;
    const auto& storage_contract_address = public_call_public_inputs.call_context.storage_contract_address;
    const auto& new_l2_to_l1_msgs = public_call_public_inputs.new_l2_to_l1_msgs;

    std::array<NT::fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> new_l2_to_l1_msgs_to_insert{};
    for (size_t i = 0; i < new_l2_to_l1_msgs.size(); ++i) {
        if (!new_l2_to_l1_msgs[i].is_zero()) {
            const auto chain_id = public_kernel_inputs.previous_kernel.public_inputs.constants.tx_context.chain_id;
            const auto version = public_kernel_inputs.previous_kernel.public_inputs.constants.tx_context.version;

            new_l2_to_l1_msgs_to_insert[i] = compute_l2_to_l1_hash<NT>(
                storage_contract_address, version, portal_contract_address, chain_id, new_l2_to_l1_msgs[i]);
        }
    }
    push_array_to_array(
        builder,
        new_l2_to_l1_msgs_to_insert,
        circuit_outputs.end.new_l2_to_l1_msgs,
        format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many new l2 to l1 messages in one tx"));
}

/**
 * @brief Accumulates unencrypted logs hashes and lengths.
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 * @note See the following thread if not clear:
 *       https://discourse.aztec.network/t/proposal-forcing-the-sequencer-to-actually-submit-data-to-l1/426
 * @note Used by public kernels which had previous iterations.
 */
template <typename NT> void accumulate_unencrypted_logs(PublicKernelInputs<NT> const& public_kernel_inputs,
                                                        KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    const auto public_call_public_inputs = public_kernel_inputs.public_call.call_stack_item.public_inputs;

    const auto& previous_kernel_end = public_kernel_inputs.previous_kernel.public_inputs.end;
    const auto& previous_unencrypted_logs_hash = previous_kernel_end.unencrypted_logs_hash;

    const auto& current_unencrypted_logs_hash = public_call_public_inputs.unencrypted_logs_hash;
    circuit_outputs.end.unencrypted_logs_hash = accumulate_sha256<NT>({ previous_unencrypted_logs_hash[0],
                                                                        previous_unencrypted_logs_hash[1],
                                                                        current_unencrypted_logs_hash[0],
                                                                        current_unencrypted_logs_hash[1] });

    // Add log preimages lengths from current iteration to accumulated lengths
    const auto& current_unencrypted_log_preimages_length = public_call_public_inputs.unencrypted_log_preimages_length;
    circuit_outputs.end.unencrypted_log_preimages_length =
        previous_kernel_end.unencrypted_log_preimages_length + current_unencrypted_log_preimages_length;
}

/**
 * @brief Propagates valid (i.e. non-empty) public data reads from this iteration to the circuit output
 * @tparam The type of kernel input
 * @tparam The current builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput, typename Builder>
void common_update_public_end_values(Builder& builder,
                                     KernelInput const& public_kernel_inputs,
                                     KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Updates the circuit outputs with new state changes, call stack etc
    circuit_outputs.is_private = false;

    // If this call is a static call, certain operations are disallowed, such as creating new state.
    perform_static_call_checks(builder, public_kernel_inputs);

    const auto& stack = public_kernel_inputs.public_call.call_stack_item.public_inputs.public_call_stack;
    push_array_to_array(
        builder,
        stack,
        circuit_outputs.end.public_call_stack,
        format(PUBLIC_KERNEL_CIRCUIT_ERROR_MESSAGE_BEGINNING, "too many public call stack items in one tx"));

    propagate_new_commitments(builder, public_kernel_inputs, circuit_outputs);
    propagate_new_nullifiers(builder, public_kernel_inputs, circuit_outputs);

    propagate_new_l2_to_l1_messages(builder, public_kernel_inputs, circuit_outputs);

    propagate_valid_public_data_update_requests(builder, public_kernel_inputs, circuit_outputs);

    propagate_valid_public_data_reads(builder, public_kernel_inputs, circuit_outputs);
}

/**
 * @brief Initialises the circuit output end state from provided inputs
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be initialised
 */
void common_initialise_end_values(PublicKernelInputs<NT> const& public_kernel_inputs,
                                  KernelCircuitPublicInputs<NT>& circuit_outputs);

/**
 * @brief Validates that the call stack item for this circuit iteration is at the top of the call stack
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param public_inputs The circuit outputs
 */
void validate_this_public_call_hash(DummyBuilder& builder,
                                    PublicKernelInputs<NT> const& public_kernel_inputs,
                                    KernelCircuitPublicInputs<NT>& public_inputs);
}  // namespace aztec3::circuits::kernel::public_kernel