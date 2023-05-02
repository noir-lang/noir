#pragma once

#include "common.hpp"
#include "init.hpp"

#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs_no_previous_kernel.hpp>
#include <aztec3/utils/dummy_composer.hpp>

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using DummyComposer = aztec3::utils::DummyComposer;

KernelCircuitPublicInputs<NT> native_public_kernel_circuit_no_previous_kernel(
    DummyComposer& composer, PublicKernelInputsNoPreviousKernel<NT> const& public_kernel_inputs);
}  // namespace aztec3::circuits::kernel::public_kernel