#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs_final.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <array>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputsFinal;
using aztec3::circuits::abis::PreviousKernelData;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

KernelCircuitPublicInputsFinal<NT> native_private_kernel_circuit_ordering(
    DummyBuilder& builder, PreviousKernelData<NT> const& previous_kernel);

}  // namespace aztec3::circuits::kernel::private_kernel