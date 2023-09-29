#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_initial(DummyBuilder& builder,
                                                                    PrivateKernelInputsInit<NT> const& private_inputs);

}  // namespace aztec3::circuits::kernel::private_kernel