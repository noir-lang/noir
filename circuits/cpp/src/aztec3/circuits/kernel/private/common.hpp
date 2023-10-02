#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/contract_deployment_data.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/private_kernel/private_call_data.hpp"
#include "aztec3/circuits/abis/read_request_membership_witness.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"


namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::CombinedAccumulatedData;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
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

void common_validate_arrays(DummyBuilder& builder, PrivateCircuitPublicInputs<NT> const& app_public_inputs);
void common_validate_previous_kernel_arrays(DummyBuilder& builder, CombinedAccumulatedData<NT> const& end);
void common_validate_previous_kernel_values(DummyBuilder& builder, CombinedAccumulatedData<NT> const& end);
void common_validate_previous_kernel_0th_nullifier(DummyBuilder& builder, CombinedAccumulatedData<NT> const& end);

void common_update_end_values(DummyBuilder& builder,
                              PrivateCallData<NT> const& private_call,
                              KernelCircuitPublicInputs<NT>& public_inputs);

void common_contract_logic(DummyBuilder& builder,
                           PrivateCallData<NT> const& private_call,
                           KernelCircuitPublicInputs<NT>& public_inputs,
                           ContractDeploymentData<NT> const& contract_dep_data,
                           FunctionData<NT> const& function_data);

template <typename KernelPublicInputs>
void common_initialise_end_values(PreviousKernelData<NT> const& previous_kernel, KernelPublicInputs& public_inputs)
{
    public_inputs.constants = previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    auto& end = public_inputs.end;
    const auto& start = previous_kernel.public_inputs.end;

    end.new_commitments = start.new_commitments;
    end.new_nullifiers = start.new_nullifiers;
    end.nullified_commitments = start.nullified_commitments;

    end.private_call_stack = start.private_call_stack;
    end.public_call_stack = start.public_call_stack;
    end.new_l2_to_l1_msgs = start.new_l2_to_l1_msgs;

    end.encrypted_logs_hash = start.encrypted_logs_hash;
    end.unencrypted_logs_hash = start.unencrypted_logs_hash;

    end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;

    end.optionally_revealed_data = start.optionally_revealed_data;
}

}  // namespace aztec3::circuits::kernel::private_kernel