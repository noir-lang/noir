#pragma once

#include "init.hpp"

#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs_no_previous_kernel.hpp>
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/state_read.hpp>
#include <aztec3/circuits/abis/state_transition.hpp>
#include <aztec3/circuits/abis/public_data_transition.hpp>
#include <aztec3/utils/dummy_composer.hpp>
#include <aztec3/utils/array.hpp>
#include <aztec3/circuits/hash.hpp>

using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PublicDataRead;
using aztec3::circuits::abis::PublicDataTransition;
using aztec3::circuits::abis::StateRead;
using aztec3::circuits::abis::StateTransition;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using DummyComposer = aztec3::utils::DummyComposer;
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
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_kernel_execution(DummyComposer& composer, KernelInput const& public_kernel_inputs)
{
    // Ensures that the stack of pre-images corresponds to the call stack
    auto& stack = public_kernel_inputs.public_call.call_stack_item.public_inputs.public_call_stack;
    auto& preimages = public_kernel_inputs.public_call.public_call_stack_preimages;
    auto calling_contract = public_kernel_inputs.public_call.call_stack_item.contract_address;
    auto storage_address =
        public_kernel_inputs.public_call.call_stack_item.public_inputs.call_context.storage_contract_address;
    for (size_t i = 0; i < stack.size(); ++i) {
        const auto& hash = stack[i];
        const auto& preimage = preimages[i];

        // Note: this assumes it's computationally infeasible to have `0` as a valid call_stack_item_hash.
        // Assumes `hash == 0` means "this stack item is empty".
        const auto calculated_hash = hash == 0 ? 0 : preimage.hash();
        composer.do_assert(hash == calculated_hash,
                           format("public_call_stack[", i, "] = ", hash, "; does not reconcile"),
                           CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_MISMATCH);
        const auto preimage_calling_contract = preimage.public_inputs.call_context.msg_sender;
        composer.do_assert(calling_contract == preimage_calling_contract,
                           format("call_stack_msg_sender[",
                                  i,
                                  "] = ",
                                  preimage_calling_contract,
                                  " expected ",
                                  calling_contract,
                                  "; does not reconcile"),
                           CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_MISMATCH);
        const auto preimage_storage_address = preimage.public_inputs.call_context.storage_contract_address;
        composer.do_assert(storage_address == preimage_storage_address,
                           format("call_stack_storage_address[",
                                  i,
                                  "] = ",
                                  preimage_storage_address,
                                  " expected ",
                                  storage_address,
                                  "; does not reconcile"),
                           CircuitErrorCode::PUBLIC_KERNEL__PUBLIC_CALL_STACK_MISMATCH);
        calling_contract = preimage.contract_address;
        if (preimage.public_inputs.call_context.is_delegate_call == false) {
            storage_address = preimage.contract_address;
        }
    }
};

/**
 * @brief Validates inputs to the kernel circuit that are common to all invocation scenarios
 * @tparam The type of kernel input
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
template <typename KernelInput>
void common_validate_inputs(DummyComposer& composer, KernelInput const& public_kernel_inputs)
{
    // Validates commons inputs for all type of kernel inputs
    const auto& this_call_stack_item = public_kernel_inputs.public_call.call_stack_item;
    composer.do_assert(this_call_stack_item.public_inputs.call_context.is_contract_deployment == false,
                       "Contract deployment can't be a public function",
                       CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_DEPLOYMENT_NOT_ALLOWED);
    composer.do_assert(this_call_stack_item.contract_address != 0,
                       "Contract address must be valid",
                       CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_ADDRESS_INVALID);
    composer.do_assert(this_call_stack_item.function_data.function_selector != 0,
                       "Function signature must be valid",
                       CircuitErrorCode::PUBLIC_KERNEL__FUNCTION_SIGNATURE_INVALID);
    composer.do_assert(this_call_stack_item.function_data.is_constructor == false,
                       "Constructors can't be public functions",
                       CircuitErrorCode::PUBLIC_KERNEL__CONSTRUCTOR_NOT_ALLOWED);
    composer.do_assert(this_call_stack_item.function_data.is_private == false,
                       "Cannot execute a private function with the public kernel circuit",
                       CircuitErrorCode::PUBLIC_KERNEL__PRIVATE_FUNCTION_NOT_ALLOWED);
    composer.do_assert(public_kernel_inputs.public_call.bytecode_hash != 0,
                       "Bytecode hash must be valid",
                       CircuitErrorCode::PUBLIC_KERNEL__BYTECODE_HASH_INVALID);
}

/**
 * @brief Proagates valid (i.e. non-empty) state transitions from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput>
void propagate_valid_state_transitions(KernelInput const& public_kernel_inputs,
                                       KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    const auto& contract_address = public_kernel_inputs.public_call.call_stack_item.contract_address;
    const auto& transitions = public_kernel_inputs.public_call.call_stack_item.public_inputs.state_transitions;
    for (size_t i = 0; i < STATE_TRANSITIONS_LENGTH; ++i) {
        const auto& state_transition = transitions[i];
        if (state_transition.is_empty()) {
            continue;
        }
        const auto new_write = PublicDataTransition<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, state_transition.storage_slot),
            .new_value = compute_public_data_tree_value<NT>(state_transition.new_value),
        };
        array_push(circuit_outputs.end.state_transitions, new_write);
    }
}

/**
 * @brief Proagates valid (i.e. non-empty) state reads from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput>
void propagate_valid_state_reads(KernelInput const& public_kernel_inputs,
                                 KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    const auto& contract_address = public_kernel_inputs.public_call.call_stack_item.contract_address;
    const auto& reads = public_kernel_inputs.public_call.call_stack_item.public_inputs.state_reads;
    for (size_t i = 0; i < STATE_READS_LENGTH; ++i) {
        const auto& state_read = reads[i];
        if (state_read.is_empty()) {
            continue;
        }
        const auto new_read = PublicDataRead<NT>{
            .leaf_index = compute_public_data_tree_index<NT>(contract_address, state_read.storage_slot),
            .value = compute_public_data_tree_value<NT>(state_read.current_value),
        };
        array_push(circuit_outputs.end.state_reads, new_read);
    }
}

/**
 * @brief Proagates valid (i.e. non-empty) state reads from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
template <typename KernelInput>
void update_public_end_values(KernelInput const& public_kernel_inputs, KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Updates the circuit outputs with new state changes, call stack etc
    circuit_outputs.is_private = false;

    const auto& stack = public_kernel_inputs.public_call.call_stack_item.public_inputs.public_call_stack;
    push_array_to_array(stack, circuit_outputs.end.public_call_stack);

    propagate_valid_state_transitions(public_kernel_inputs, circuit_outputs);

    propagate_valid_state_reads(public_kernel_inputs, circuit_outputs);
}

/**
 * @brief Initialises the circuit output end state from provided inputs
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be initialised
 */
void common_initialise_end_values(PublicKernelInputs<NT> const& public_kernel_inputs,
                                  KernelCircuitPublicInputs<NT>& circuit_outputs);

} // namespace aztec3::circuits::kernel::public_kernel