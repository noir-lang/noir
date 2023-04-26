#include "init.hpp"

#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include "native_public_kernel_circuit_public_previous_kernel.hpp"
#include "common.hpp"

#include <aztec3/utils/array.hpp>
#include <aztec3/utils/dummy_composer.hpp>
#include "aztec3/constants.hpp"

namespace {
using CircuitErrorCode = aztec3::utils::CircuitErrorCode;

void validate_inputs(DummyComposer& composer, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    const auto& this_call_stack_item = public_kernel_inputs.public_call.call_stack_item;
    composer.do_assert(array_length(this_call_stack_item.public_inputs.public_call_stack) > 0,
                       "Public call stack can't be empty", 
                       CircuitErrorCode::PUBLIC_KERNEL__EMPTY_PUBLIC_CALL_STACK);
    composer.do_assert(public_kernel_inputs.previous_kernel.public_inputs.end.public_call_count > 0,
                       "Public call count can't be zero", 
                       CircuitErrorCode::PUBLIC_KERNEL__ZERO_PUBLIC_CALL_COUNT);
    composer.do_assert(public_kernel_inputs.previous_kernel.public_inputs.is_private == false,
                       "Previous kernel must be public", 
                       CircuitErrorCode::PUBLIC_KERNEL__PREVIOUS_KERNEL_NOT_PUBLIC);
}
} // namespace

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::kernel::public_kernel::common_validate_inputs;
using aztec3::circuits::kernel::public_kernel::common_validate_kernel_execution;
using aztec3::utils::push_array_to_array;

using DummyComposer = aztec3::utils::DummyComposer;

KernelCircuitPublicInputs<NT> native_public_kernel_circuit_public_previous_kernel(
    DummyComposer& composer, PublicKernelInputs<NT> const& public_kernel_inputs)
{
    // construct the circuit outputs
    KernelCircuitPublicInputs<NT> public_inputs{};

    // initialise the end state with our provided previous kernel state
    common_initialise_end_values(public_kernel_inputs, public_inputs);

    // validate the inputs common to all invocation circumstances
    common_validate_inputs(composer, public_kernel_inputs);

    // validate the inputs unique to having a previous public kernel
    validate_inputs(composer, public_kernel_inputs);

    // validate the kernel execution common to all invocation circumstances
    common_validate_kernel_execution(composer, public_kernel_inputs);

    // vallidate our public call hash
    validate_this_public_call_hash(composer, public_kernel_inputs);

    // update the public end state of the circuit
    update_public_end_values(public_kernel_inputs, public_inputs);

    return public_inputs;
};

} // namespace aztec3::circuits::kernel::public_kernel