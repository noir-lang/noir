#pragma once

#include "init.hpp"

#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/utils/dummy_composer.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateInputs;
// using abis::private_kernel::PublicInputs;
using DummyComposer = aztec3::utils::DummyComposer;

// TODO: decide what to return.
KernelCircuitPublicInputs<NT> native_private_kernel_circuit(DummyComposer& composer,
                                                            PrivateInputs<NT> const& _private_inputs);

} // namespace aztec3::circuits::kernel::private_kernel