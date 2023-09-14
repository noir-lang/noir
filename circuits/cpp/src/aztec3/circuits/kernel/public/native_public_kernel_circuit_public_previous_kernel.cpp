#include "native_public_kernel_circuit_public_previous_kernel.hpp"

#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

// Purpose of this anonymous namespace is to avoid to clash with the validate_inputs()
// counterpart defined in native_public_kernel_circuit_private_previous_kernel.cpp
namespace {
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;
using aztec3::circuits::kernel::public_kernel::NT;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;

/**
 * @brief Validates the kernel circuit inputs specific to having a public previous kernel
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
void validate_inputs(DummyBuilder& builder, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    const auto& previous_kernel = public_kernel_inputs.previous_kernel.public_inputs;
    builder.do_assert(previous_kernel.is_private == false,
                      "Previous kernel must be public when in this public kernel version",
                      CircuitErrorCode::PUBLIC_KERNEL__PREVIOUS_KERNEL_NOT_PUBLIC);
}
}  // namespace

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::kernel::public_kernel::common_validate_kernel_execution;


/**
 * @brief Entry point for the native public kernel circuit with a public previous kernel
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 * @return The circuit public inputs
 */
KernelCircuitPublicInputs<NT> native_public_kernel_circuit_public_previous_kernel(
    DummyBuilder& builder, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    // construct the circuit outputs
    KernelCircuitPublicInputs<NT> public_inputs{};

    // initialise the end state with our provided previous kernel state
    common_initialise_end_values(public_kernel_inputs, public_inputs);

    // validate the inputs common to all invocation circumstances
    common_validate_inputs(builder, public_kernel_inputs);

    // validate the inputs unique to having a previous public kernel
    validate_inputs(builder, public_kernel_inputs);

    // validate the kernel execution common to all invocation circumstances
    common_validate_kernel_execution(builder, public_kernel_inputs);

    // validate our public call hash
    validate_this_public_call_hash(builder, public_kernel_inputs, public_inputs);

    // update the public end state of the circuit
    common_update_public_end_values(builder, public_kernel_inputs, public_inputs);

    accumulate_unencrypted_logs<NT>(public_kernel_inputs, public_inputs);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::public_kernel