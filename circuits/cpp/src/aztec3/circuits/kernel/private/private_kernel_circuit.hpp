#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;

KernelCircuitPublicInputs<NT> private_kernel_circuit(Builder& builder,
                                                     PrivateKernelInputsInner<NT> const& private_inputs,
                                                     bool first_iteration);

}  // namespace aztec3::circuits::kernel::private_kernel