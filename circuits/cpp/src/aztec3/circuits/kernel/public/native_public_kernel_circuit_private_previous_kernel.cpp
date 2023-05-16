#include "common.hpp"
#include "init.hpp"
#include "native_public_kernel_circuit_public_previous_kernel.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/utils/array.hpp>
#include <aztec3/utils/dummy_composer.hpp>

namespace {
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;

/**
 * @brief Validates the kernel circuit inputs specific to having a private previous kernel
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
void validate_inputs(DummyComposer& composer, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    composer.do_assert(array_length(public_kernel_inputs.previous_kernel.public_inputs.end.private_call_stack) == 0,
                       "Private call stack must be empty",
                       CircuitErrorCode::PUBLIC_KERNEL__NON_EMPTY_PRIVATE_CALL_STACK);
    composer.do_assert(public_kernel_inputs.previous_kernel.public_inputs.end.private_call_count > 0,
                       "Private call count can't be zero",
                       CircuitErrorCode::PUBLIC_KERNEL__ZERO_PRIVATE_CALL_COUNT);
    composer.do_assert(public_kernel_inputs.previous_kernel.public_inputs.end.public_call_count == 0,
                       "Public call count must be zero",
                       CircuitErrorCode::PUBLIC_KERNEL__NON_ZERO_PUBLIC_CALL_COUNT);
    composer.do_assert(public_kernel_inputs.previous_kernel.public_inputs.is_private == true,
                       "Previous kernel must be private",
                       CircuitErrorCode::PUBLIC_KERNEL__PREVIOUS_KERNEL_NOT_PRIVATE);
}
}  // namespace

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::kernel::public_kernel::common_initialise_end_values;
using aztec3::circuits::kernel::public_kernel::common_validate_kernel_execution;

using DummyComposer = aztec3::utils::DummyComposer;

/**
 * @brief Entry point for the native public kernel circuit with a private previous kernel
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @return The circuit public inputs
 */
KernelCircuitPublicInputs<NT> native_public_kernel_circuit_private_previous_kernel(
    DummyComposer& composer, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    // construct the circuit outputs
    KernelCircuitPublicInputs<NT> public_inputs{};

    // initialise the end state with our provided previous kernel state
    common_initialise_end_values(public_kernel_inputs, public_inputs);

    // validate the inputs common to all invocation circumstances
    common_validate_inputs(composer, public_kernel_inputs);

    // validate the inputs unique to having a previous private kernel
    validate_inputs(composer, public_kernel_inputs);

    // validate the kernel execution common to all invocation circumstances
    common_validate_kernel_execution(composer, public_kernel_inputs);

    // vallidate our public call hash
    validate_this_public_call_hash(composer, public_kernel_inputs, public_inputs);

    // update the public end state of the circuit
    update_public_end_values(public_kernel_inputs, public_inputs);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::public_kernel