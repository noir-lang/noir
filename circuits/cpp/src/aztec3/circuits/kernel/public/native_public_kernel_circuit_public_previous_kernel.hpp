#pragma once

#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

KernelCircuitPublicInputs<NT> native_public_kernel_circuit_public_previous_kernel(
    DummyBuilder& builder, PublicKernelInputs<NT> const& public_kernel_inputs);
}  // namespace aztec3::circuits::kernel::public_kernel