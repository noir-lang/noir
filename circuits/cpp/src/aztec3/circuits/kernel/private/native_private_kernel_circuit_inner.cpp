#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;

using aztec3::circuits::abis::ContractLeafPreimage;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
using aztec3::circuits::kernel::private_kernel::common_initialise_end_values;
using aztec3::utils::array_length;
using aztec3::utils::array_pop;
using aztec3::utils::CircuitErrorCode;
using aztec3::utils::DummyCircuitBuilder;

void initialise_end_values(PreviousKernelData<NT> const& previous_kernel, KernelCircuitPublicInputs<NT>& public_inputs)
{
    common_initialise_end_values(previous_kernel, public_inputs);

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    auto& end = public_inputs.end;
    const auto& start = previous_kernel.public_inputs.end;
    end.read_requests = start.read_requests;
}
}  // namespace

namespace aztec3::circuits::kernel::private_kernel {

void pop_and_validate_this_private_call_hash(
    DummyCircuitBuilder& builder,
    PrivateCallData<NT> const& private_call,
    std::array<NT::fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>& private_call_stack)
{
    // TODO(mike): this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the
    // 'fee' tx, and the 'gas rebate' tx).
    const auto popped_private_call_hash = array_pop(private_call_stack);
    const auto calculated_this_private_call_hash = private_call.call_stack_item.hash();

    builder.do_assert(
        popped_private_call_hash == calculated_this_private_call_hash,
        format("calculated private_call_hash (",
               calculated_this_private_call_hash,
               ") does not match provided private_call_hash (",
               popped_private_call_hash,
               ") at the top of the call stack"),
        CircuitErrorCode::PRIVATE_KERNEL__CALCULATED_PRIVATE_CALL_HASH_AND_PROVIDED_PRIVATE_CALL_HASH_MISMATCH);
};

void validate_contract_tree_root(DummyCircuitBuilder& builder, PrivateKernelInputsInner<NT> const& private_inputs)
{
    auto const& purported_contract_tree_root =
        private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.contract_tree_root;
    auto const& previous_kernel_contract_tree_root =
        private_inputs.previous_kernel.public_inputs.constants.block_data.contract_tree_root;
    builder.do_assert(
        purported_contract_tree_root == previous_kernel_contract_tree_root,
        "purported_contract_tree_root doesn't match previous_kernel_contract_tree_root",
        CircuitErrorCode::PRIVATE_KERNEL__PURPORTED_CONTRACT_TREE_ROOT_AND_PREVIOUS_KERNEL_CONTRACT_TREE_ROOT_MISMATCH);
}

void validate_inputs(DummyCircuitBuilder& builder, PrivateKernelInputsInner<NT> const& private_inputs)
{
    const auto& this_call_stack_item = private_inputs.private_call.call_stack_item;

    builder.do_assert(this_call_stack_item.function_data.is_private == true,
                      "Cannot execute a non-private function with the private kernel circuit",
                      CircuitErrorCode::PRIVATE_KERNEL__NON_PRIVATE_FUNCTION_EXECUTED_WITH_PRIVATE_KERNEL);

    const auto& start = private_inputs.previous_kernel.public_inputs.end;

    // TODO(mike): we might want to range-constrain the call_count to prevent some kind of overflow errors. Having
    // said that, iterating 2^254 times isn't feasible.

    NT::fr const start_private_call_stack_length = array_length(start.private_call_stack);

    builder.do_assert(private_inputs.previous_kernel.public_inputs.is_private == true,
                      "Cannot verify a non-private kernel snark in the private kernel circuit",
                      CircuitErrorCode::PRIVATE_KERNEL__NON_PRIVATE_KERNEL_VERIFIED_WITH_PRIVATE_KERNEL);
    builder.do_assert(this_call_stack_item.function_data.is_constructor == false,
                      "A constructor must be executed as the first tx in the recursion",
                      CircuitErrorCode::PRIVATE_KERNEL__CONSTRUCTOR_EXECUTED_IN_RECURSION);
    builder.do_assert(start_private_call_stack_length != 0,
                      "Cannot execute private kernel circuit with an empty private call stack",
                      CircuitErrorCode::PRIVATE_KERNEL__PRIVATE_CALL_STACK_EMPTY);
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_inner(DummyCircuitBuilder& builder,
                                                                  PrivateKernelInputsInner<NT> const& private_inputs)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    common_validate_previous_kernel_values(builder, private_inputs.previous_kernel.public_inputs.end);

    // Do this before any functions can modify the inputs.
    initialise_end_values(private_inputs.previous_kernel, public_inputs);

    validate_inputs(builder, private_inputs);

    common_validate_arrays(builder, private_inputs.private_call.call_stack_item.public_inputs);

    pop_and_validate_this_private_call_hash(builder, private_inputs.private_call, public_inputs.end.private_call_stack);

    common_validate_call_stack(builder, private_inputs.private_call);

    common_validate_read_requests(
        builder,
        public_inputs.constants.block_data.private_data_tree_root,
        private_inputs.private_call.call_stack_item.public_inputs.read_requests,  // read requests from private call
        private_inputs.private_call.read_request_membership_witnesses);


    // TODO(dbanks12): feels like update_end_values should happen later
    common_update_end_values(builder, private_inputs.private_call, public_inputs);

    // ensure that historic/purported contract tree root matches the one in previous kernel
    validate_contract_tree_root(builder, private_inputs);

    const auto private_call_stack_item = private_inputs.private_call.call_stack_item;
    common_contract_logic(builder,
                          private_inputs.private_call,
                          public_inputs,
                          private_call_stack_item.public_inputs.contract_deployment_data,
                          private_call_stack_item.function_data);

    // This is where a real circuit would perform recursive verification of the previous kernel proof and private call
    // proof.

    // Note: given that we skipped the verify_proof function, the aggregation object we get at the end will just be
    // the same as we had at the start. public_inputs.end.aggregation_object = aggregation_object;
    public_inputs.end.aggregation_object = private_inputs.previous_kernel.public_inputs.end.aggregation_object;

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel