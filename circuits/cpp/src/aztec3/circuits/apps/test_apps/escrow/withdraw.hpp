#pragma once

#include "init.hpp"

#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/apps/function_execution_context.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> withdraw(FunctionExecutionContext& exec_ctx,
                                                NT::fr const& _amount,
                                                NT::fr const& _asset_id,
                                                NT::fr const& _memo,
                                                NT::fr const& _l1_withdrawal_address,
                                                NT::fr const& _fee);

}  // namespace aztec3::circuits::apps::test_apps::escrow