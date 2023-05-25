#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/utils/dummy_composer.hpp"

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::ContractDeploymentData;
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::private_kernel::PrivateCallData;

// TODO(suyash): Add comments to these as well as other functions in PKC-init.
void common_validate_call_stack(DummyComposer& composer, PrivateCallData<NT> const& private_call);

void common_update_end_values(DummyComposer& composer,
                              PrivateCallData<NT> const& private_call,
                              KernelCircuitPublicInputs<NT>& public_inputs);

void common_contract_logic(DummyComposer& composer,
                           PrivateCallData<NT> const& private_call,
                           KernelCircuitPublicInputs<NT>& public_inputs,
                           ContractDeploymentData<NT> const& contract_dep_data,
                           FunctionData<NT> const& function_data);
}  // namespace aztec3::circuits::kernel::private_kernel