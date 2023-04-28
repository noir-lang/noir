#include "aztec3/utils/circuit_errors.hpp"
#include "init.hpp"

#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs_no_previous_kernel.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include "native_public_kernel_circuit_no_previous_kernel.hpp"
#include "common.hpp"

#include <aztec3/utils/array.hpp>
#include <aztec3/utils/dummy_composer.hpp>
#include <aztec3/circuits/hash.hpp>
#include "aztec3/constants.hpp"

namespace {

/**
 * @brief Initialises the circuit output end state from provided inputs
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @param circuit_outputs The circuit outputs to be initialised
 */
void initialise_end_values(PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs,
                           KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    circuit_outputs.constants.tx_context = public_kernel_inputs.signed_tx_request.tx_request.tx_context;
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
} // namespace

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
    // There is not circuit state carried over from previous iterations.
    // We are construcitng fresh state that will be added to during this circuit execution.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // initialise the circuit end state with defaults and constants from the provided input
    initialise_end_values(public_kernel_inputs, public_inputs);

    // validate the inputs common to all invocation circumstances
    common_validate_inputs(composer, public_kernel_inputs);

    // validate the inputs unique to there being no previous kernel
    validate_inputs(composer, public_kernel_inputs);

    // validate the kernel execution commonn to all invocation circumstances
    common_validate_kernel_execution(composer, public_kernel_inputs);

    // update the public end state of the circuit
    update_public_end_values(public_kernel_inputs, public_inputs);

    // TODO: check for the existence on the public function in the contract tree
    return public_inputs;
};

} // namespace aztec3::circuits::kernel::public_kernel