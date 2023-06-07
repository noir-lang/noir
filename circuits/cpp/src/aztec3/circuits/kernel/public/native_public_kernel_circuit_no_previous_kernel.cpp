#include "native_public_kernel_circuit_no_previous_kernel.hpp"

#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs_no_previous_kernel.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/circuit_errors.hpp"
#include "aztec3/utils/dummy_composer.hpp"

namespace {

using aztec3::circuits::kernel::public_kernel::common_update_public_end_values;
using aztec3::utils::is_array_empty;
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;

/**
 * @brief Initialises the circuit output end state from provided inputs
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be initialised
 */
void initialise_end_values(PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs,
                           KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    circuit_outputs.constants.tx_context = public_kernel_inputs.signed_tx_request.tx_request.tx_context;
    circuit_outputs.constants.historic_tree_roots = public_kernel_inputs.historic_tree_roots;
}

/**
 * @brief Validates the kernel circuit inputs specific to having no previous kernel
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
void validate_inputs(DummyComposer& composer, PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs)
{
    const auto& this_call_stack_item = public_kernel_inputs.public_call.call_stack_item;
    composer.do_assert(this_call_stack_item.public_inputs.call_context.is_delegate_call == false,
                       "Users cannot make a delegatecall",
                       aztec3::utils::CircuitErrorCode::PUBLIC_KERNEL__DELEGATE_CALL_PROHIBITED_BY_USER);
    composer.do_assert(this_call_stack_item.public_inputs.call_context.is_static_call == false,
                       "Users cannot make a static call",
                       aztec3::utils::CircuitErrorCode::PUBLIC_KERNEL__STATIC_CALL_PROHIBITED_BY_USER);
    composer.do_assert(this_call_stack_item.public_inputs.call_context.storage_contract_address ==
                           this_call_stack_item.contract_address,
                       "Storage contract address must be that of the called contract",
                       aztec3::utils::CircuitErrorCode::PUBLIC_KERNEL__CONTRACT_ADDRESS_MISMATCH);
}

/**
 * @brief Propagates valid (i.e. non-empty) public data reads from this iteration to the circuit output
 * @tparam The type of kernel input
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be populated
 */
void update_public_end_values(DummyComposer& composer,
                              PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs,
                              KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Since it's the first iteration we need to inject the tx hash nullifier
    // Note: If the nullifiers array is not empty and `first_iteration` flag is correctly set a change was made and
    // we need to rework this
    composer.do_assert(is_array_empty(circuit_outputs.end.new_nullifiers),
                       "new_nullifiers array must be empty in a first iteration of public kernel",
                       CircuitErrorCode::PUBLIC_KERNEL__NEW_NULLIFIERS_NOT_EMPTY_IN_FIRST_ITERATION);

    array_push(composer, circuit_outputs.end.new_nullifiers, public_kernel_inputs.signed_tx_request.hash());

    common_update_public_end_values(composer, public_kernel_inputs, circuit_outputs);
}
}  // namespace

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using aztec3::circuits::kernel::public_kernel::common_validate_kernel_execution;

using DummyComposer = aztec3::utils::DummyComposer;

/**
 * @brief Entry point for the native public kernel circuit with no previous kernel
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @return The circuit public inputs
 */
KernelCircuitPublicInputs<NT> native_public_kernel_circuit_no_previous_kernel(
    DummyComposer& composer, PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs)
{
    // TODO(dbanks12): consider rename of public_kernel_inputs to just private_inputs?
    //                 or consider other renaming options
    //                 (confusing since they are private inputs to the public kernel)

    // There is not circuit state carried over from previous iterations.
    // We are construcitng fresh state that will be added to during this circuit execution.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // initialise the circuit end state with defaults and constants from the provided input
    initialise_end_values(public_kernel_inputs, public_inputs);

    // validate the inputs common to all invocation circumstances
    common_validate_inputs(composer, public_kernel_inputs);

    // validate the inputs unique to there being no previous kernel
    validate_inputs(composer, public_kernel_inputs);

    // validate the kernel execution common to all invocation circumstances
    common_validate_kernel_execution(composer, public_kernel_inputs);

    // update the public end state of the circuit
    update_public_end_values(composer, public_kernel_inputs, public_inputs);

    // TODO: check for the existence on the public function in the contract tree
    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::public_kernel