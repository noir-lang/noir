#pragma once
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_executor.hpp>
#include "init.hpp"

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> deposit(FunctionExecutionContext<Composer>& exec_ctx,
                                               NT::fr const& _amount,
                                               NT::fr const& _asset_id,
                                               NT::fr const& _memo);

} // namespace aztec3::circuits::apps::test_apps::escrow