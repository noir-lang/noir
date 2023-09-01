#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs_final.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_ordering.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <array>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputsFinal;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsOrdering;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

KernelCircuitPublicInputsFinal<NT> native_private_kernel_circuit_ordering(
    DummyBuilder& builder, PrivateKernelInputsOrdering<NT> const& private_inputs);

}  // namespace aztec3::circuits::kernel::private_kernel