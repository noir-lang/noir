#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/abis/read_request_membership_witness.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"


namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::ReadRequestMembershipWitness;
using aztec3::circuits::abis::private_kernel::PrivateCallData;

using DummyBuilder = aztec3::utils::DummyCircuitBuilder;


// TODO(suyash): Add comments to these as well as other functions in PKC-init.
void common_validate_call_stack(DummyBuilder& builder, PrivateCallData<NT> const& private_call);

void common_validate_read_requests(DummyBuilder& builder,
                                   NT::fr const& historic_private_data_tree_root,
                                   std::array<fr, MAX_READ_REQUESTS_PER_CALL> const& read_requests,
                                   std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>,
                                              MAX_READ_REQUESTS_PER_CALL> const& read_request_membership_witnesses);

void common_validate_previous_kernel_read_requests(
    DummyBuilder& builder,
    std::array<NT::fr, MAX_READ_REQUESTS_PER_TX> const& read_requests,
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX> const&
        read_request_membership_witnesses);

void common_update_end_values(DummyBuilder& builder,
                              PrivateCallData<NT> const& private_call,
                              KernelCircuitPublicInputs<NT>& public_inputs);

void common_contract_logic(DummyBuilder& builder,
                           PrivateCallData<NT> const& private_call,
                           KernelCircuitPublicInputs<NT>& public_inputs,
                           ContractDeploymentData<NT> const& contract_dep_data,
                           FunctionData<NT> const& function_data);

void common_initialise_end_values(PreviousKernelData<NT> const& previous_kernel,
                                  KernelCircuitPublicInputs<NT>& public_inputs);

}  // namespace aztec3::circuits::kernel::private_kernel