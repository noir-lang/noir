#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/utils/dummy_composer.hpp"

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
using DummyComposer = aztec3::utils::DummyComposer;

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_ordering(
    DummyComposer& composer, PrivateKernelInputsInner<NT> const& private_inputs);

}  // namespace aztec3::circuits::kernel::private_kernel